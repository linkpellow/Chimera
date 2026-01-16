# Mission Alpha Execution Guide

## Quick Start

This guide provides step-by-step instructions for executing the Mission Alpha Run and capturing results.

## Prerequisites

1. **Railway Deployment**: Services deployed and running
2. **Worker Access**: Ability to start worker sessions
3. **Screenshot Capture**: Method to capture browser screenshots
4. **Log Access**: Access to Railway logs

## Execution Sequence

### Step 1: Pre-Flight Checks

Verify all systems are operational:

```bash
# Check Brain service
curl http://brainscraper.railway.internal:50052/health

# Check Body service (via gRPC or health endpoint)
# Verify all 5/5 instances report "Sanitized and Ready"

# Check Redis connection
redis-cli -h [REDIS_HOST] ping
```

**Expected Output**:
- Brain: Healthy
- Body: 5/5 instances ready
- Redis: PONG

### Step 2: CreepJS Validation

**Objective**: Verify 100% "Human" trust score

**Execution**:
1. Start a worker session
2. Navigate to: `https://abrahamjuliot.github.io/creepjs/`
3. Wait for page to fully load and calculate trust score
4. Capture full-page screenshot
5. Extract metrics:
   - Trust Score (%)
   - Lies count
   - Anomalies count
   - navigator.webdriver value
   - WebGL parameters

**Success Indicators**:
- Trust Score = 100%
- Lies = 0
- Anomalies = 0
- navigator.webdriver = undefined
- WebGL Vendor = "Intel Inc."
- MAX_TEXTURE_SIZE = 16384

### Step 3: JA4 Fingerprint Verification

**Objective**: Verify TLS fingerprint matches Chrome

**Execution**:
1. Navigate to: `https://ja4db.com`
2. Verify traffic goes through StealthProxy
3. Capture JA4 fingerprint
4. Compare with expected Chrome signature

**Success Indicators**:
- JA4 fingerprint matches Chrome
- No automation tool signatures
- TLS 1.3 detected
- Extension order matches

### Step 4: High-Value Target Extraction

**Objective**: Extract 50 records with zero detection

**Execution**:
1. **Worker 1**:
   - Navigate to target site
   - Authenticate (if required)
   - Execute search
   - Extract first 25 records
   - Store Vector Experience in Redis

2. **Worker 2**:
   - Pull Vector Experience from Redis
   - Resume session (verify no re-authentication)
   - Extract remaining 25 records
   - Verify seamless continuation

**Success Indicators**:
- 50/50 records extracted
- Zero CAPTCHA triggers
- Zero rate-limit blocks
- Session continuity maintained
- Vector Experience shared successfully

## Log Capture

### Railway Logs

Capture logs during execution:

```bash
# Capture Body logs
railway logs --service chimera-core > mission_alpha_body.log

# Capture Brain logs
railway logs --service brainscraper > mission_alpha_brain.log
```

### Key Log Messages to Verify

**Startup**:
```
✅ Engine health verified: Binary is successfully sanitized
✅ Redis session verified: Identity Grafting active
🔒 TLS-JA4 Sidecar Proxy initialized with Chrome fingerprint
✅ Body Status: Sanitized and Ready
```

**During Execution**:
```
🔍 Verifying engine health (native engine verification)...
✅ Engine health verified: Binary is successfully sanitized
✅ Redis session verified: Identity Grafting active
```

## Screenshot Capture

### Required Screenshots

1. **CreepJS Results**:
   - Full page showing trust score
   - Zero lies/anomalies visible
   - File: `creepjs_100_percent_trust.png`

2. **JA4 Verification**:
   - JA4 fingerprint display
   - Chrome match confirmation
   - File: `ja4_chrome_match.png`

3. **High-Value Target**:
   - Worker 1 extraction progress
   - Worker 2 session resume
   - Final results (50 records)
   - Files: `extraction_worker1.png`, `extraction_worker2.png`, `extraction_complete.png`

## Metrics Collection

### CreepJS Metrics

```json
{
  "trust_score": 100,
  "lies": 0,
  "anomalies": 0,
  "navigator_webdriver": "undefined",
  "webgl_vendor": "Intel Inc.",
  "webgl_renderer": "Intel(R) Iris(R) Xe Graphics",
  "max_texture_size": 16384,
  "max_renderbuffer_size": 16384,
  "timestamp": "2026-01-16T..."
}
```

### JA4 Metrics

```json
{
  "ja4_fingerprint": "t13d1516h2_8daaf6152771_...",
  "matches_chrome": true,
  "tls_version": "TLS 1.3",
  "timestamp": "2026-01-16T..."
}
```

### Extraction Metrics

```json
{
  "records_extracted": 50,
  "total_records": 50,
  "captcha_triggers": 0,
  "rate_limit_blocks": 0,
  "session_continuity": true,
  "vector_experience_shared": true,
  "workers_used": 2,
  "timestamp": "2026-01-16T..."
}
```

## Report Generation

After execution, update `MISSION_ALPHA_RUN.md` with:
1. All captured metrics
2. Screenshot references
3. Log excerpts
4. Final verification status

## Troubleshooting

### If Trust Score < 100%

1. Check binary sanitization: `navigator.webdriver` should be undefined
2. Verify WebGL parameters: MAX_TEXTURE_SIZE should be 16384
3. Check behavioral patterns: Displacement should be wide, not sharp peaks
4. Verify Redis profile: Should have "lived-in" history

### If JA4 Doesn't Match

1. Verify StealthProxy is running
2. Check reqwest-impersonate version
3. Verify Chrome fingerprint configuration
4. Check TLS handshake logs

### If Extraction Fails

1. Verify Redis connection
2. Check Vector Experience storage
3. Verify session continuity
4. Check rate-limiting/CAPTCHA triggers

---

**Ready for Execution**: All systems verified  
**Next Step**: Execute Mission Alpha Run and capture results
