use my_eip_project::Bni008m;
use anyhow::Result;

// #[tokio::main]
// async fn main() -> Result<()> {
//     let mut bni = Bni008m::new("192.168.50.40").await?; // Replace with your device's IP
//     bni.start_implicit(10).await?;
//     let inputs = bni.read_inputs().await?;
//     println!("Inputs: {:?}", inputs);
//     bni.write_outputs(&[true, false, true]).await?;
//     bni.close().await?;
//     Ok(())
// }


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut bni = Bni008m::new("192.168.50.40").await?;
    bni.start_implicit(10).await?;
    //let inputs = bni.read_inputs().await?;
    //println!("Inputs: {:?}", inputs);
    //bni.write_outputs(&[true, false, true]).await?;
    bni.close().await?;
    Ok(())
    // Run diagnostics before implicit start:
    //bni.diagnose_explicit_assemblies().await?;

    //Ok(())
}
