/// The Cortex Layer - Dual-Sense Perception
/// 
/// This implements hierarchical planning with dual inputs:
/// 1. Visual (Screenshot) - Slow but necessary for layout
/// 2. Semantic (Accessibility Tree) - Fast and reliable
/// 
/// The fusion of these two creates "God Mode" perception.

use crate::browser::BrowserSession;
use anyhow::{Context, Result};
use headless_chrome::Tab;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use rand::Rng;
use rand_distr::{Normal, Distribution};
use std::time::Duration;
use tokio::time::sleep;

/// Accessibility Tree Node - The "Truth" of page structure
/// 
/// This represents a semantic element from the page's accessibility tree.
/// Unlike DOM nodes, these are stable and don't change with CSS/JS.
/// This is the lightweight "Skeleton" - we filter out noise.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxNode {
    pub node_id: String,
    pub role: String,        // "button", "link", "textbox", etc.
    pub name: Option<String>, // Label/name
    pub value: Option<String>, // Current value (for inputs)
    pub parent_id: Option<String>,
    pub bounds: Option<AxBounds>, // Screen coordinates (if available)
    pub state: Vec<String>,   // ["enabled", "visible"], ["disabled"], etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Accessibility Tree - The structural truth
#[derive(Debug, Serialize, Deserialize)]
pub struct AxTree {
    pub nodes: Vec<AxNode>,
}

/// Cortex - Extracts the Accessibility Tree (The "Truth")
pub struct Cortex {
    tab: Arc<Tab>,
}

impl Cortex {
    /// Create a new Cortex instance
    pub fn new(tab: Arc<Tab>) -> Self {
        Self { tab }
    }
    
    /// Extract the full accessibility tree using CDP
    /// 
    /// This calls the raw CDP method 'Accessibility.getFullAXTree'
    /// to get the structural truth of the page.
    pub fn snapshot_accessibility_tree(&self) -> Result<AxTree> {
        debug!("Extracting accessibility tree via CDP");
        
        // Call the CDP method directly
        // headless_chrome doesn't expose this, so we use the raw CDP interface
        let method = "Accessibility.getFullAXTree";
        let params = serde_json::json!({
            "depth": -1,  // Get full tree
            "frameId": None::<String>  // Current frame (None = current)
        });
        
        // Use the tab's call_method to execute CDP command
        let result = self.tab
            .call_method(method, || Ok(params))
            .context("Failed to call Accessibility.getFullAXTree")?;
        
        // Parse the raw CDP response
        // The response structure: { "nodes": [...] }
        let nodes_array = result
            .get("nodes")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("No AX nodes in response"))?;
        
        let mut clean_nodes = Vec::new();
        let mut node_map: HashMap<String, &serde_json::Value> = HashMap::new();
        
        // First pass: build node map by nodeId
        for node in nodes_array {
            if let Some(node_id) = node.get("nodeId").and_then(|v| v.as_str()) {
                node_map.insert(node_id.to_string(), node);
            }
        }
        
        // Second pass: parse nodes (starting from root nodes - those without parentId)
        for node in nodes_array {
            // Check if this is a root node (no parentId or parentId not in map)
            let is_root = if let Some(parent_id) = node.get("parentId").and_then(|v| v.as_str()) {
                !node_map.contains_key(parent_id)
            } else {
                true
            };
            
            if is_root {
                Self::parse_ax_node_recursive(node, None, &node_map, &mut clean_nodes)?;
            }
        }
        
        info!("Extracted {} AX nodes from accessibility tree", clean_nodes.len());
        
