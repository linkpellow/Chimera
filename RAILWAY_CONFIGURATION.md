# Railway Configuration Status - my-lead-engine

**Last Updated**: 2024-01-15  
**Project**: `my-lead-engine` (production)  
**Status**: ✅ Variables Configured

---

## Services Status

| Service | Status | Purpose |
|---------|--------|---------|
| `scrapegoat-worker-swarm` | ✅ SUCCESS | Rust Body (Chimera Core) |
| `brainscraper` | ⚠️ FAILED | Python Brain (Chimera Brain) |
| `Redis` | ✅ SUCCESS | Hive Mind Storage |
| `Postgres` | ✅ SUCCESS | Database |

---

## Configuration Applied

### scrapegoat-worker-swarm (Chimera Core)

**Environment Variables Set:**
- ✅ `CHIMERA_VISION_ADDR=http://brainscraper.railway.internal:50052`
- ✅ `CHIMERA_AGENT_ADDR=0.0.0.0:50051`
- ✅ `CHIMERA_PROXY_PORT=8080`
- ✅ `CHROME_BIN=/usr/bin/chromium`
- ✅ `RUST_LOG=info`
- ✅ `REDIS_URL` (already configured)

**Next Steps:**
1. ⏳ Set **Root Directory** to `/chimera-core` (via Dashboard)
2. ⏳ Verify Dockerfile is being used
3. ⏳ Scale to 50+ replicas (via Dashboard)

### brainscraper (Chimera Brain)

**Environment Variables Set:**
- ✅ `PORT=50052`
- ✅ `CHIMERA_VISION_PORT=50052`
- ✅ `PYTHONUNBUFFERED=1`
- ✅ `REDIS_URL` (needs to be set: `${{Redis.REDIS_URL}}`)

**Next Steps:**
1. ⏳ Set **Root Directory** to `/chimera-brain` (via Dashboard)
2. ⏳ Set `REDIS_URL=${{Redis.REDIS_URL}}` (via Dashboard)
3. ⏳ Fix deployment issues (currently FAILED)

---

## Critical Dashboard Actions Required

### For scrapegoat-worker-swarm:

1. **Settings → Root Directory**: `/chimera-core`
2. **Settings → Scaling**: Set replicas to 50+
3. **Settings → Variables**: Verify all Chimera variables are set

### For brainscraper:

1. **Settings → Root Directory**: `/chimera-brain`
2. **Settings → Variables**: 
   - Add `REDIS_URL=${{Redis.REDIS_URL}}`
   - Verify `PORT=50052` is set
3. **Deployments**: Check why it's failing and redeploy

---

## Network Architecture

```
Railway Private Network (my-lead-engine)
│
├── brainscraper:50052 (Chimera Brain)
│   └── Connects to Redis (Hive Mind)
│
├── scrapegoat-worker-swarm:50051 (x50 replicas)
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

## Verification Commands

### Check Service Status
```bash
railway service status
```

### View Logs
```bash
# Brain service
railway logs --service brainscraper --lines 50

# Swarm service
railway logs --service scrapegoat-worker-swarm --lines 50
```

### Check Variables
```bash
# Swarm variables
railway service link scrapegoat-worker-swarm
railway variables

# Brain variables
railway service link brainscraper
railway variables
```

---

## Current Issues

1. **brainscraper is FAILED**
   - Needs root directory set to `/chimera-brain`
   - Needs `REDIS_URL` configured
   - May need Dockerfile verification

2. **Root Directories Not Set**
   - Must be set via Dashboard (Settings → Root Directory)
   - CLI cannot set root directories

3. **Scaling Not Configured**
   - Swarm should be scaled to 50+ replicas
   - Set via Dashboard (Settings → Scaling)

---

## Next Actions

1. ✅ Variables configured via CLI
2. ⏳ Set root directories via Dashboard
3. ⏳ Configure Redis URL for brainscraper
4. ⏳ Fix brainscraper deployment
5. ⏳ Scale swarm to 50+ replicas
6. ⏳ Verify services are communicating

---

## Quick Reference

**Project**: `my-lead-engine`  
**Environment**: `production`  
**CLI Status**: ✅ Linked and authenticated

**Services:**
- `scrapegoat-worker-swarm` (ID: `516ca7b1-f247-4fdb-acd8-f6beabe49b74`)
- `brainscraper` (ID: `4700a4a4-4093-4dc4-86e9-443a3b31679e`)
- `Redis` (ID: `4a318b21-b9fb-436b-9e38-4a5959b7e19d`)

---

*Configuration completed via Railway CLI. Dashboard actions required for root directories and scaling.*
