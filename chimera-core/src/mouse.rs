/// Human-like mouse movement using Bezier curves for stealth
/// 
/// This module implements the "Stealth Layer" - making mouse movements
/// indistinguishable from human behavior by using curved paths with
/// random variations.

use rand::Rng;
use std::time::Duration;
use tokio::time::sleep;
use headless_chrome::Tab;
use anyhow::Context;
use tracing::debug;

/// Generate a human-like curved path between two points using Bezier curves
/// 
/// This creates a natural mouse movement path that:
/// - Accelerates at the start
/// - Decelerates at the end  
/// - Curves slightly (not a straight line)
/// - Has random variations (no two paths are identical)
fn generate_human_path(
    start_x: f64,
    start_y: f64,
    end_x: f64,
    end_y: f64,
) -> Vec<(f64, f64)> {
    let mut rng = rand::thread_rng();
    
    // Create control points for the Bezier curve
    // The randomness adds "imperfection" that makes it human-like
    let mid_x = (start_x + end_x) / 2.0;
    let mid_y = (start_y + end_y) / 2.0;
    
    // Random offset for the control point (creates the curve)
    let distance = ((end_x - start_x).powi(2) + (end_y - start_y).powi(2)).sqrt();
    let offset_range = (distance * 0.1).min(100.0).max(20.0); // 10% of distance, clamped
    
    let control_x = mid_x + rng.gen_range(-offset_range..offset_range);
    let control_y = mid_y + rng.gen_range(-offset_range..offset_range);

    // More steps = smoother movement (but slower)
    // Humans typically take 15-30 steps for a mouse movement
    let steps = rng.gen_range(15..30);
    let mut path = Vec::with_capacity(steps + 1);

    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        
        // Quadratic Bezier Curve Formula:
        // B(t) = (1-t)²P₀ + 2(1-t)tP₁ + t²P₂
        // Where P₀ = start, P₁ = control, P₂ = end
        let x = (1.0 - t).powi(2) * start_x 
            + 2.0 * (1.0 - t) * t * control_x 
            + t.powi(2) * end_x;
        let y = (1.0 - t).powi(2) * start_y 
            + 2.0 * (1.0 - t) * t * control_y 
            + t.powi(2) * end_y;
        
        path.push((x, y));
    }
    
    path
}

/// Move mouse along a human-like curved path
async fn move_mouse_human_like(
    tab: &Tab,
    start_x: f64,
    start_y: f64,
    end_x: f64,
    end_y: f64,
) -> anyhow::Result<()> {
    let path = generate_human_path(start_x, start_y, end_x, end_y);
    let mut rng = rand::thread_rng();
    
    debug!("Moving mouse along {} point path from ({:.0}, {:.0}) to ({:.0}, {:.0})", 
           path.len(), start_x, start_y, end_x, end_y);
    
    for (x, y) in path {
        tab.move_mouse(x, y)
            .context("Failed to move mouse")?;
        
        // Micro-sleeps between movements simulate human hand drag
        // Humans don't move at constant speed - we accelerate and decelerate
        let delay_ms = rng.gen_range(5..15);
        sleep(Duration::from_millis(delay_ms)).await;
    }
    
    Ok(())
}

/// Perform a human-like click with natural timing
/// 
/// This simulates:
/// - Curved mouse movement to target
/// - Slight pause before clicking (humans don't click instantly)
/// - Variable hold time (humans don't click for exactly the same duration)
pub async fn human_click(
    tab: &Tab,
    target_x: f64,
    target_y: f64,
    current_x: Option<f64>,
    current_y: Option<f64>,
) -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();
    
    // Get current mouse position (or use provided)
    let (start_x, start_y) = if let (Some(x), Some(y)) = (current_x, current_y) {
        (x, y)
    } else {
        // Default to center of screen if position unknown
        // In production, you'd track mouse position in session state
        (960.0, 540.0)
    };
    
    // Move to target with human-like curve
    move_mouse_human_like(tab, start_x, start_y, target_x, target_y).await?;
    
    // Small random delay before clicking (humans pause slightly)
    let pre_click_delay = rng.gen_range(50..150);
    sleep(Duration::from_millis(pre_click_delay)).await;
    
    // Press mouse button
    tab.click(headless_chrome::types::MouseButton::Left)
        .context("Failed to click")?;
    
    // Variable hold time (humans don't release instantly)
    let hold_time = rng.gen_range(50..150);
    sleep(Duration::from_millis(hold_time)).await;
    
    debug!("Human-like click completed at ({:.0}, {:.0})", target_x, target_y);
    
    Ok(())
}

/// Type text with human-like timing
pub async fn human_type(
    tab: &Tab,
    text: &str,
) -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();
    
    debug!("Typing text with human-like timing: {}", text);
    
    for ch in text.chars() {
        tab.type_str(&ch.to_string())
            .context("Failed to type character")?;
        
        // Humans type at variable speeds (WPM varies)
        // Average is ~40 WPM, but we add randomness
        let delay_ms = rng.gen_range(50..200);
        sleep(Duration::from_millis(delay_ms)).await;
    }
    
    Ok(())
}

/// Perform micro-fidgeting - subtle mouse movements during wait/think states
/// 
/// The Problem: When waiting, the mouse is perfectly still (dead giveaway).
/// Real humans fidget - hands drift, micro-movements, text highlighting.
/// 
/// The Fix: Perform tiny random movements (1-3 pixels) or drift towards center.
pub async fn perform_micro_fidget(tab: &Tab) -> anyhow::Result<()> {
    use rand::Rng;
    use tokio::time::sleep;
    
    let mut rng = rand::thread_rng();
    
    // Get current mouse position (or use center as default)
    // In production, you'd track this in session state
    let current_x = 960.0; // Center of 1920px screen
    let current_y = 540.0; // Center of 1080px screen
    
    // Micro-movement: 1-3 pixels in random direction
    // These are imperceptible to humans but prevent "dead mouse" detection
    let drift_x = rng.gen_range(-3.0..3.0);
    let drift_y = rng.gen_range(-3.0..3.0);
    
    let new_x = (current_x + drift_x).max(0.0).min(1920.0);
    let new_y = (current_y + drift_y).max(0.0).min(1080.0);
    
    // Move mouse slightly (imperceptible to humans, but prevents "dead mouse" detection)
    tab.move_mouse(new_x, new_y)
        .context("Failed to perform micro-fidget")?;
    
    // Small delay before next fidget
    sleep(Duration::from_millis(rng.gen_range(50..200))).await;
    
    Ok(())
}
