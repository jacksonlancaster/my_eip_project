pub mod implicit;
pub mod balluff;



pub use implicit::{ImplicitClient, ImplicitConnection};
pub use balluff::bni008m::Bni008m;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut bni = Bni008m::new("192.168.0.100").await?;
    bni.start_implicit(10).await?;
    let inputs = bni.read_inputs().await?;
    println!("Inputs: {:?}", inputs);
    bni.write_outputs(&[true, false, true]).await?;
    bni.close().await?;
    Ok(())
}