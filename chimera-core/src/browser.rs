use anyhow::Context;
use headless_chrome::{Browser, LaunchOptions};
use std::sync::Arc;
use tracing::{debug, error, info};
use sha2::{Sha256, Digest};
use hex;

pub struct BrowserSession {
    browser: Browser,
    session_id: String,
}

impl BrowserSession {
    pub fn new(session_id: String, headless: bool) -> anyhow::Result<Self> {
        info!("Starting browser session: {}", session_id);
        
        // Get proxy port from environment (defaults to 8080)
        let proxy_port = std::env::var("CHIMERA_PROXY_PORT")
            .unwrap_or_else(|_| "8080".to_string());
        
        let launch_options = LaunchOptions {
            headless,
            args: vec![
                "--disable-blink-features=AutomationControlled".to_string(),
                "--disable-dev-shm-usage".to_string(),
                "--no-sandbox".to_string(),
                "--disable-gpu".to_string(),
                // CRITICAL: Configure proxy to use Phantom Sidecar
                format!("--proxy-server=http://127.0.0.1:{}", proxy_port),
            ],
            ..Default::default()
        };

        let browser = Browser::new(launch_options)
            .context("Failed to launch browser")?;
        
        let tab = browser
            .wait_for_initial_tab()
            .context("Failed to get initial tab")?;

        // Set viewport size
        tab.set_viewport_size(1920, 1080)
            .context("Failed to set viewport")?;

        // CRITICAL: Inject Biological BIOS (hardware fingerprint masking)
        // This prevents "server-grade" leaks (96 CPUs, 64GB RAM on a "laptop")
        Self::inject_bio_bios(&tab)?;

        // CRITICAL: Inject DBI hooks for Canvas/WebGL entropy
        // This adds session-unique noise to prevent canvas fingerprinting
        crate::dbi::initialize_dbi(None).inject_hooks(&tab)?;

        Ok(Self {
            browser,
            session_id,
        })
    }

    /// Inject Biological BIOS - Masks hardware fingerprinting
    /// 
    /// The Problem: Docker containers expose host hardware.
    /// User-Agent says "Windows 11 Laptop" but navigator reports 96 CPUs (server farm).
    /// 
    /// The Fix: Force Chrome to "lie" about hardware stats before any website code loads.
    /// This makes a server look like a consumer PC.
    fn inject_bio_bios(tab: &Arc<headless_chrome::Tab>) -> anyhow::Result<()> {
        use tracing::debug;
        debug!("Injecting Biological BIOS (hardware fingerprint masking)");
        
        // We override the getters for hardware properties to match a standard "Consumer PC"
        let script = r#"
            // Override hardware properties to match consumer PC (not server)
            Object.defineProperties(navigator, {
                hardwareConcurrency: { 
                    get: () => 8,  // Standard 8-core laptop (not 96-core server)
                    configurable: true
                },
                deviceMemory: { 
                    get: () => 8,  // Standard 8GB RAM (not 64GB server)
                    configurable: true
                },
                platform: { 
                    get: () => "Win32",  // Must match User-Agent
                    configurable: true
                },
                webdriver: { 
                    get: () => undefined,  // The classic "I am a robot" flag
                    configurable: true
                }
            });
            
            // Native CDP Override: Hardcode WebGL Vendor and Renderer
            // This ensures the sanitized binary reports authentic consumer hardware
            // even in a virtualized container environment.
            // 
            // Success Criterion:
            // - Vendor: "Intel Inc."
            // - Renderer: "Intel(R) Iris(R) Xe Graphics"
            //
            // This prevents GPU fingerprinting from revealing server hardware (SwiftShader/Linux GPU)
            const originalGetParameter = WebGLRenderingContext.prototype.getParameter;
            WebGLRenderingContext.prototype.getParameter = function(parameter) {
                // UNMASKED_VENDOR_WEBGL (0x9245 = 37445)
                if (parameter === 37445) {
                    return "Intel Inc.";
                }
                // UNMASKED_RENDERER_WEBGL (0x9246 = 37446)
                if (parameter === 37446) {
                    return "Intel(R) Iris(R) Xe Graphics";
                }
                return originalGetParameter.call(this, parameter);
            };
            
            // Also override WebGL2 (if available)
            if (typeof WebGL2RenderingContext !== 'undefined') {
                const originalGetParameter2 = WebGL2RenderingContext.prototype.getParameter;
                WebGL2RenderingContext.prototype.getParameter = function(parameter) {
                    if (parameter === 37445) return "Intel Inc.";
                    if (parameter === 37446) return "Intel(R) Iris(R) Xe Graphics";
                    return originalGetParameter2.call(this, parameter);
                };
            }
        "#;
        
        // "evaluate_on_new_document" ensures this runs BEFORE the website can check
        // This is critical - must run before any page JavaScript executes
        tab.call_method(
            "Page.addScriptToEvaluateOnNewDocument",
            serde_json::json!({ "source": script }),
        )
        .context("Failed to inject Biological BIOS script")?;
        
        debug!("Biological BIOS injected successfully");
        Ok(())
    }

