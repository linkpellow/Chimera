/// Binary Patching - Source-Level Automation Marker Removal
/// 
/// The Apex Strategy: We don't just "mask" properties; we erase them from
/// the browser's DNA before it even launches.
/// 
/// This module implements hardcoded binary patching to remove automation
/// markers from the Chromium binary itself.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

/// Binary patching configuration
pub struct BinaryPatchConfig {
    /// Path to Chromium binary
    pub chromium_path: String,
    
    /// Whether to enable binary patching
    pub enabled: bool,
    
    /// Backup original binary before patching
    pub backup: bool,
}

impl Default for BinaryPatchConfig {
    fn default() -> Self {
        Self {
            chromium_path: "/usr/bin/chromium".to_string(),
            enabled: true,
            backup: true,
        }
    }
}

/// Hex-offset patterns for Chromium automation markers
/// 
/// These are the known byte sequences that Chromium uses internally
/// to mark automation-controlled browsers. We replace them with
/// innocuous strings that don't trigger detection.
pub struct PatchPattern {
    /// Original byte sequence (automation marker)
    pub original: Vec<u8>,
    
    /// Replacement byte sequence (innocuous)
    pub replacement: Vec<u8>,
    
    /// Description of what this patch does
    pub description: &'static str,
}

impl PatchPattern {
    /// Get all known patch patterns for Chromium
    pub fn all_patterns() -> Vec<Self> {
        vec![
            // Pattern 1: navigator.webdriver internal string
            // Chromium stores "webdriver" as an internal property name
            // We replace it with a unique, innocuous identifier
            PatchPattern {
                original: b"webdriver".to_vec(),
                replacement: b"__chimera_internal__".to_vec(),
                description: "Replace 'webdriver' internal string with innocuous identifier",
            },
            
            // Pattern 2: "Headless" in error stack traces
            // V8 isolate metadata contains "Headless" in stack traces
            // We replace it with a generic identifier
            PatchPattern {
                original: b"Headless".to_vec(),
                replacement: b"Standard".to_vec(),
                description: "Replace 'Headless' in V8 isolate metadata",
            },
            
            // Pattern 3: "CDP" (Chrome DevTools Protocol) references
            // Internal CDP markers can leak automation
            // We replace with generic protocol identifier
            PatchPattern {
                original: b"CDP".to_vec(),
                replacement: b"PRO".to_vec(), // "PRO" = Protocol
                description: "Replace 'CDP' references in internal metadata",
            },
            
            // Pattern 4: Automation-controlled flag
            // Binary flag that marks browser as automation-controlled
            // We flip the bit or replace the flag
            PatchPattern {
                original: b"AutomationControlled".to_vec(),
                replacement: b"UserControlled".to_vec(),
                description: "Replace 'AutomationControlled' flag with 'UserControlled'",
            },
        ]
    }
}

/// Binary patcher
pub struct BinaryPatcher {
    config: BinaryPatchConfig,
    patterns: Vec<PatchPattern>,
}

impl BinaryPatcher {
    /// Create a new binary patcher
    pub fn new(config: BinaryPatchConfig) -> Self {
        Self {
            config,
            patterns: PatchPattern::all_patterns(),
        }
    }
    
    /// Patch the Chromium binary
    pub fn patch(&self) -> Result<()> {
        if !self.config.enabled {
            info!("Binary patching disabled, skipping");
            return Ok(());
        }
        
        let chromium_path = Path::new(&self.config.chromium_path);
        
        if !chromium_path.exists() {
            warn!("Chromium binary not found at: {}, skipping binary patching", self.config.chromium_path);
            return Ok(());
        }
        
        info!("Starting binary patching on: {}", self.config.chromium_path);
        
        // Backup original binary if requested
        if self.config.backup {
            let backup_path = format!("{}.backup", self.config.chromium_path);
            fs::copy(&chromium_path, &backup_path)
                .context("Failed to backup Chromium binary")?;
            debug!("Backed up original binary to: {}", backup_path);
        }
        
        // Read binary into memory
        let mut binary_data = fs::read(&chromium_path)
            .context("Failed to read Chromium binary")?;
        
        let original_size = binary_data.len();
        debug!("Read binary: {} bytes", original_size);
        
        // Apply all patches
        let mut total_replacements = 0;
        for pattern in &self.patterns {
            let matches = self.apply_pattern(&mut binary_data, pattern)?;
            if matches > 0 {
                info!("Pattern '{}': {} replacements", pattern.description, matches);
                total_replacements += matches;
            } else {
                debug!("Pattern '{}': No matches found (may already be patched or pattern not present)", pattern.description);
            }
        }
        
        if total_replacements > 0 {
            // Write patched binary back
            fs::write(&chromium_path, &binary_data)
                .context("Failed to write patched binary")?;
            
            info!("Binary patching complete: {} total replacements", total_replacements);
        } else {
            info!("No patches applied (binary may already be patched or patterns not found)");
        }
        
        Ok(())
    }
    
    /// Apply a single patch pattern to binary data
    fn apply_pattern(&self, data: &mut Vec<u8>, pattern: &PatchPattern) -> Result<usize> {
        let mut replacements = 0;
        let pattern_len = pattern.original.len();
        let replacement_len = pattern.replacement.len();
        
        // Find all occurrences of the pattern
        let mut i = 0;
        while i <= data.len().saturating_sub(pattern_len) {
            if data[i..i + pattern_len] == pattern.original[..] {
                // Found a match - replace it
                if replacement_len == pattern_len {
                    // Same length - direct replacement
                    data[i..i + pattern_len].copy_from_slice(&pattern.replacement);
                    replacements += 1;
                    i += pattern_len;
                } else {
                    // Different length - need to resize
                    // For now, we only support same-length replacements
                    // (Different-length would require more complex binary manipulation)
                    warn!("Pattern '{}' has different length replacement, skipping", pattern.description);
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
        
        Ok(replacements)
    }
    
    /// Verify that patches were applied successfully
    pub fn verify(&self) -> Result<bool> {
        let chromium_path = Path::new(&self.config.chromium_path);
        
        if !chromium_path.exists() {
            return Ok(false);
        }
        
        let binary_data = fs::read(&chromium_path)
            .context("Failed to read Chromium binary for verification")?;
        
        // Check if any original patterns still exist
        for pattern in &self.patterns {
            if binary_data.windows(pattern.original.len()).any(|window| window == pattern.original.as_slice()) {
                warn!("Pattern '{}' still found in binary - patch may have failed", pattern.description);
                return Ok(false);
            }
        }
        
        debug!("Binary verification passed - all patterns replaced");
        Ok(true)
    }
}

/// Initialize binary patching (called at build/runtime)
pub fn initialize_binary_patching() -> Result<()> {
    let config = BinaryPatchConfig {
        chromium_path: std::env::var("CHROME_BIN")
            .unwrap_or_else(|_| "/usr/bin/chromium".to_string()),
        enabled: std::env::var("CHIMERA_BINARY_PATCH")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true),
        backup: true,
    };
    
    let patcher = BinaryPatcher::new(config);
    patcher.patch()?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pattern_matching() {
        let mut data = b"webdriver test".to_vec();
        let pattern = PatchPattern {
            original: b"webdriver".to_vec(),
            replacement: b"__chimera__".to_vec(),
            description: "Test pattern",
        };
        
        let patcher = BinaryPatcher::new(BinaryPatchConfig::default());
        let replacements = patcher.apply_pattern(&mut data, &pattern).unwrap();
        
        assert_eq!(replacements, 1);
        assert_eq!(data, b"__chimera__ test");
    }
}
