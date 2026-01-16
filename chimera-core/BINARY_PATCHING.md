# Binary Patching - Hex-Offset Patterns Documentation

## Overview

The Apex Source-Level Strategy: We don't just "mask" properties; we erase them from the browser's DNA before it even launches.

This document details the hex-offset patterns used for hardcoded binary patching of Chromium automation markers.

## Architecture

### Build-Time Patching

Binary patching occurs at runtime (when the binary is first accessed), not during the Rust build process. This is because:

1. **Chromium Binary Location**: The Chromium binary is installed system-wide (e.g., `/usr/bin/chromium`) and is not part of our build artifacts.
2. **Docker Build Context**: In Docker, we patch the binary in the runtime stage after Chromium is installed.
3. **Flexibility**: Runtime patching allows us to patch any Chromium installation without rebuilding.

### Patch Patterns

The following patterns are applied to the Chromium binary:

#### Pattern 1: `navigator.webdriver` Internal String

**Target**: Internal property name that marks automation-controlled browsers

**Original Bytes**: `77 65 62 64 72 69 76 65 72` (ASCII: "webdriver")

**Replacement**: `5F 5F 63 68 69 6D 65 72 61 5F 69 6E 74 65 72 6E 61 6C 5F 5F` (ASCII: "__chimera_internal__")

**Location**: Found in Chromium's internal property registry, typically in the V8 isolate initialization code.

**Impact**: Removes the `navigator.webdriver` property at the binary level, preventing JavaScript-based detection.

**Hex Offset**: Variable (depends on Chromium version)
- Chrome 124: Typically around `0x[VERSION_SPECIFIC]`
- Pattern matching is used instead of fixed offsets for version independence

#### Pattern 2: "Headless" in V8 Isolate Metadata

**Target**: Stack trace metadata that reveals headless mode

**Original Bytes**: `48 65 61 64 6C 65 73 73` (ASCII: "Headless")

**Replacement**: `53 74 61 6E 64 61 72 64` (ASCII: "Standard")

**Location**: V8 isolate error stack traces, internal logging strings.

**Impact**: Prevents detection via error stack traces that contain "Headless" markers.

**Hex Offset**: Variable (V8 isolate initialization)

#### Pattern 3: "CDP" (Chrome DevTools Protocol) References

**Target**: Internal CDP markers that leak automation

**Original Bytes**: `43 44 50` (ASCII: "CDP")

**Replacement**: `50 52 4F` (ASCII: "PRO" - Protocol)

**Location**: Chrome DevTools Protocol internal identifiers.

**Impact**: Removes CDP markers from internal metadata.

**Hex Offset**: Variable (CDP initialization code)

#### Pattern 4: "AutomationControlled" Flag

**Target**: Binary flag marking browser as automation-controlled

**Original Bytes**: `41 75 74 6F 6D 61 74 69 6F 6E 43 6F 6E 74 72 6F 6C 6C 65 64` (ASCII: "AutomationControlled")

**Replacement**: `55 73 65 72 43 6F 6E 74 72 6F 6C 6C 65 64` (ASCII: "UserControlled")

**Location**: Browser feature flag registry.

**Impact**: Changes the automation flag to user-controlled at the binary level.

**Hex Offset**: Variable (feature flag registry)

## Implementation Details

### Pattern Matching vs Fixed Offsets

We use **pattern matching** instead of fixed hex offsets because:

1. **Version Independence**: Chromium versions change binary layouts
2. **Distribution Independence**: Different Linux distributions may compile Chromium differently
3. **Reliability**: Pattern matching is more robust than fixed offsets

### Same-Length Replacement Constraint

Currently, we only support same-length replacements:

- ✅ `"webdriver"` (9 bytes) → `"__chimera__"` (11 bytes) - **NOT SUPPORTED YET**
- ✅ `"Headless"` (8 bytes) → `"Standard"` (8 bytes) - **SUPPORTED**

For different-length replacements, we would need:
- Binary relocation
- Symbol table updates
- More complex binary manipulation

**Future Enhancement**: Implement variable-length patching with binary relocation.

### Backup Strategy

Before patching, the original binary is backed up to `{chromium_path}.backup`. This allows:

1. **Rollback**: Restore original if patching fails
2. **Verification**: Compare patched vs original
3. **Safety**: Never lose the original binary

## Usage

### Automatic Patching

Binary patching is enabled by default and runs automatically when:

1. The Rust binary starts
2. `CHIMERA_BINARY_PATCH=true` (default)
3. Chromium binary is found at `CHROME_BIN` (default: `/usr/bin/chromium`)

### Manual Patching

```rust
use chimera_core::binary_patch::{BinaryPatcher, BinaryPatchConfig};

let config = BinaryPatchConfig {
    chromium_path: "/usr/bin/chromium".to_string(),
    enabled: true,
    backup: true,
};

let patcher = BinaryPatcher::new(config);
patcher.patch()?;
patcher.verify()?;
```

### Verification

After patching, verify that patterns were replaced:

```rust
let verified = patcher.verify()?;
if verified {
    println!("Binary patching verified successfully");
} else {
    println!("Warning: Some patterns may still be present");
}
```

## Docker Integration

In the Dockerfile, binary patching runs in the runtime stage:

```dockerfile
# Runtime stage
FROM debian:bookworm-slim

# Install Chromium
RUN apt-get update && apt-get install -y chromium

# Copy Rust binary (which will patch Chromium on first run)
COPY --from=builder /app/chimera_core /app/chimera_core

# Binary patching happens automatically when chimera_core starts
CMD ["./chimera_core"]
```

## Security Considerations

1. **Binary Integrity**: Patching modifies system binaries - ensure proper permissions
2. **Backup Safety**: Always backup before patching
3. **Version Compatibility**: Test patching on target Chromium version
4. **Rollback Plan**: Keep backup binary for emergency rollback

## Limitations

1. **Version Dependency**: Patterns may not match if Chromium version changes significantly
2. **Same-Length Only**: Currently only supports same-length replacements
3. **Pattern Detection**: If patterns are obfuscated or encrypted, patching will fail silently
4. **Distribution Differences**: Different Linux distributions may have different binary layouts

## Future Enhancements

1. **Variable-Length Patching**: Support different-length replacements with binary relocation
2. **Offset Database**: Maintain a database of known offsets for common Chromium versions
3. **Signature Verification**: Verify binary signatures before/after patching
4. **Multi-Version Support**: Automatically detect Chromium version and apply appropriate patches

---

**Note**: Binary patching is a powerful technique that modifies system binaries. Use with caution and always test in a safe environment first.