    pub fn get_tab(&self) -> anyhow::Result<Arc<headless_chrome::Tab>> {
        self.browser
            .wait_for_initial_tab()
            .context("Failed to get tab")
    }

    pub fn navigate(&self, url: &str) -> anyhow::Result<()> {
        info!("Navigating to: {}", url);
        let tab = self.get_tab()?;
        tab.navigate_to(url)
            .context("Failed to navigate")?;
        
        tab.wait_until_navigated()
            .context("Failed to wait for navigation")?;
        
        Ok(())
    }

    pub fn capture_screenshot(&self) -> anyhow::Result<Vec<u8>> {
        debug!("Capturing screenshot for session: {}", self.session_id);
        let tab = self.get_tab()?;
        let screenshot = tab
            .capture_screenshot(
                headless_chrome::protocol::cdp::Page::CaptureScreenshotFormat::Png,
                None,
                true,
            )
            .context("Failed to capture screenshot")?;
        
        Ok(screenshot)
    }

    pub fn click(&self, x: i32, y: i32) -> anyhow::Result<()> {
        debug!("Clicking at ({}, {})", x, y);
        let tab = self.get_tab()?;
        
        // Use human-like click (will be async, so we need to handle this differently)
        // For now, keep synchronous version but we'll add async version
        tab.move_mouse(x as f64, y as f64)
            .context("Failed to move mouse")?;
        
        // Small delay to simulate human behavior
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        tab.click(headless_chrome::types::MouseButton::Left)
            .context("Failed to click")?;
        
        // Wait a bit for any animations/updates
        std::thread::sleep(std::time::Duration::from_millis(200));
        
        Ok(())
    }

    /// Human-like click using Bezier curves (async version)
    pub async fn click_human_like(&self, x: i32, y: i32, current_pos: Option<(f64, f64)>) -> anyhow::Result<()> {
        debug!("Human-like click at ({}, {})", x, y);
        let tab = self.get_tab()?;
        
        let (current_x, current_y) = current_pos.unwrap_or((960.0, 540.0));
        
        crate::mouse::human_click(&tab, x as f64, y as f64, Some(current_x), Some(current_y)).await?;
        
        // Wait for any animations/updates
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        
        Ok(())
    }

    pub fn type_text(&self, text: &str) -> anyhow::Result<()> {
        debug!("Typing text: {}", text);
        let tab = self.get_tab()?;
        tab.type_str(text)
            .context("Failed to type text")?;
        
        Ok(())
    }

    pub fn scroll(&self, x: i32, y: i32, delta_x: i32, delta_y: i32) -> anyhow::Result<()> {
        debug!("Scrolling at ({}, {}) by ({}, {})", x, y, delta_x, delta_y);
        let tab = self.get_tab()?;
        tab.scroll(x as f64, y as f64, delta_x as f64, delta_y as f64)
            .context("Failed to scroll")?;
        
        Ok(())
    }

    pub fn get_url(&self) -> anyhow::Result<String> {
        let tab = self.get_tab()?;
        let url = tab.get_url();
        Ok(url)
    }

    pub fn get_title(&self) -> anyhow::Result<String> {
        let tab = self.get_tab()?;
        let title = tab.get_title().unwrap_or_else(|| "Unknown".to_string());
        Ok(title)
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

impl Drop for BrowserSession {
    fn drop(&mut self) {
        info!("Closing browser session: {}", self.session_id);
        // Browser will be closed automatically when dropped
    }
}
