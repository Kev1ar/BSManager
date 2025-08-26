#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Orange Pi Control starting up...");

    // Run two async tasks in parallel:
    let backend_handle = task::spawn(async {
        backend::run().await.unwrap();
    });

    let esp32_handle = task::spawn(async {
        esp32::run().await.unwrap();
    });

    // Wait for them
    backend_handle.await?;
    esp32_handle.await?;

    Ok(())
}