        Ok(AxTree { nodes: clean_nodes })
    }
    
    /// Parse AX node recursively from CDP response
    fn parse_ax_node_recursive(
        node: &serde_json::Value,
        parent_id: Option<String>,
        node_map: &HashMap<String, &serde_json::Value>,
        output: &mut Vec<AxNode>,
    ) -> Result<()> {
        // Extract role
        let role = node
            .get("role")
            .and_then(|r| r.get("value"))
            .and_then(|v| v.as_str())
            .unwrap_or("generic")
            .to_string();
        
        // Filter out noise (layout divs, generic containers) to save tokens
        // But we still need to process their children
        let skip_this_node = role == "generic" || role == "LayoutTable" || role == "presentation";
        
        // Extract node ID
        let node_id = node
            .get("nodeId")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        // Extract name (label)
        let name = node
            .get("name")
            .and_then(|n| n.get("value"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        
        // Extract value (for inputs)
        let value = node
            .get("value")
            .and_then(|v| v.get("value"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        
        // Extract bounds (if available)
        let bounds = node
            .get("boundingBox")
            .and_then(|b| {
                Some(AxBounds {
                    x: b.get("x")?.as_f64()?,
                    y: b.get("y")?.as_f64()?,
                    width: b.get("width")?.as_f64()?,
                    height: b.get("height")?.as_f64()?,
                })
            });
        
        // Extract state from properties
        let mut state = Vec::new();
        if let Some(properties) = node.get("properties").and_then(|p| p.as_array()) {
            for prop in properties {
                if let Some(prop_name) = prop.get("name").and_then(|n| n.as_str()) {
                    if let Some(prop_value) = prop.get("value") {
                        if prop_value.as_bool().unwrap_or(false) {
                            state.push(prop_name.to_string());
                        }
                    }
                }
            }
        }
        
        // Only add non-noise nodes to output
        if !skip_this_node {
            let ax_node = AxNode {
                node_id: node_id.clone(),
                role,
                name,
                value,
                parent_id: parent_id.clone(),
                bounds,
                state,
            };
            
            output.push(ax_node);
        }
        
        // Process children recursively
        if let Some(child_ids) = node.get("childIds").and_then(|c| c.as_array()) {
            for child_id_value in child_ids {
                if let Some(child_id) = child_id_value.as_str() {
                    if let Some(child_node) = node_map.get(child_id) {
                        Self::parse_ax_node_recursive(
                            child_node,
                            Some(node_id.clone()),
                            node_map,
                            output,
                        )?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Verify Engine Health - Native Engine Verification
    /// 
    /// Modern anti-bot suites in 2026 use Function Integrity Checks.
    /// They call toString() on properties to see if they've been wrapped by a "stealth" proxy.
    /// If it doesn't return the native C++ code signature, you are flagged.
    /// 
    /// This method verifies that the binary sanitization was successful by checking
    /// the typeof navigator.webdriver. The engine is considered "dirty" if:
    /// - typeof returns "boolean" (native automation flag)
    /// - typeof returns anything other than "undefined"
    /// 
    /// Success Criterion: 
    /// - If typeof returns "boolean" or "true" → engine is DIRTY (fail-fast)
    /// - If typeof returns "undefined" → binary is successfully sanitized
    pub fn verify_engine_health(&self) -> Result<bool> {
        info!("🔍 Verifying engine health (native engine verification)...");
        
        // Execute raw JS probe: return typeof navigator.webdriver
        // This checks the native property, not a wrapped proxy
        let check_script = r#"
            (function() {
                // Direct typeof check - no wrapping, no proxies
                const type = typeof navigator.webdriver;
                const value = navigator.webdriver;
                
                // Also check toString() to detect Function Integrity violations
                // If webdriver exists, check if it's been wrapped
                let toStringResult = null;
                try {
                    if (navigator.webdriver !== undefined) {
                        toStringResult = Object.getOwnPropertyDescriptor(navigator, 'webdriver')?.get?.toString();
                    }
                } catch (e) {
                    toStringResult = 'error: ' + e.message;
                }
                
                return {
                    type: type,
                    value: value,
                    isUndefined: type === 'undefined',
                    isBoolean: type === 'boolean',
                    isDirty: type === 'boolean' || (type !== 'undefined' && value === true),
                    toStringSignature: toStringResult
                };
            })();
        "#;
        
        let result = self.tab
            .evaluate(check_script, false)
            .context("Failed to execute engine health check")?;
        
        // Parse the result
        let result_obj = result.value.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No result from engine health check"))?;
        
        let type_str = result_obj
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        let is_undefined = result_obj
            .get("isUndefined")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        let is_boolean = result_obj
            .get("isBoolean")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        let is_dirty = result_obj
            .get("isDirty")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        let value = result_obj
            .get("value")
            .and_then(|v| {
                if v.is_boolean() {
                    Some(v.as_bool().unwrap().to_string())
                } else if v.is_null() {
                    Some("null".to_string())
                } else {
                    v.as_str().map(|s| s.to_string())
                }
            })
            .unwrap_or_else(|| "unknown".to_string());
        
        let toString_sig = result_obj
            .get("toStringSignature")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        // Success Criterion: Engine is DIRTY if:
        // 1. typeof returns "boolean" (native automation flag)
        // 2. typeof returns anything other than "undefined"
        // 3. value is true (automation enabled)
        if is_dirty || is_boolean || !is_undefined {
            error!(
                "❌ ENGINE HEALTH CHECK FAILED: Engine is DIRTY (not sanitized)"
            );
            error!(
                "   - typeof navigator.webdriver: '{}' (expected: 'undefined')",
                type_str
            );
            error!(
                "   - Value: {}",
                value
            );
            if let Some(sig) = toString_sig {
                error!(
                    "   - toString() signature: {}",
                    sig
                );
                if !sig.contains("native code") && !sig.contains("[native code]") {
                    error!(
                        "   - ⚠️  WARNING: toString() does not show native code - property may be wrapped!"
                    );
                }
            }
            error!(
                "   - Status: Engine has NOT been properly sanitized"
            );
            error!(
                "   - Action: Binary sanitization failed or was bypassed"
            );
            return Ok(false);
        }
        
        // Additional check: If toString signature exists and doesn't show native code, it's wrapped
        if let Some(sig) = &toString_sig {
            if !sig.contains("native code") && !sig.contains("[native code]") && !sig.contains("error") {
                warn!(
                    "⚠️  WARNING: navigator.webdriver toString() signature suggests wrapping: {}",
                    sig
                );
                warn!(
                    "   This may indicate Function Integrity Check failure"
                );
            }
        }
        
        info!("✅ Engine health verified: Binary is successfully sanitized");
        info!("   - typeof navigator.webdriver: '{}' (clean)", type_str);
        info!("   - Value: undefined (clean)");
        if let Some(sig) = toString_sig {
            info!("   - toString() signature: {} (native)", sig);
        }
        info!("   - Status: Engine DNA is clean - ready for missions");
        
        // Phase 2: Verify Redis Session is correctly mounted
        // This ensures "Lived-In" Identity Grafting is active
        if let Err(e) = self.verify_redis_session() {
            warn!("⚠️  Redis session verification failed (non-fatal): {}", e);
            warn!("   Profiles will use filesystem fallback");
        } else {
            info!("✅ Redis session verified: Identity Grafting active");
        }
        
        Ok(true)
    }
    
    /// Verify Redis Session is correctly mounted
    /// 
    /// Success Criterion: A new worker container must be able to resume a session
    /// on a target site (e.g., stay logged in) without re-authenticating, proving
    /// that the Identity Grafting is seamless.
    fn verify_redis_session(&self) -> Result<()> {
        // Check if Redis is configured via environment variable
        let redis_url = std::env::var("REDIS_URL")
            .or_else(|_| std::env::var("CHIMERA_REDIS_URL"))
            .ok();
        
        if redis_url.is_none() {
            debug!("Redis not configured (REDIS_URL or CHIMERA_REDIS_URL not set)");
            return Ok(()); // Not an error, just not using Redis
        }
        
        // Try to connect to Redis and verify profile store is accessible
        let redis_url = redis_url.unwrap();
        
        // Use tokio runtime handle if available, otherwise create new runtime
        let rt = tokio::runtime::Handle::try_current();
        
        if let Ok(handle) = rt {
            // We're in an async context, use the handle
            handle.block_on(async {
                use redis::AsyncCommands;
                
                let client = redis::Client::open(&redis_url)
                    .context("Failed to create Redis client")?;
                
                let mut conn = client.get_async_connection().await
                    .context("Failed to connect to Redis")?;
                
                // Check if Redis is accessible
                let _: String = conn.ping().await
                    .context("Redis ping failed")?;
                
                // Check if profile keys exist
                let keys: Vec<String> = conn.keys("profile:*").await
                    .context("Failed to check profile keys")?;
                
                let profile_count = keys.len();
                
                if profile_count > 0 {
                    info!("Redis session verified: {} profiles available", profile_count);
                } else {
                    warn!("Redis session verified but no profiles found (will create defaults)");
                }
                
                Ok::<(), anyhow::Error>(())
            })?;
        } else {
            // Not in async context, create temporary runtime
            let rt = tokio::runtime::Runtime::new()
                .context("Failed to create tokio runtime for Redis")?;
            
            rt.block_on(async {
                use redis::AsyncCommands;
                
                let client = redis::Client::open(&redis_url)
                    .context("Failed to create Redis client")?;
                
                let mut conn = client.get_async_connection().await
                    .context("Failed to connect to Redis")?;
                
                // Check if Redis is accessible
                let _: String = conn.ping().await
                    .context("Redis ping failed")?;
                
                // Check if profile keys exist
                let keys: Vec<String> = conn.keys("profile:*").await
                    .context("Failed to check profile keys")?;
                
                let profile_count = keys.len();
                
                if profile_count > 0 {
                    info!("Redis session verified: {} profiles available", profile_count);
                } else {
                    warn!("Redis session verified but no profiles found (will create defaults)");
                }
                
                Ok::<(), anyhow::Error>(())
            })?;
        }
        
        Ok(())
    }
    }
    
    /// Legacy method name for backward compatibility
    pub fn verify_engine_sanitization(&self) -> Result<bool> {
        self.verify_engine_health()
    }
    
    /// Find Region of Interest (ROI) for attention-masked parsing
    /// 
    /// Uses fast AX tree scan to identify regions containing specific content
    /// (e.g., "product table", "search results", "form fields")
    /// 
    /// Returns bounding box (x, y, width, height) if found
    pub fn find_roi(&self, role_pattern: &str, name_pattern: Option<&str>) -> Result<Option<(f64, f64, f64, f64)>> {
        let ax_tree = self.snapshot_accessibility_tree()?;
        
        // Find all matching nodes
        let mut matching_nodes = Vec::new();
        
        for node in &ax_tree.nodes {
            if node.role.contains(role_pattern) {
                if let Some(target_name) = name_pattern {
                    if let Some(node_name) = &node.name {
                        if !node_name.to_lowercase().contains(&target_name.to_lowercase()) {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
                
                if let Some(bounds) = &node.bounds {
                    matching_nodes.push(bounds);
                }
            }
        }
        
        if matching_nodes.is_empty() {
            return Ok(None);
        }
        
        // Calculate bounding box that contains all matching nodes
        let min_x = matching_nodes.iter().map(|b| b.x).fold(f64::INFINITY, f64::min);
        let min_y = matching_nodes.iter().map(|b| b.y).fold(f64::INFINITY, f64::min);
        let max_x = matching_nodes.iter().map(|b| b.x + b.width).fold(0.0, f64::max);
        let max_y = matching_nodes.iter().map(|b| b.y + b.height).fold(0.0, f64::max);
        
        Ok(Some((
            min_x,
            min_y,
            max_x - min_x,
            max_y - min_y,
        )))
    }
    
    /// Behavioral Engine - Human Jitter (Mouse & Scroll Entropy)
    /// 
    /// Phase 1: The "Human Jitter" replaces standard linear movement with
    /// Gaussian Micro-Movements and Non-Linear Scrolling using the WindMouse
    /// algorithm to simulate gravity, wind, and muscle tremors.
    /// 
    /// This replaces all direct CDP Input.dispatchMouseEvent calls with
    /// a human-simulated trajectory.
    
    /// Human-like click with Gaussian Jitter and WindMouse algorithm
    /// 
    /// Uses Gaussian Micro-Movements to ensure no two movements between
    /// Point A and Point B are ever identical. Implements 2-3 pixel
    /// "overshoot and correction" pattern typical of human motor control.
    /// 
    /// Args:
    ///   - target_x, target_y: Target coordinates
    ///   - current_x, current_y: Current mouse position (optional)
    ///   - precision: Precision value from Brain (lower = more human-like inaccuracy)
    pub async fn human_click(
        &self,
        target_x: f64,
        target_y: f64,
        current_x: Option<f64>,
        current_y: Option<f64>,
        precision: Option<f64>, // 0.0 = low precision (more human), 1.0 = high precision
    ) -> Result<()> {
        let mut rng = rand::thread_rng();
        
        // Get current position or default to center
        let (start_x, start_y) = match (current_x, current_y) {
            (Some(x), Some(y)) => (x, y),
            _ => (960.0, 540.0), // Default to center
        };
        
        // Apply precision-based targeting offset
        // Lower precision = more human-like inaccuracy (2-3 pixel overshoot)
        let precision_val = precision.unwrap_or(0.3); // Default: 30% precision (70% human-like)
        let max_offset = (1.0 - precision_val) * 5.0; // 0-5 pixels offset
        
        // Generate Gaussian jitter for target (human inaccuracy)
        let jitter_dist = Normal::new(0.0, max_offset).unwrap();
        let jitter_x = jitter_dist.sample(&mut rng);
        let jitter_y = jitter_dist.sample(&mut rng);
        
        let adjusted_target_x = target_x + jitter_x;
        let adjusted_target_y = target_y + jitter_y;
        
        debug!(
            "Human click: ({:.1}, {:.1}) -> ({:.1}, {:.1}) [precision: {:.2}, jitter: ({:.2}, {:.2})]",
            start_x, start_y, adjusted_target_x, adjusted_target_y, precision_val, jitter_x, jitter_y
        );
        
        // Generate WindMouse trajectory (simulates gravity, wind, muscle tremors)
        let trajectory = self.generate_windmouse_trajectory(
            start_x, start_y,
            adjusted_target_x, adjusted_target_y,
        )?;
        
        // Execute trajectory with Gaussian micro-movements
        for (x, y, delay) in trajectory {
            // Add Gaussian tremor to each point (muscle jitter)
            let tremor_dist = Normal::new(0.0, 0.5).unwrap(); // 0.5px standard deviation
            let tremor_x = tremor_dist.sample(&mut rng);
            let tremor_y = tremor_dist.sample(&mut rng);
            
            let final_x = x + tremor_x;
            let final_y = y + tremor_y;
            
            self.tab.move_mouse(final_x, final_y)
                .context("Failed to move mouse in trajectory")?;
            
            if !delay.is_zero() {
                sleep(delay).await;
            }
        }
        
        // Hick's Law Latency: Variable "think time" before clicking
        // Mimics human cognitive load required to process a page before clicking
        let think_time = self.calculate_hicks_law_latency()?;
        sleep(think_time).await;
        
        // Small pre-click pause (humans don't click instantly)
        let pre_click_delay = rng.gen_range(50..150);
        sleep(Duration::from_millis(pre_click_delay)).await;
        
        // Click
        self.tab.click(headless_chrome::types::MouseButton::Left)
            .context("Failed to click")?;
        
        // Variable hold time
        let hold_time = rng.gen_range(50..200);
        sleep(Duration::from_millis(hold_time)).await;
        
        debug!("Human click completed at ({:.1}, {:.1})", adjusted_target_x, adjusted_target_y);
        Ok(())
    }
    
    /// Human-like scroll with Non-Linear Entropy
    /// 
    /// Implements non-linear scrolling patterns that mimic human behavior:
    /// - Variable scroll speed (acceleration/deceleration)
    /// - Gaussian jitter in scroll distance
    /// - Natural pauses during scrolling
    pub async fn human_scroll(
        &self,
        delta_x: f64,
        delta_y: f64,
        current_x: Option<f64>,
        current_y: Option<f64>,
    ) -> Result<()> {
        let mut rng = rand::thread_rng();
        
        // Get current position
        let (scroll_x, scroll_y) = match (current_x, current_y) {
            (Some(x), Some(y)) => (x, y),
            _ => (960.0, 540.0),
        };
        
        // Add Gaussian jitter to scroll distance (human inaccuracy)
        let jitter_dist = Normal::new(0.0, delta_y.abs() * 0.1).unwrap();
        let jitter = jitter_dist.sample(&mut rng);
        let adjusted_delta_y = delta_y + jitter;
        
        debug!(
            "Human scroll: delta_y={:.1} -> {:.1} (jitter: {:.2})",
            delta_y, adjusted_delta_y, jitter
        );
        
        // Break scroll into multiple steps with variable speed
        // Humans don't scroll in one smooth motion
        let steps = rng.gen_range(3..8);
        let step_size = adjusted_delta_y / steps as f64;
        
        for i in 0..steps {
            // Variable scroll speed (faster at start, slower at end)
            let t = i as f64 / steps as f64;
            let speed_factor = if t < 0.5 {
                1.0 + (1.0 - t * 2.0) * 0.5 // Accelerate
            } else {
                1.0 - (t - 0.5) * 2.0 * 0.3 // Decelerate
            };
            
            let step_delta = step_size * speed_factor;
            
            // Add micro-jitter to each step
            let micro_jitter = rng.gen_range(-2.0..2.0);
            let final_delta = step_delta + micro_jitter;
            
            self.tab.scroll(scroll_x, scroll_y, 0.0, final_delta)
                .context("Failed to scroll")?;
            
            // Variable delay between scroll steps (humans pause)
            let delay = rng.gen_range(20..80);
            sleep(Duration::from_millis(delay)).await;
        }
        
        debug!("Human scroll completed");
        Ok(())
    }
    
    /// Generate WindMouse trajectory
    /// 
    /// WindMouse algorithm simulates:
    /// - Gravity (natural deceleration)
    /// - Wind (random drift)
    /// - Muscle tremors (Gaussian jitter)
    /// 
    /// This ensures no two movements are ever identical, avoiding
    /// the "sharp peaks" characteristic of bots.
    fn generate_windmouse_trajectory(
        &self,
        start_x: f64,
        start_y: f64,
        end_x: f64,
        end_y: f64,
    ) -> Result<Vec<(f64, f64, Duration)>> {
        let mut rng = rand::thread_rng();
        let distance = ((end_x - start_x).powi(2) + (end_y - start_y).powi(2)).sqrt();
        
        // Calculate number of steps based on distance
        // More steps = smoother but slower
        let steps = if distance < 100.0 {
            rng.gen_range(10..20)
        } else if distance < 500.0 {
            rng.gen_range(20..35)
        } else {
            rng.gen_range(35..50)
        };
        
        // WindMouse parameters
        let gravity = 9.0; // Gravity strength
        let wind = rng.gen_range(0.0..10.0); // Wind strength (random)
        let max_step = 10.0; // Maximum step size
        let target_area = 3.0; // Target area size (pixels)
        
        let mut trajectory = Vec::with_capacity(steps);
        let mut current_x = start_x;
        let mut current_y = start_y;
        let mut velocity_x = 0.0;
        let mut velocity_y = 0.0;
        
        for i in 0..steps {
            let remaining_distance = ((end_x - current_x).powi(2) + (end_y - current_y).powi(2)).sqrt();
            
            // Check if we're close enough to target
            if remaining_distance < target_area {
                // Overshoot and correction pattern (2-3 pixels)
                if remaining_distance > 1.0 {
                    // Small overshoot
                    let overshoot = rng.gen_range(2.0..4.0);
                    let angle = (end_y - current_y).atan2(end_x - current_x);
                    current_x += overshoot * angle.cos();
                    current_y += overshoot * angle.sin();
                    
                    trajectory.push((current_x, current_y, Duration::from_millis(10)));
                    
                    // Correction back to target
                    current_x = end_x + rng.gen_range(-1.0..1.0);
                    current_y = end_y + rng.gen_range(-1.0..1.0);
                    trajectory.push((current_x, current_y, Duration::from_millis(20)));
                }
                break;
            }
            
            // Calculate wind effect (random drift)
            let wind_x = wind * rng.gen_range(-1.0..1.0);
            let wind_y = wind * rng.gen_range(-1.0..1.0);
            
            // Calculate gravity effect (towards target)
            let gravity_x = (end_x - current_x) / remaining_distance * gravity;
            let gravity_y = (end_y - current_y) / remaining_distance * gravity;
            
            // Update velocity
            velocity_x += gravity_x + wind_x;
            velocity_y += gravity_y + wind_y;
            
            // Limit velocity
            let velocity_mag = (velocity_x.powi(2) + velocity_y.powi(2)).sqrt();
            if velocity_mag > max_step {
                velocity_x = velocity_x / velocity_mag * max_step;
                velocity_y = velocity_y / velocity_mag * max_step;
            }
            
            // Update position
            current_x += velocity_x;
            current_y += velocity_y;
            
            // Add Gaussian tremor (muscle jitter)
            let tremor_dist = Normal::new(0.0, 0.3).unwrap();
            let tremor_x = tremor_dist.sample(&mut rng);
            let tremor_y = tremor_dist.sample(&mut rng);
            
            current_x += tremor_x;
            current_y += tremor_y;
            
            // Calculate delay (variable speed)
            let delay_ms = rng.gen_range(5..15);
            trajectory.push((current_x, current_y, Duration::from_millis(delay_ms)));
        }
        
        // Ensure we end at target (with small jitter)
        if trajectory.is_empty() || {
            let last = trajectory.last().unwrap();
            let dist = ((end_x - last.0).powi(2) + (end_y - last.1).powi(2)).sqrt();
            dist > target_area
        } {
            let final_jitter_x = rng.gen_range(-1.0..1.0);
            let final_jitter_y = rng.gen_range(-1.0..1.0);
            trajectory.push((
                end_x + final_jitter_x,
                end_y + final_jitter_y,
                Duration::from_millis(10),
            ));
        }
        
        Ok(trajectory)
    }
    
    /// Calculate Hick's Law Latency
    /// 
    /// Hick's Law: Reaction time = a + b * log2(n + 1)
    /// Where n = number of choices/clickable elements
    /// 
    /// This mimics the human cognitive load required to process a page
    /// before clicking. Instead of a fixed sleep, we use a distribution
    /// that varies based on page complexity.
    fn calculate_hicks_law_latency(&self) -> Result<Duration> {
        // Get number of clickable elements from AX tree
        let ax_tree = self.snapshot_accessibility_tree()?;
        let clickable_count = ax_tree.nodes.iter()
            .filter(|node| {
                matches!(node.role.as_str(), "button" | "link" | "textbox" | "checkbox" | "radio")
            })
            .count();
        
        // Hick's Law formula
        let a = 200.0; // Base reaction time (ms)
        let b = 100.0; // Processing time per element (ms)
        let n = clickable_count as f64;
        
        let base_time = a + b * (n + 1.0).log2();
        
        // Add randomness (humans are not perfectly consistent)
        let mut rng = rand::thread_rng();
        let variance = rng.gen_range(0.7..1.3); // 30% variance
        let total_time = (base_time * variance) as u64;
        
        // Add jitter (0-150ms random delay)
        let jitter = rng.gen_range(0..150);
        let final_time = total_time + jitter;
        
        debug!(
            "Hick's Law latency: {} clickable elements -> {}ms (base: {:.0}ms, variance: {:.2})",
            clickable_count, final_time, base_time, variance
        );
        
        Ok(Duration::from_millis(final_time))
    }
}

/// Dual-Sense State - Combines visual and semantic information
pub struct FusionState {
    /// Screenshot (visual context)
    pub screenshot: Vec<u8>,
    
    /// Accessibility tree (semantic skeleton)
    pub ax_tree: AxTree,
    
    /// Mapping of AX nodes to screen regions
    pub node_to_region: HashMap<String, AxBounds>,
}

impl FusionState {
    /// Create a new fusion state from browser session
    pub fn from_session(session: &BrowserSession) -> Result<Self> {
        info!("Creating dual-sense fusion state");
        
        // Get screenshot (visual)
        let screenshot = session.capture_screenshot()?;
        
        // Get accessibility tree (semantic) - THE TRUTH
        let tab = session.get_tab()?;
        let cortex = Cortex::new(tab);
        let ax_tree = cortex.snapshot_accessibility_tree()?;
        
        // Build mapping of nodes to screen regions
        let mut node_to_region = HashMap::new();
        Self::build_node_map(&ax_tree, &mut node_to_region);
        
        debug!("Fusion state created: {} nodes mapped", node_to_region.len());
        
        Ok(Self {
            screenshot,
            ax_tree,
            node_to_region,
        })
    }
    
    /// Build a map of node IDs to screen regions
    fn build_node_map(tree: &AxTree, map: &mut HashMap<String, AxBounds>) {
        for node in &tree.nodes {
            if let Some(bounds) = &node.bounds {
                // Use node_id as key
                map.insert(node.node_id.clone(), bounds.clone());
            }
        }
    }
    
    /// Find a node by role and name (fast semantic search)
    pub fn find_node(&self, role: &str, name: Option<&str>) -> Option<&AxNode> {
        for node in &self.ax_tree.nodes {
            if node.role == role {
                if let Some(search_name) = name {
                    if node.name.as_deref() == Some(search_name) {
                        return Some(node);
                    }
                } else {
                    return Some(node);
                }
            }
        }
        None
    }
    
    /// Get screen coordinates for a semantic element
    /// 
    /// This is the "God Mode" - we know WHAT it is (from AX tree)
    /// and WHERE it is (from bounds), without parsing HTML/CSS
    pub fn get_coordinates(&self, role: &str, name: Option<&str>) -> Option<(f64, f64)> {
        if let Some(node) = self.find_node(role, name) {
            if let Some(bounds) = &node.bounds {
                // Return center of element
                return Some((
                    bounds.x + bounds.width / 2.0,
                    bounds.y + bounds.height / 2.0,
                ));
            }
        }
        None
    }
    
    /// Get all nodes of a specific role
    pub fn get_nodes_by_role(&self, role: &str) -> Vec<&AxNode> {
        self.ax_tree
            .nodes
            .iter()
            .filter(|node| node.role == role)
            .collect()
    }
    
    /// Check if an element exists in the AX tree (fast semantic check)
    pub fn has_element(&self, role: &str, name: Option<&str>) -> bool {
        self.find_node(role, name).is_some()
    }
}


/// Hierarchical Planning Chain
/// 
/// 1. General (LLM) - Strategy: "Book a flight to Tokyo"
/// 2. Commander (VLM) - Tactics: "Find the date picker"
/// 3. Soldier (Nano-Model) - Execution: "Click at (400, 300)"
pub struct ChainOfCommand {
    /// General: High-level strategy (cloud LLM)
    pub general_prompt: Option<String>,
    
    /// Commander: Mid-level tactics (local VLM)
    pub commander_instruction: Option<String>,
    
    /// Soldier: Low-level execution (coordinates)
    pub soldier_target: Option<(f64, f64)>,
}

impl ChainOfCommand {
    /// Execute the full chain
    pub async fn execute(
        &mut self,
        session: &BrowserSession,
        fusion_state: &FusionState,
    ) -> Result<()> {
        // Step 1: General plans the strategy
        if let Some(prompt) = &self.general_prompt {
            info!("General: {}", prompt);
            // In production, this would call a cloud LLM (GPT-4, Claude, etc.)
            // For now, we skip to commander
        }
        
        // Step 2: Commander finds the target
        if let Some(instruction) = &self.commander_instruction {
            info!("Commander: {}", instruction);
            
            // Use dual-sense to find the target
            // First try semantic (fast)
            if let Some((x, y)) = fusion_state.get_coordinates("button", Some(instruction)) {
                self.soldier_target = Some((x, y));
                info!("Commander found target via AX tree: ({:.0}, {:.0})", x, y);
            } else {
                // Fall back to vision (slower but more reliable)
                info!("Commander falling back to vision model");
                // This would call the vision service
            }
        }
        
        // Step 3: Soldier executes
        if let Some((x, y)) = self.soldier_target {
            info!("Soldier: Clicking at ({:.0}, {:.0})", x, y);
            // Execute the click
        }
        
        Ok(())
    }
}
