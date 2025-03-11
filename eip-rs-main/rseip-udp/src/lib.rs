use tokio::{net::UdpSocket, time, sync::Mutex};
use bytes::{BytesMut, BufMut};
use std::net::{SocketAddr, Ipv4Addr};
use std::collections::HashMap;
use std::sync::Arc;
use crate::forward_open::ForwardOpenRequest;
use crate::forward_close::ForwardCloseRequest;
use crate::assembly::AssemblyManager;
use crate::explicit::ExplicitMessage;

use tokio::time::{Duration, Instant};
use anyhow::Result;

mod forward_open;
mod forward_close;
mod assembly;
mod explicit;

const CONNECTION_TIMEOUT_MS: u64 = 5000; // 5 seconds timeout

pub struct UdpServer {
    socket: Arc<UdpSocket>,
    multicast_addr: Ipv4Addr,
    active_connections: Arc<Mutex<HashMap<u32, SocketAddr>>>,
    assembly_manager: AssemblyManager,
    input_instance: u16,
    output_instance: u16,
    config_instance: u16,
    input_size: u16,
    output_size: u16,
    config_size: u16,
}

impl UdpServer {
    pub async fn new(
        bind_addr: &str,
        multicast_ip: &str,
        input_instance: u16,
        output_instance: u16,
        config_instance: u16,
        input_size: u16,
        output_size: u16,
        config_size: u16,
    ) -> Result<Self> {
        let socket = Arc::new(UdpSocket::bind(bind_addr).await?);
        let multicast_addr: Ipv4Addr = multicast_ip.parse()?;
        let active_connections = Arc::new(Mutex::new(HashMap::new()));
        let assembly_manager = AssemblyManager::new();

        Ok(Self {
            socket,
            multicast_addr,
            active_connections,
            assembly_manager,
            input_instance,
            output_instance,
            config_instance,
            input_size,
            output_size,
            config_size,
        })
    }

    pub async fn run(self: Arc<Self>) -> Result<()> {
        let send_task = tokio::spawn({
            let server = Arc::clone(&self);
            async move {
                server.start_cyclic_multicast().await
            }
        });

        let recv_task = tokio::spawn({
            let server = Arc::clone(&self);
            async move {
                server.listen_for_packets().await
            }
        });

        let timeout_task = tokio::spawn({
            let server = Arc::clone(&self);
            async move {
                server.monitor_connection_timeouts().await
            }
        });

        let _ = tokio::try_join!(send_task, recv_task, timeout_task)?;
        Ok(())
    }

    async fn listen_for_packets(&self) -> Result<()> {
        loop {
            let mut buf = [0; 1024];
            let (len, addr) = self.socket.recv_from(&mut buf).await?;
            log::info!("Received {} bytes from {}", len, addr);
            self.process_packet(&buf[..len], addr).await?;
        }
    }

    async fn process_packet(&self, data: &[u8], addr: SocketAddr) -> Result<()> {
        let mut buf = BytesMut::from(data);

        if let Some(forward_open) = ForwardOpenRequest::parse(&mut buf) {
            log::info!("Processing Forward Open Request: {:?}", forward_open);
            let mut connections = self.active_connections.lock().await;
            connections.insert(forward_open.connection_id, addr);
            self.assembly_manager
                .register(forward_open.connection_id, forward_open.requested_packet_interval)
                .await;
            let response = self.create_forward_open_response();
            self.socket.send_to(&response, addr).await?;
            return Ok(());
        }

        if let Some(forward_close) = ForwardCloseRequest::parse(&mut buf) {
            log::info!("Processing Forward Close Request: {:?}", forward_close);
            let mut connections = self.active_connections.lock().await;
            connections.remove(&forward_close.connection_id);
            return Ok(());
        }

        Ok(())
    }

    async fn monitor_connection_timeouts(&self) -> Result<()> {
        let active_connections = Arc::clone(&self.active_connections);
        let assembly_manager = self.assembly_manager.clone();

        tokio::spawn(async move {
            loop {
                let timed_out = assembly_manager.cleanup_timed_out_connections(CONNECTION_TIMEOUT_MS).await;
                if !timed_out.is_empty() {
                    let mut connections = active_connections.lock().await;
                    for conn_id in timed_out {
                        connections.remove(&conn_id);
                        log::warn!("Connection {} timed out and was removed.", conn_id);
                    }
                }
                time::sleep(Duration::from_millis(1000)).await;
            }
        });

        Ok(())
    }

    async fn start_cyclic_multicast(&self) -> Result<()> {
        let socket = Arc::clone(&self.socket);
        let multicast_addr = self.multicast_addr;
        let assembly_manager = self.assembly_manager.clone();

        tokio::spawn(async move {
            loop {
                let ready_objects = assembly_manager.get_ready_transmissions().await;
                for assembly in ready_objects {
                    let mut response = Vec::new();
                    response.put_u32_le(assembly.connection_id);
                    response.extend_from_slice(&assembly.data);
                    let multicast_socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
                    multicast_socket
                        .send_to(&response, SocketAddr::new(multicast_addr.into(), 2222))
                        .await
                        .unwrap();
                }
                time::sleep(Duration::from_millis(5)).await;
            }
        });

        Ok(())
    }

    fn create_forward_open_response(&self) -> Vec<u8> {
        let mut response = Vec::new();
        response.put_u8(0xD4); // Success code
        response.put_u16_le(self.input_instance);
        response.put_u16_le(self.output_instance);
        response.put_u16_le(self.config_instance);
        response.put_u16_le(self.input_size);
        response.put_u16_le(self.output_size);
        response.put_u16_le(self.config_size);
        response
    }
}