/// OODA Loop Implementation - The Nervous System
/// 
/// Observe-Orient-Decide-Act loop with visual verification
/// This is what makes Chimera self-healing and resilient.

use crate::browser::BrowserSession;
use crate::cortex::AxTree;
use crate::error::{ChimeraError, Result};
use crate::vision_client::VisionClient;
use rand::Rng;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info, warn};

/// Execute an action with visual verification (OODA Loop)
/// 
/// This implements the "Nervous System" - it doesn't just hope the action worked,
/// it verifies that the screen actually changed.
/// 
/// # OODA Loop:
/// 1. **Observe**: Capture screenshot and hash it
/// 2. **Orient**: Brain identifies target coordinates
/// 3. **Decide**: Determine action to take
/// 4. **Act**: Execute action with human-like movement
/// 5. **Loop**: Verify screen changed, retry if not
pub async fn execute_with_verification(
    session: &BrowserSession,
    vision_client: &mut VisionClient,
    instruction: &str,
    max_retries: u32,
) -> Result<()> {
    info!("Starting OODA loop for instruction: {}", instruction);
    
    for attempt in 0..max_retries {
        debug!("OODA Loop iteration {} of {}", attempt + 1, max_retries);
        
        // OBSERVE: Capture current visual state
        let initial_hash = session
            .get_visual_hash()
            .map_err(|e| ChimeraError::ActionFailed(format!("Failed to get visual hash: {}", e)))?;
        
        debug!("Initial visual hash: {}", &initial_hash[..16]);
        
        // ORIENT: Get coordinates from vision service
        let screenshot = session
            .capture_screenshot()
            .map_err(|e| ChimeraError::ActionFailed(format!("Screenshot failed: {}", e)))?;
        
        // Get AX tree for cognitive delay calculation (Hick's Law)
        let tab = session.get_tab()
            .map_err(|e| ChimeraError::ActionFailed(format!("Failed to get tab: {}", e)))?;
        let cortex = crate::cortex::Cortex::new(tab);
        let ax_tree = cortex.snapshot_accessibility_tree()
            .map_err(|e| ChimeraError::ActionFailed(format!("Failed to get AX tree: {}", e)))?;
        
        // Apply cognitive delay based on visual complexity (Hick's Law)
        apply_cognitive_delay(&ax_tree).await;
        
        let (x, y, confidence) = vision_client
            .get_coordinates(screenshot, instruction.to_string())
            .await
            .map_err(|e| ChimeraError::Vision(format!("Vision service error: {}", e)))?;
        
        debug!("Target identified at ({}, {}) with confidence: {:.2}", x, y, confidence);
        
        // DECIDE: Low confidence might indicate wrong target
        if confidence < 0.3 {
            warn!("Low confidence ({:.2}), but proceeding with action", confidence);
        }
        
        // ACT: Execute human-like click
        session
            .click_human_like(x, y, None)
            .await
            .map_err(|e| ChimeraError::ActionFailed(format!("Click failed: {}", e)))?;
        
        // Wait for page to react (animations, navigation, etc.)
        sleep(Duration::from_secs(2)).await;
        
        // LOOP: Verify the screen changed
        let new_hash = session
            .get_visual_hash()
            .map_err(|e| ChimeraError::ActionFailed(format!("Failed to get new visual hash: {}", e)))?;
        
        debug!("New visual hash: {}", &new_hash[..16]);
        
        if initial_hash != new_hash {
            info!("✅ Action verified: Screen state changed (attempt {})", attempt + 1);
            return Ok(()); // Success! The screen changed.
        } else {
            warn!("⚠️  Screen didn't change after click (attempt {}/{})", attempt + 1, max_retries);
            
            if attempt < max_retries - 1 {
                // Wait a bit longer and try again
                // Maybe the page is slow to load, or a popup appeared
                sleep(Duration::from_secs(1)).await;
                
                // Optional: Check for popups or error messages
                // In production, you'd ask the vision service: "Is there a popup blocking the action?"
            }
        }
    }
    
    Err(ChimeraError::ActionFailed(format!(
        "Action failed after {} retries - screen state did not change",
        max_retries
    )))
}

