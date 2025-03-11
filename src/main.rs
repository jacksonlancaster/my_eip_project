use rseip::client::eip::EipClient;
//use rseip_udp::{ForwardOpenRequest, ForwardOpenOptions};
use rseip_udp::forward_open::ForwardOpenRequest;
use tokio::time::{sleep, Duration};
use anyhow::Result;
use std::net::{Ipv4Addr, SocketAddrV4};

#[tokio::main]
async fn main() -> Result<()> {
    // Create SocketAddrV4 explicitly (Ethernet/IP standard port 44818)
    let device_addr = SocketAddrV4::new(Ipv4Addr::new(192, 168, 50, 40), 44818);

    let input_instance = 100;
    let input_size = 392; // bytes
    let output_instance = 101;
    let output_size = 262; // bytes
    let config_instance = 102;

    // Create EipClient synchronously (no await)
    let mut client = EipClient::new(device_addr);

    // Prepare ForwardOpenRequest for implicit messaging
    let forward_open_request = ForwardOpenRequest {
        input_instance,
        input_size: input_size as u16,
        output_instance,
        output_size: output_size as u16,
        config_instance,
        config_data: vec![],
    };

    // Open implicit connection (await needed here)
    let connection = forward_open(
        &mut client,
        forward_open_request,
        ForwardOpenOptions {
            rpi_ms: 10,
            timeout_multiplier: 3,
        },
    )
    .await?;

    println!("Implicit connection established!");

    // Data exchange loop
    for _ in 0..10 {
        if let Some(input_data) = connection.receive().await? {
            println!("Input data received: {:?}", input_data);
        }

        let output_data = vec![0u8; output_size];
        connection.send(&output_data).await?;
        println!("Output data sent.");

        sleep(Duration::from_millis(10)).await;
    }

    connection.close().await?;
    println!("Connection closed.");

    Ok(())
}
