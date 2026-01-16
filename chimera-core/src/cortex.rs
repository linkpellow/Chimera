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
use tracing::{debug, info, warn};

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
