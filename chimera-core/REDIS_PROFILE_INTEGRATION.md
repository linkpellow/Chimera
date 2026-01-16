# Redis Profile Integration - "Lived-In" Identity Grafting

## Overview

To reach the End Goal, we must move beyond a "clean" browser and create an "authentic" one. The "Lived-In" Identity Grafting system allows Rust workers to pull "Synthetic Profiles" (cookies, history, and cache) from Redis so that every worker arrives at the target site with a "history," bypassing "New User" suspicion algorithms.

## Architecture

### Profile Mounting

Workers do not launch with a fresh profile. Instead, they:

1. **Pull from Redis**: Fetch a synthetic profile from the Redis swarm
2. **Mount Profile**: Launch browser with the profile's directory (cookies, cache, history)
3. **Update Profile**: After use, update the profile in Redis with new history/cookies
4. **Share Across Swarm**: Other workers can use the same "lived-in" profile

### Profile Structure

Each synthetic profile contains:
- **Visit History**: YouTube, Reddit, CNN, etc. (realistic browsing patterns)
- **Cache**: 500MB+ of cached content from top-tier sites
- **Cookies**: Authentic logged-in sessions for unrelated sites
- **Local Storage**: Persistent data that real users accumulate
- **Browser Fingerprint**: Consistent hardware/software identity

## Implementation Status

### Current State

- ✅ Profile structure defined (`SyntheticProfile`)
- ✅ Filesystem storage implemented
- ✅ Profile rotation logic
- ⚠️ Redis integration structure prepared (not yet implemented)

### To Enable Redis Integration

1. **Add Redis Dependency** to `Cargo.toml`:
```toml
[dependencies]
redis = { version = "0.24", features = ["tokio-comp"] }
```

2. **Implement Redis Methods** in `identity_grafting.rs`:
   - `load_profiles_from_redis()` - Fetch profiles from Redis
   - `save_profile_to_redis()` - Update profile after use
   - `mount_profile_from_redis()` - Mount profile directory

3. **Configure Redis URL**:
```rust
let grafting = IdentityGrafting::new(
    "/path/to/profiles",
    Some("redis://redis.railway.internal:6379".to_string())
)?;
```

## Usage

### With Redis (Swarm Sharing)

```rust
use chimera_core::identity_grafting::IdentityGrafting;

// Initialize with Redis URL
let grafting = IdentityGrafting::new(
    "/tmp/profiles",  // Fallback directory
    Some("redis://redis.railway.internal:6379".to_string())  // Redis URL
)?;

// Get a profile (pulls from Redis if available)
let profile = grafting.get_profile(None)?;  // Rotates automatically

// Launch browser with grafted profile
let browser = Browser::new(LaunchOptions {
    user_data_dir: Some(profile.profile_dir),
    ..Default::default()
})?;
```

### Without Redis (Local Development)

```rust
// Initialize without Redis (filesystem only)
let grafting = IdentityGrafting::new(
    "/tmp/profiles",
    None  // No Redis
)?;

// Works the same way, but profiles are local only
let profile = grafting.get_profile(None)?;
```

## Redis Key Structure

```
profile:{profile_id} -> JSON serialized SyntheticProfile
profile:windows_chrome_124 -> { "id": "windows_chrome_124", ... }
profile:mac_safari_17 -> { "id": "mac_safari_17", ... }
```

## Benefits

1. **Swarm Sharing**: All workers share the same "lived-in" profiles
2. **History Accumulation**: Profiles get richer over time (more cookies, cache, history)
3. **Bypass "New User" Detection**: Every worker arrives with authentic history
4. **Consistency**: Same profile used across multiple workers maintains consistency

## Next Steps

1. Add `redis` crate to `Cargo.toml`
2. Implement `load_profiles_from_redis()`
3. Implement `save_profile_to_redis()`
4. Test with Railway Redis service
5. Deploy and verify "lived-in" profiles work across swarm

---

**Status**: Structure prepared, Redis integration pending  
**Priority**: Medium (filesystem works for now, Redis needed for swarm)
