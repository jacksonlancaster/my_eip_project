use anyhow::{Result, anyhow};
use byteorder::{ByteOrder, LittleEndian};
use bytes::{BytesMut, BufMut};
use rseip::client::ab_eip::AbEipClient;
use rseip::cip::{MessageReply, MessageRequest};
use rseip::precludes::MessageService;
use crate::implicit::{ImplicitClient, ImplicitConnection};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Bni008m {
    explicit_client: Arc<Mutex<AbEipClient>>,
    implicit_client: ImplicitClient,
    implicit_conn: Option<ImplicitConnection>,
    ip_address: String,
}

impl Bni008m {

    pub async fn diagnose_explicit_assemblies(&self) -> Result<()> {
        let mut client = self.explicit_client.lock().await;

        // Assembly 100 (input)
        let read_input = MessageRequest::new(0x0E, vec![0x20, 0x04, 0x24, 100, 0x30, 0x03], &[] as &[u8]);


        let input_resp: MessageReply<Vec<u8>> = client.send(read_input).await?;
        println!("Assembly 100 (Input) explicit read: {:?}", input_resp.data);

       // Assembly 101 (output)
let read_output = MessageRequest::new(0x0E, vec![0x20, 0x04, 0x24, 101, 0x30, 0x03], &[] as &[u8]);
let output_resp: MessageReply<Vec<u8>> = client.send(read_output).await?;
        println!("Assembly 101 (Output) explicit read: {:?}", output_resp.data);

        // Assembly 102 (config)
let read_config = MessageRequest::new(0x0E, vec![0x20, 0x04, 0x24, 102, 0x30, 0x03], &[] as &[u8]);
        let config_resp: MessageReply<Vec<u8>> = client.send(read_config).await?;
        println!("Assembly 102 (Config) explicit read: {:?}", config_resp.data);

        Ok(())
    }
    pub async fn new(ip_address: &str) -> Result<Self> {
        let explicit_client = AbEipClient::new_host_lookup(ip_address).await?;
        let implicit_client = ImplicitClient::new(ip_address).await?;
        Ok(Self {
            explicit_client: Arc::new(Mutex::new(explicit_client)),
            implicit_client,
            implicit_conn: None,
            ip_address: ip_address.to_string(),
        })
    }

    pub async fn read_inputs_explicit(&mut self) -> Result<Vec<bool>> {
        let path: Vec<u8> = vec![0x20, 0x04, 0x24, 100, 0x30, 0x03];
        let mr = MessageRequest::new(0x0E, path, &[] as &[u8]); // Explicitly typed
        let mut client = self.explicit_client.lock().await;
        let response: MessageReply<Vec<u8>> = client.send(mr).await?;
        let response_data = response.data;
    
        if response_data.len() < 2 {
            return Err(anyhow!("Input data too short"));
        }
    
        let val = LittleEndian::read_u16(&response_data);
        let inputs = (0..16).map(|i| (val & (1 << i)) != 0).collect();
    
        Ok(inputs)
    }
    
    pub async fn write_outputs_explicit(&mut self, outputs: &[bool]) -> Result<()> {
        let mut data = BytesMut::with_capacity(2);
        let mut val = 0u16;
        for (i, &state) in outputs.iter().take(16).enumerate() {
            if state {
                val |= 1 << i;
            }
        }
        data.put_u16_le(val);
    
        let path: Vec<u8> = vec![0x20, 0x04, 0x24, 101, 0x30, 0x03];
        let mr = MessageRequest::new(0x10, path, &data[..]); // slice with &data[..]
        let mut client = self.explicit_client.lock().await;
        let _: MessageReply<()> = client.send(mr).await?;
    
        Ok(())
    }
    
    pub async fn configure_port(&mut self, port: u8, mode: PortMode) -> Result<()> {
        let data = [port, mode as u8]; // Direct array
        let path: Vec<u8> = vec![0x20, 0x04, 0x24, 102, 0x30, 0x03];
        let mr = MessageRequest::new(0x10, path, &data as &[u8]); // explicit &[u8] slice
        let mut client = self.explicit_client.lock().await;
        let _: MessageReply<()> = client.send(mr).await?;
    
        Ok(())
    }

    pub async fn start_implicit(&mut self, rpi_ms: u32) -> Result<()> {
        if self.implicit_conn.is_none() {
            println!("Starting implicit connection with RPI {} ms", rpi_ms);
            let conn = self.implicit_client.open_connection(rpi_ms).await?;
            self.implicit_conn = Some(conn);
        }
        Ok(())
    }

    pub async fn read_inputs(&self) -> Result<Vec<bool>> {
        match &self.implicit_conn {
            Some(conn) => Ok(conn.read_inputs().await),
            None => Err(anyhow!("Implicit connection not started")),
        }
    }

    pub async fn write_outputs(&self, outputs: &[bool]) -> Result<()> {
        match &self.implicit_conn {
            Some(conn) => conn.write_outputs(outputs).await,
            None => Err(anyhow!("Implicit connection not started")),
        }
    }

    pub async fn close(&mut self) -> Result<()> {
        if let Some(conn) = &self.implicit_conn {
            println!("Closing implicit connection.");
            conn.close().await?;
        }
        self.explicit_client.lock().await.close().await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PortMode {
    Digital = 0,
    IoLink = 1,
}
