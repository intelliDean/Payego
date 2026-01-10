#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    payego::run().await
}
