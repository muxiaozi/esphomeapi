use esphomeapi_rs::{Result, Connection};

#[tokio::main]
async fn main() -> Result<()> {
    let mut connection = Connection::new("athom-smart-plug-v3-f97fdc.local".to_string(), 6053, None, Some("udPCdNIAc416KwATtWZr5M0Dl9oNd07Mh0k2r0jFsso=".to_string()));
    connection.connect().await?;
    Ok(())
}