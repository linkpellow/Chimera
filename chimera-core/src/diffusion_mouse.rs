/// Diffusion-Based Human Movement Engine
/// 
/// This replaces GAN-based movement with Flow-Matching Diffusion Transformers.
/// Unlike GANs (which suffer from mode collapse), Diffusion models generate
/// statistically unique trajectories every single time, even for the same target.
/// 
/// The model learns the "noise" of the human hand, not just the path.
/// 
/// Architecture: Pre-trained Diffusion Transformer (DiT) exported to ONNX,
/// running natively in Rust via ONNX Runtime for micro-second latency.

use anyhow::{Context, Result};
use ort::{GraphOptimizationLevel, Session, SessionBuilder, Value};
use ndarray::{Array, Array2, Array3, Axis};
use rand::Rng;
use std::path::Path;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};
use serde::{Deserialize, Serialize};

/// Point in 2D space
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    
    pub fn distance_to(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

/// Diffusion Mouse - Generates human-like trajectories using Diffusion models
pub struct DiffusionMouse {
    /// ONNX Runtime session for the Diffusion model
    #[cfg(feature = "onnx")]
    model: Option<Session>,
    
    /// Model path (if using pre-trained model)
    model_path: Option<String>,
    
    /// Fallback to physics-based movement if model not available
    use_fallback: bool,
}

impl DiffusionMouse {
    /// Create a new Diffusion Mouse
    /// 
    /// If model_path is provided and valid, loads the ONNX model.
    /// Otherwise, falls back to physics-based movement.
    pub fn new(model_path: Option<&str>) -> Result<Self> {
        let mut mouse = Self {
            #[cfg(feature = "onnx")]
            model: None,
            model_path: model_path.map(|s| s.to_string()),
            use_fallback: true,
        };
        
        // Try to load the model
        #[cfg(feature = "onnx")]
        {
            if let Some(path) = &mouse.model_path {
                if Path::new(path).exists() {
                    match mouse.load_model(path) {
                        Ok(_) => {
                            info!("Diffusion model loaded successfully from {}", path);
                            mouse.use_fallback = false;
                        }
                        Err(e) => {
                            warn!("Failed to load Diffusion model: {}. Using fallback.", e);
                            mouse.use_fallback = true;
                        }
                    }
                } else {
                    warn!("Model path does not exist: {}. Using fallback.", path);
                }
            } else {
                info!("No model path provided. Using physics-based fallback.");
            }
        }
        
        #[cfg(not(feature = "onnx"))]
        {
            info!("ONNX feature not enabled. Using physics-based fallback.");
        }
        
        Ok(mouse)
    }
    
    /// Load ONNX model from file
    #[cfg(feature = "onnx")]
    fn load_model(&mut self, path: &str) -> Result<()> {
        debug!("Loading Diffusion model from: {}", path);
        
        let session = SessionBuilder::new()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(1)?  // Single thread for deterministic results
            .commit_from_file(path)
            .context("Failed to load ONNX model file")?;
        
        self.model = Some(session);
        Ok(())
    }
    
    /// Generate a human-like trajectory using Diffusion model
    /// 
    /// The Diffusion model takes:
    /// - Start point
    /// - End point  
    /// - Random Gaussian noise (the "seed")
    /// 
    /// And generates a unique trajectory that includes:
    /// - Natural overshoot
    /// - Micro-tremors
    /// - Variable acceleration
    /// - Human-like corrections
    pub fn generate_trajectory(
        &self,
        start: Point,
        end: Point,
        target_size: f64,
    ) -> Vec<(Point, Duration)> {
        #[cfg(feature = "onnx")]
        {
            if let Some(ref model) = self.model {
                // Use Diffusion model
                return self.generate_diffusion_trajectory(model, start, end, target_size);
            }
        }
        
        // Fallback to physics-based movement
        warn!("Using fallback physics-based trajectory (Diffusion model not available)");
        self.generate_fallback_trajectory(start, end, target_size)
    }
    
    /// Generate trajectory using Diffusion model
    #[cfg(feature = "onnx")]
    fn generate_diffusion_trajectory(
        &self,
        model: &Session,
        start: Point,
        end: Point,
        target_size: f64,
    ) -> Vec<(Point, Duration)> {
        debug!("Generating Diffusion trajectory from ({:.0}, {:.0}) to ({:.0}, {:.0})", 
               start.x, start.y, end.x, end.y);
        
        let distance = start.distance_to(&end);
        
        // Number of steps (more steps = smoother, but slower)
        // Diffusion models typically use 50-100 steps
        let num_steps = if distance < 100.0 {
            50
        } else if distance < 500.0 {
            75
        } else {
            100
        };
        
        // Generate random Gaussian noise (the "seed" for Diffusion)
        // This is what makes each trajectory unique
        let mut rng = rand::thread_rng();
        let noise_shape = (1, num_steps, 2); // [batch, steps, x/y]
        use rand_distr::Distribution;
        let normal = rand_distr::StandardNormal;
        let noise: Vec<f32> = (0..noise_shape.0 * noise_shape.1 * noise_shape.2)
            .map(|_| normal.sample(&mut rng) as f32)
            .collect();
        
        // Prepare inputs for the model
        // Input format: [start_x, start_y, end_x, end_y, target_size, noise...]
        let mut input_vec = vec![
            start.x as f32,
            start.y as f32,
            end.x as f32,
            end.y as f32,
            target_size as f32,
        ];
        input_vec.extend_from_slice(&noise);
        
        // Create input tensor
        // Shape: [1, num_steps + 5] (5 for start/end/target, num_steps for noise)
        let input_array = Array::from_shape_vec((1, num_steps + 5), input_vec)
            .expect("Failed to create input array");
        
        // Run the model
        let inputs = vec![Value::from_array(model.allocator(), &input_array)
            .context("Failed to create input tensor")
            .unwrap()];
        
        let outputs = model.run(inputs)
            .context("Failed to run Diffusion model")
            .unwrap();
        
        // Extract trajectory from output
        // Output shape: [1, num_steps, 2] (x, y coordinates for each step)
        let output_array = outputs[0]
            .try_extract::<f32>()
            .context("Failed to extract output")
            .unwrap();
        
        let trajectory = self.parse_trajectory(&output_array, start, end, distance);
        
        debug!("Generated {} point trajectory via Diffusion", trajectory.len());
        trajectory
    }
    
    /// Parse model output into trajectory points with timing
    fn parse_trajectory(
        &self,
        output: &Array3<f32>,
        start: Point,
        end: Point,
        distance: f64,
    ) -> Vec<(Point, Duration)> {
        let mut trajectory = Vec::new();
        let num_steps = output.shape()[1];
        
        // Calculate total movement time using Fitts's Law
        let a = 100.0; // Base time (ms)
        let b = 200.0; // Difficulty coefficient
        let target_size = 50.0; // Default target size
        let movement_time_ms = a + b * (distance / target_size + 1.0).log2();
        
        // Add randomness for human-like variation
        let mut rng = rand::thread_rng();
        let time_variance = rng.gen_range(0.8..1.2);
        let total_time_ms = (movement_time_ms * time_variance) as u64;
        
        for i in 0..num_steps {
            let x = output[[0, i, 0]] as f64;
            let y = output[[0, i, 1]] as f64;
            
            // Calculate timing (non-linear acceleration/deceleration)
            let t = i as f64 / num_steps as f64;
            let time_at_point = if t < 0.5 {
                // Acceleration phase
                total_time_ms as f64 * (t * 2.0).powi(2) * 0.4
            } else {
                // Deceleration phase
                let decel_t = (t - 0.5) * 2.0;
                total_time_ms as f64 * (0.4 + 0.6 * (1.0 - (1.0 - decel_t).powi(3)))
            };
            
            let delay = Duration::from_millis(time_at_point as u64);
            
            trajectory.push((Point::new(x, y), delay));
        }
        
        trajectory
    }
    
    /// Fallback: Physics-based trajectory (used when model not available)
    fn generate_fallback_trajectory(
        &self,
        start: Point,
        end: Point,
        target_size: f64,
    ) -> Vec<(Point, Duration)> {
        // Use the existing neuromotor physics as fallback
        // This is the code from ghost_mouse.rs
        use crate::ghost_mouse::NeuromotorMouse;
        
        let mut neuromotor = NeuromotorMouse::new(start.x, start.y);
        let path = neuromotor.generate_human_path(end.x, end.y, target_size);
        
        path.into_iter()
            .map(|(x, y, delay)| (Point::new(x, y), delay))
            .collect()
    }
    
    /// Check if Diffusion model is available
    pub fn has_model(&self) -> bool {
        #[cfg(feature = "onnx")]
        {
            self.model.is_some()
        }
        #[cfg(not(feature = "onnx"))]
        {
            false
        }
    }
    
    /// Get current position (for tracking)
    pub fn current_position(&self) -> Option<Point> {
        // This would be tracked in the session state
        None
    }
}

/// Execute a Diffusion-based click
pub async fn diffusion_click(
    tab: &headless_chrome::Tab,
    mouse: &DiffusionMouse,
    target_x: f64,
    target_y: f64,
    target_size: f64,
    current_pos: Option<Point>,
) -> Result<()> {
    use tokio::time::sleep;
    
    let start = current_pos.unwrap_or(Point::new(960.0, 540.0));
    let end = Point::new(target_x, target_y);
    
    debug!("Starting Diffusion-based click to ({:.0}, {:.0})", target_x, target_y);
    
    // Generate trajectory
    let trajectory = mouse.generate_trajectory(start, end, target_size);
    
    // Move along the trajectory
    for (point, delay) in trajectory {
        tab.move_mouse(point.x, point.y)
            .map_err(|e| anyhow::anyhow!("Failed to move mouse: {}", e))?;
        
        if !delay.is_zero() {
            sleep(delay).await;
        }
    }
    
    // Small pause before clicking (humans don't click instantly)
    let mut rng = rand::thread_rng();
    let pre_click_delay = rng.gen_range(50..150);
    sleep(Duration::from_millis(pre_click_delay)).await;
    
    // Press mouse button
    tab.click(headless_chrome::types::MouseButton::Left)
        .map_err(|e| anyhow::anyhow!("Failed to click: {}", e))?;
    
    // Variable hold time
    let hold_time = rng.gen_range(50..200);
    sleep(Duration::from_millis(hold_time)).await;
    
    debug!("Diffusion-based click completed");
    
    Ok(())
}

/// Generate Gaussian noise for Diffusion model
fn generate_gaussian_noise(shape: (usize, usize)) -> Vec<f32> {
    use rand_distr::{Distribution, StandardNormal};
    let mut rng = rand::thread_rng();
    let normal = StandardNormal;
    
    (0..shape.0 * shape.1)
        .map(|_| normal.sample(&mut rng) as f32)
        .collect()
}
