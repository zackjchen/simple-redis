use anyhow::Result;
use redis::{backend::Backend, network::stream_handler};
use tokio::net::TcpListener;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let addr = "0.0.0.0:6379";
    info!("Starting redis server on {}", addr);
    let listener = TcpListener::bind(addr).await?;
    let backend = Backend::new();
    loop {
        let (socket, remote_addr) = listener.accept().await?;
        // backend is Arc<BackendInner>
        let backend = backend.clone();
        info!("Accepted connection from {}", remote_addr);
        tokio::spawn(async move {
            match stream_handler(socket, backend).await {
                Ok(_) => info!("Connection from {} is exited", remote_addr),
                Err(e) => {
                    warn!("error processing connection: {}, error:{}", addr, e);
                }
            }
        });
    }
}