/// Execute a typing action with verification
pub async fn type_with_verification(
    session: &BrowserSession,
    vision_client: &mut VisionClient,
    field_instruction: &str,
    text: &str,
    max_retries: u32,
) -> Result<()> {
    info!("Typing action: '{}' in field matching '{}'", text, field_instruction);
    
    for attempt in 0..max_retries {
        // OBSERVE
        let initial_hash = session
            .get_visual_hash()
            .map_err(|e| ChimeraError::ActionFailed(format!("Failed to get visual hash: {}", e)))?;
        
        // ORIENT: Find the input field
        let screenshot = session
            .capture_screenshot()
            .map_err(|e| ChimeraError::ActionFailed(format!("Screenshot failed: {}", e)))?;
        
        let (x, y, confidence) = vision_client
            .get_coordinates(screenshot, field_instruction.to_string())
            .await
            .map_err(|e| ChimeraError::Vision(format!("Vision service error: {}", e)))?;
        
        // DECIDE & ACT: Click field and type
        session
            .click_human_like(x, y, None)
            .await
            .map_err(|e| ChimeraError::ActionFailed(format!("Click failed: {}", e)))?;
        
        // Small delay before typing
        sleep(Duration::from_millis(100)).await;
        
        // Type with human-like timing
        let tab = session.get_tab()
            .map_err(|e| ChimeraError::ActionFailed(format!("Failed to get tab: {}", e)))?;
        crate::mouse::human_type(&tab, text).await
            .map_err(|e| ChimeraError::ActionFailed(format!("Type failed: {}", e)))?;
        
        // Wait for any updates
        sleep(Duration::from_secs(1)).await;
        
        // VERIFY: Check if field was filled (visual change)
        let new_hash = session
            .get_visual_hash()
            .map_err(|e| ChimeraError::ActionFailed(format!("Failed to get new visual hash: {}", e)))?;
        
        if initial_hash != new_hash {
            info!("✅ Typing verified: Screen state changed");
            return Ok(());
        } else {
            warn!("⚠️  Screen didn't change after typing (attempt {}/{})", attempt + 1, max_retries);
            if attempt < max_retries - 1 {
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
    
        Err(ChimeraError::ActionFailed(format!(
        "Typing failed after {} retries",
        max_retries
    )))
}

/// Apply cognitive delay based on Hick's Law
/// 
/// Hick's Law: Reaction time = b * log2(n + 1)
/// Where n = number of choices (clickable elements)
/// 
/// The Problem: Bots click too fast. Real humans take longer on complex pages.
/// The Fix: Calculate visual complexity and add proportional delay.
pub async fn apply_cognitive_delay(ax_tree: &AxTree) {
    // Count clickable elements (buttons, links, inputs)
    let n = ax_tree.nodes.iter()
        .filter(|node| {
            matches!(node.role.as_str(), 
                "button" | "link" | "textbox" | "checkbox" | "radio" | 
                "menuitem" | "tab" | "option"
            )
        })
        .count();
    
    // Hick's Law: Time = b * log2(n + 1)
    // Base reaction time (200ms) + Processing time per element
    let base_delay_ms = 200u64;
    let processing_per_element = 100.0;
    let delay_ms = base_delay_ms + (processing_per_element * (n as f64 + 1.0).log2()) as u64;
    
    // Add randomness (Human Jitter) - humans are not perfectly consistent
    let mut rng = rand::thread_rng();
    let jitter = rng.gen_range(0..150);
    
    let total_delay = delay_ms + jitter;
    
    info!("🧠 Thinking... (Cognitive Load: {}ms for {} clickable elements)", total_delay, n);
    
    tokio::time::sleep(Duration::from_millis(total_delay)).await;
}
