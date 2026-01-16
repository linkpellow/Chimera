use chimera_core::agent::ChimeraAgentService;
use chimera_core::proto::chimera_agent_server::ChimeraAgentServer;
use chimera_core::stealth_transport::StealthProxy;
use std::env;
use std::time::Duration;
use tonic::transport::Server;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    // 1. IGNITE THE PHANTOM PROXY (Sidecar)
    // We spawn it in the background on port 8080.
    // This intercepts all Chrome traffic and launders it through our impersonation engine.
    info!("👻 Starting Phantom Sidecar (Stealth Proxy)...");
    
    let proxy_port = env::var("CHIMERA_PROXY_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap_or(8080);
    
    tokio::spawn(async move {
        let proxy = match StealthProxy::new(proxy_port) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("FATAL: Failed to create Phantom Proxy: {}", e);
                return;
            }
        };
        
        if let Err(e) = proxy.serve().await {
            eprintln!("FATAL: Phantom Proxy died: {}", e);
        }
    });

    // Give it 500ms to warm up
    tokio::time::sleep(Duration::from_millis(500)).await;
    info!("✅ Phantom Sidecar ready on port {}", proxy_port);

    // Get configuration from environment
    let agent_addr = env::var("CHIMERA_AGENT_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50051".to_string());
    
    let vision_addr = env::var("CHIMERA_VISION_ADDR")
        .unwrap_or_else(|_| "http://127.0.0.1:50052".to_string());

    info!("🚀 Launching Chimera with Phantom Sidecar active...");
    info!("Starting Chimera Agent Service on {}", agent_addr);
    info!("Vision service address: {}", vision_addr);

    let service = ChimeraAgentService::new(vision_addr);
    let addr = agent_addr.parse()?;

    Server::builder()
        .add_service(ChimeraAgentServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
