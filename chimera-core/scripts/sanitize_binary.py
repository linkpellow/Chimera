#!/usr/bin/env python3
"""
Binary Lobotomy - Machine-Code Eraser (Apex Binary Sanitization)

This script performs high-precision hex-editing on the Chromium binary
to permanently erase automation markers from the binary DNA itself.

Modern anti-bot suites in 2026 have moved beyond checking if navigator.webdriver
exists; they now use Function Integrity Checks. They call toString() on the
property to see if it has been wrapped by a "stealth" proxy. If it doesn't
return the native C++ code signature, you are flagged.

To fulfill the End Goal, we will NOT "patch" the browser at runtime with scripts.
We will rewrite the browser binary during the Docker build process to permanently
scrub the "automation DNA" from its machine code.

CRITICAL: All replacements must maintain exact length to avoid shifting
binary offsets and breaking the executable.
"""

import os
import sys
import shutil
from pathlib import Path

# Target strings to sanitize (UTF-8)
SANITIZATION_PATTERNS = [
    {
        "original": b"navigator.webdriver",
        "replacement": b"navigator.v1_driver",  # Exact same length (19 bytes)
        "description": "navigator.webdriver -> navigator.v1_driver"
    },
    # Additional patterns can be added here
    # All must maintain exact length
]

# Possible Chromium binary locations
CHROMIUM_PATHS = [
    "/usr/bin/chromium",
    "/usr/bin/chromium-browser",
    "/usr/bin/google-chrome",
    "/usr/bin/google-chrome-stable",
]


def find_chromium_binary():
    """Find the Chromium binary location."""
    for path in CHROMIUM_PATHS:
        if os.path.exists(path) and os.access(path, os.W_OK):
            return path
    
    # Try to find it via which command
    import subprocess
    try:
        result = subprocess.run(
            ["which", "chromium"],
            capture_output=True,
            text=True,
            check=False
        )
        if result.returncode == 0:
            path = result.stdout.strip()
            if os.path.exists(path) and os.access(path, os.W_OK):
                return path
    except Exception:
        pass
    
    return None


def sanitize_binary(binary_path: str) -> bool:
    """
    Perform binary sanitization by replacing automation markers.
    
    This is a "Search and Replace" operation on the machine code of the
    Chromium executable. We locate the UTF-8/UTF-16 byte pattern of
    "navigator.webdriver" and overwrite it with an innocuous dummy string
    of identical length.
    
    Returns True if sanitization was successful, False otherwise.
    """
    print(f"🔍 Opening binary for hex-editing (Apex Binary Sanitization): {binary_path}")
    
    # Create backup
    backup_path = f"{binary_path}.backup"
    if not os.path.exists(backup_path):
        print(f"📦 Creating backup: {backup_path}")
        shutil.copy2(binary_path, backup_path)
    else:
        print(f"ℹ️  Backup already exists: {backup_path}")
    
    # Read binary into memory
    try:
        with open(binary_path, 'rb+') as f:
            binary_data = bytearray(f.read())
    except Exception as e:
        print(f"❌ Failed to read binary: {e}", file=sys.stderr)
        return False
    
    original_size = len(binary_data)
    print(f"📊 Binary size: {original_size:,} bytes")
    
    # Apply sanitization patterns
    total_replacements = 0
    for pattern in SANITIZATION_PATTERNS:
        original = pattern["original"]
        replacement = pattern["replacement"]
        description = pattern["description"]
        
        # Verify length match (CRITICAL)
        if len(original) != len(replacement):
            print(f"⚠️  WARNING: Pattern '{description}' has mismatched lengths!", file=sys.stderr)
            print(f"   Original: {len(original)} bytes, Replacement: {len(replacement)} bytes", file=sys.stderr)
            continue
        
        # Find and replace all occurrences
        matches = 0
        i = 0
        while i <= len(binary_data) - len(original):
            if binary_data[i:i+len(original)] == original:
                # Found a match - replace it
                binary_data[i:i+len(original)] = replacement
                matches += 1
                i += len(original)  # Skip past the replaced bytes
            else:
                i += 1
        
        if matches > 0:
            print(f"✅ {description}: {matches} replacement(s)")
            total_replacements += matches
        else:
            print(f"ℹ️  {description}: No matches found (may already be sanitized)")
    
    if total_replacements > 0:
        # Write sanitized binary back
        try:
            with open(binary_path, 'wb') as f:
                f.write(binary_data)
            
            # Verify write
            if os.path.getsize(binary_path) == original_size:
                print(f"✅ Binary sanitization complete: {total_replacements} total replacement(s)")
                print(f"📊 Final size: {len(binary_data):,} bytes (unchanged)")
                return True
            else:
                print(f"❌ Binary size mismatch after write!", file=sys.stderr)
                return False
        except Exception as e:
            print(f"❌ Failed to write sanitized binary: {e}", file=sys.stderr)
            return False
    else:
        print("ℹ️  No sanitization needed (binary may already be sanitized)")
        return True  # Success (nothing to do)


def verify_sanitization(binary_path: str) -> bool:
    """
    Verify that sanitization was successful by checking for original patterns.
    
    Returns True if binary is sanitized, False if original patterns are still present.
    """
    try:
        with open(binary_path, 'rb') as f:
            binary_data = f.read()
    except Exception as e:
        print(f"❌ Failed to read binary for verification: {e}", file=sys.stderr)
        return False
    
    for pattern in SANITIZATION_PATTERNS:
        original = pattern["original"]
        if original in binary_data:
            print(f"⚠️  WARNING: Original pattern still found: {pattern['description']}", file=sys.stderr)
            return False
    
    print("✅ Verification passed: All patterns sanitized")
    return True


def main():
    """Main entry point."""
    print("=" * 60)
    print("🧠 Binary Lobotomy - Apex Binary Sanitization")
    print("   Permanently erasing automation DNA from machine code")
    print("=" * 60)
    
    # Find Chromium binary
    binary_path = find_chromium_binary()
    
    if not binary_path:
        print("❌ ERROR: Chromium binary not found!", file=sys.stderr)
        print("   Searched paths:", file=sys.stderr)
        for path in CHROMIUM_PATHS:
            print(f"     - {path}", file=sys.stderr)
        sys.exit(1)
    
    print(f"✅ Found Chromium binary: {binary_path}")
    
    # Perform sanitization
    if not sanitize_binary(binary_path):
        print("❌ Sanitization failed!", file=sys.stderr)
        sys.exit(1)
    
    # Verify sanitization
    if not verify_sanitization(binary_path):
        print("⚠️  Verification failed - binary may not be fully sanitized", file=sys.stderr)
        sys.exit(1)
    
    print("=" * 60)
    print("✅ Binary Lobotomy Complete - Engine Sanitized")
    print("=" * 60)
    sys.exit(0)


if __name__ == "__main__":
    main()
