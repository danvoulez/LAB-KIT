use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    logline_lab_kit::cli::run().await
}
