use chimera_core::agent::ChimeraAgentService;
use chimera_core::browser::BrowserSession;
use chimera_core::proto::chimera_agent_server::ChimeraAgentServer;
use chimera_core::stealth_transport::StealthProxy;
use std::env;
use std::time::Duration;
use tonic::transport::Server;
use tracing::{error, info, warn, Level};

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
    
    // Update vision address to use brainscraper service
    let vision_addr = env::var("CHIMERA_VISION_ADDR")
        .unwrap_or_else(|_| "http://brainscraper.railway.internal:50052".to_string());

    info!("🚀 Launching Chimera with Phantom Sidecar active...");
    info!("Starting Chimera Agent Service on {}", agent_addr);
    info!("Vision service address: {}", vision_addr);

    // 2. BINARY SANITIZATION VERIFICATION
    // Verify that the binary patching was successful before starting the service
    info!("🔍 Verifying binary sanitization...");
    
    // Initialize binary patching (if not already done in Dockerfile)
    if let Err(e) = chimera_core::binary_patch::initialize_binary_patching() {
        warn!("Binary patching initialization failed (may already be sanitized): {}", e);
    }
    
    // Create a temporary browser session to verify sanitization
    // This ensures the engine is "Sanitized and Ready" before accepting missions
    let sanitization_verified = {
        info!("🧪 Creating test browser session to verify sanitization...");
        match BrowserSession::new("sanitization_test".to_string(), true) {
            Ok(test_session) => {
                match test_session.get_tab() {
                    Ok(tab) => {
                        let cortex = chimera_core::cortex::Cortex::new(tab);
                        match cortex.verify_engine_health() {
                            Ok(true) => {
                                info!("✅ Binary sanitization verified: Engine is Sanitized and Ready");
                                true
                            }
                            Ok(false) => {
                                error!("❌ Binary sanitization verification FAILED: navigator.webdriver is still present!");
                                error!("   The engine is NOT sanitized. Missions will fail.");
                                false
                            }
                            Err(e) => {
                                error!("❌ Failed to verify sanitization: {}", e);
                                false
                            }
                        }
                    }
                    Err(e) => {
                        error!("❌ Failed to get test tab: {}", e);
                        false
                    }
                }
            }
            Err(e) => {
                error!("❌ Failed to create test browser session: {}", e);
                false
            }
        }
    };
    
    if !sanitization_verified {
        error!("🚨 FATAL: Binary sanitization verification failed!");
        error!("   The Body cannot start missions until the engine is sanitized.");
        error!("   Check that sanitize_binary.py ran successfully in the Dockerfile.");
        std::process::exit(1);
    }
    
    info!("✅ Body Status: Sanitized and Ready");
    info!("   - Binary patching: ✅ Verified");
    info!("   - Engine DNA: ✅ Clean");
    info!("   - Automation markers: ✅ Erased");
    info!("   - Ready for missions");

    let service = ChimeraAgentService::new(vision_addr);
    let addr = agent_addr.parse()?;

    Server::builder()
        .add_service(ChimeraAgentServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
