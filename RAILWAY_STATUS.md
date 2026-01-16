# Railway Configuration Status - my-lead-engine

**Last Checked**: 2024-01-15  
**Project**: `my-lead-engine` (production)

---

## Services Found

| Service | Status | Purpose | Action Needed |
|---------|--------|---------|--------------|
| `brainscraper` | ⚠️ FAILED | **Chimera Brain** (Python) | Set root directory to `/chimera-brain` |
| `scrapegoat-worker-swarm` | ✅ SUCCESS | **Chimera Core** (Rust) | Set root directory to `/chimera-core` |
| `Redis` | ✅ SUCCESS | Hive Mind Storage | None |
| `Postgres` | ✅ SUCCESS | Database | None |
| `scrapegoat` | ✅ SUCCESS | Other service | None |

---

## Current Configuration

### brainscraper (Chimera Brain)

**Variables Set:**
- ✅ `PORT=50052`
- ✅ `CHIMERA_VISION_PORT=50052`
- ✅ `PYTHONUNBUFFERED=1`
- ⚠️ `REDIS_URL` - Needs to be set to `${{Redis.REDIS_URL}}`

**Issue:**
- Currently building as **Next.js app** (npm build)
- **Root Directory** must be set to `/chimera-brain` in Dashboard
- This will switch it to use the Python Dockerfile

**Dashboard Action Required:**
1. Go to `brainscraper` service
2. **Settings → Root Directory**: Set to `/chimera-brain`
3. **Settings → Variables**: Verify `REDIS_URL=${{Redis.REDIS_URL}}`
4. Redeploy

### scrapegoat-worker-swarm (Chimera Core)

**Variables Set:**
- ✅ `CHIMERA_VISION_ADDR=http://brainscraper.railway.internal:50052`
- ✅ `CHIMERA_AGENT_ADDR=0.0.0.0:50051`
- ✅ `CHIMERA_PROXY_PORT=8080`
- ✅ `CHROME_BIN=/usr/bin/chromium`
- ✅ `RUST_LOG=info`
- ✅ `REDIS_URL` (already configured)

**Dashboard Action Required:**
1. Go to `scrapegoat-worker-swarm` service
2. **Settings → Root Directory**: Set to `/chimera-core`
3. **Settings → Scaling**: Set to 50+ replicas
4. Redeploy

---

## Network Architecture

```
Railway Private Network (my-lead-engine)
│
├── brainscraper:50052 (Chimera Brain - Python)
│   └── Connects to Redis (Hive Mind)
│
├── scrapegoat-worker-swarm:50051 (x50 replicas - Rust)
│   └── Connects to brainscraper via gRPC
│   └── Connects to Redis (Hive Mind)
│
└── Redis (Shared Memory)
    └── Stores experience vectors
```

**Private Network URLs:**
- Brain: `http://brainscraper.railway.internal:50052`
- Swarm: `scrapegoat-worker-swarm.railway.internal:50051`
- Redis: `redis.railway.internal:6379`

---

## Summary

**You're using `brainscraper` as the Chimera Brain service.**

This is fine, but you need to:
1. ✅ Variables are mostly configured (REDIS_URL needs fixing)
2. ⏳ **Set root directory to `/chimera-brain`** (Dashboard only)
3. ⏳ **Set root directory for swarm to `/chimera-core`** (Dashboard only)
4. ⏳ Scale swarm to 50+ replicas

Once root directories are set, both services will rebuild with the correct Dockerfiles and should work.

---

*Note: There is no separate `chimera-brain` service. `brainscraper` is serving as the brain.*
