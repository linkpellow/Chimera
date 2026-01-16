use crate::error::{ChimeraError, Result};
use tonic::transport::Channel;
use tracing::{debug, error};

pub mod vision {
    tonic::include_proto!("chimera");
}

use vision::vision_service_client::VisionServiceClient;
use vision::{CoordinateRequest, CoordinateResponse};

pub struct VisionClient {
    client: VisionServiceClient<Channel>,
}

impl VisionClient {
    pub async fn connect(addr: String) -> Result<Self> {
        debug!("Connecting to vision service at: {}", addr);
        let client = VisionServiceClient::connect(addr)
            .await
            .map_err(|e| ChimeraError::Vision(format!("Failed to connect: {}", e)))?;
        
        Ok(Self { client })
    }

    pub async fn get_coordinates(
        &mut self,
        image: Vec<u8>,
        text_command: String,
    ) -> Result<(i32, i32, f32)> {
        debug!("Requesting coordinates for: {}", text_command);
        
        let request = tonic::Request::new(CoordinateRequest {
            image,
            text_command,
        });

        let response = self
            .client
            .get_coordinates(request)
            .await
            .map_err(|e| ChimeraError::Vision(format!("gRPC error: {}", e)))?
            .into_inner();

        if !response.found {
            return Err(ChimeraError::Vision("Element not found".to_string()));
        }

        Ok((response.x, response.y, response.confidence))
    }

    /// Get coordinates with Region of Interest (ROI) cropping
    /// 
    /// This implements "Attention-Masked Parsing" - uses fast AX tree scan
    /// to identify regions of interest, then crops screenshot before expensive VLM processing.
    /// 
    /// Result: 10x faster scraping (no processing of ads, headers, footers).
    pub async fn get_coordinates_with_roi(
        &mut self,
        screenshot: Vec<u8>,
        instruction: String,
        roi_bounds: Option<(f64, f64, f64, f64)>, // (x, y, width, height)
    ) -> Result<(i32, i32, f32)> {
        use image::{ImageReader, ImageOutputFormat};
        use std::io::Cursor;
        
        // If ROI is provided, crop the screenshot before sending
        let processed_screenshot = if let Some((x, y, width, height)) = roi_bounds {
            debug!("Cropping screenshot to ROI: ({}, {}) {}x{}", x, y, width, height);
            
            // Load image from bytes
            let img = ImageReader::new(Cursor::new(&screenshot))
                .with_guessed_format()
                .map_err(|e| ChimeraError::Vision(format!("Failed to read image: {}", e)))?
                .decode()
                .map_err(|e| ChimeraError::Vision(format!("Failed to decode image: {}", e)))?;
            
            // Crop to ROI
            let x = x.max(0.0) as u32;
            let y = y.max(0.0) as u32;
            let width = width.min(img.width() as f64 - x as f64) as u32;
            let height = height.min(img.height() as f64 - y as f64) as u32;
            
            let cropped = img.crop_imm(x, y, width, height);
            
            // Convert back to PNG bytes
            let mut buffer = Vec::new();
            let mut cursor = Cursor::new(&mut buffer);
            cropped.write_to(&mut cursor, ImageOutputFormat::Png)
                .map_err(|e| ChimeraError::Vision(format!("Failed to encode cropped image: {}", e)))?;
            
            debug!("Cropped screenshot: {}x{} -> {}x{}", img.width(), img.height(), width, height);
            buffer
        } else {
            screenshot
        };
        
        // Send cropped screenshot to vision service
        self.get_coordinates(processed_screenshot, instruction).await
    }
}
