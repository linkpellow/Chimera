# Railway Deployment Guide - God Mode

## Architecture Overview

Chimera is split into **Two Railway Services** that communicate over Railway's private network:

1. **chimera-brain** (New Service)
   - Role: The AI Cortex (Vision + Logic)
   - Language: Python
   - Port: 50052
   - Internal Address: `chimera-brain.railway.internal:50052`

2. **scrapegoat-worker-swarm** (Existing Service - Upgraded)
   - Role: The Rust "Body" (Chimera Core)
   - Language: Rust
   - Port: 50051
   - Root Directory: `/chimera-core`

---

## Step 1: Create the Brain Service

### In Railway Dashboard:

1. Click **"New Service"** → **"GitHub Repo"**
2. Select your **Chimera repository**
3. Click the new service to configure it

### Configuration:

**Settings → Root Directory:**
```
/chimera-brain
```

**Settings → Variables:**
```
PORT=50052
CHIMERA_VISION_PORT=50052
PYTHONUNBUFFERED=1
REDIS_URL=${{Redis.REDIS_URL}}
```

**Rename Service:**
```
chimera-brain
```

### Build Process:

The Dockerfile will:
1. Install Python 3.11 + system dependencies
2. Install Python packages (PyTorch, sentence-transformers, etc.)
3. Generate gRPC proto files
4. Start the vision service on port 50052

**Wait for build to complete** (first build may take 10-15 minutes due to PyTorch).

---

## Step 2: Upgrade the Swarm Service

### In Railway Dashboard:

1. Click your existing **scrapegoat-worker-swarm** service
2. Go to **Settings**

### Configuration:

**Settings → Root Directory:**
```
/chimera-core
```

**Settings → Variables:**
```
BRAIN_URL=http://chimera-brain.railway.internal:50052
CHIMERA_VISION_ADDR=http://chimera-brain.railway.internal:50052
CHIMERA_AGENT_ADDR=0.0.0.0:50051
CHIMERA_PROXY_PORT=8080
CHROME_BIN=/usr/bin/chromium
RUST_LOG=info
REDIS_URL=${{Redis.REDIS_URL}}
```

**Scaling:**
- Set replica count to **50+** for swarm mode
- Each replica is a Chimera worker

### Build Process:

The Dockerfile will:
1. **Build Stage**: Compile Rust binary with all dependencies
2. **Runtime Stage**: Install Chromium + dependencies
3. Copy compiled binary
4. Start Chimera Core on port 50051

**Wait for build to complete** (first build may take 15-20 minutes due to Rust compilation).

---

## Step 3: Verify "God Mode" Deployment

### Check Brain Service:

1. Go to **chimera-brain** service
2. Check **Logs** tab
3. Look for:
   ```
   Starting Vision Service on [::]:50052
   ```

### Check Swarm Service:

1. Go to **scrapegoat-worker-swarm** service
2. Check **Logs** tab
3. Look for:
   ```
   👻 Starting Phantom Sidecar (Stealth Proxy)...
   ✅ Phantom Sidecar ready on port 8080
   🚀 Launching Chimera with Phantom Sidecar active...
   Starting Chimera Agent Service on 0.0.0.0:50051
   ```

### Check Redis:

1. Go to **Redis** service
2. Check **Data** tab
3. Look for keys starting with `experience:` (Hive Mind vectors)

---

## Network Architecture

```
┌─────────────────────────────────────────┐
│         Railway Private Network         │
│                                         │
│  ┌──────────────┐      ┌─────────────┐ │
│  │ chimera-brain│◄────►│   Redis     │ │
│  │  :50052      │      │             │ │
│  └──────┬───────┘      └─────────────┘ │
│         │                               │
│         │ gRPC                          │
│         │                               │
│  ┌──────▼──────────────┐               │
│  │scrapegoat-worker-   │               │
│  │swarm (x50 replicas) │               │
│  │  :50051             │               │
│  └─────────────────────┘               │
└─────────────────────────────────────────┘
```

**Key Points:**
- Services communicate via Railway's private network (`*.railway.internal`)
- No public internet required for inter-service communication
- Redis is shared across all workers (Hive Mind)
- Each worker connects to the Brain for vision processing

---

## Environment Variables Reference

### chimera-brain

| Variable | Value | Description |
|----------|-------|-------------|
| `PORT` | `50052` | gRPC service port |
| `CHIMERA_VISION_PORT` | `50052` | Vision service port |
| `REDIS_URL` | `${{Redis.REDIS_URL}}` | Redis connection (Hive Mind) |
| `PYTHONUNBUFFERED` | `1` | Python logging |

### scrapegoat-worker-swarm

| Variable | Value | Description |
|----------|-------|-------------|
| `CHIMERA_VISION_ADDR` | `http://chimera-brain.railway.internal:50052` | Brain service URL |
| `CHIMERA_AGENT_ADDR` | `0.0.0.0:50051` | Agent gRPC port |
| `CHIMERA_PROXY_PORT` | `8080` | Phantom Sidecar port |
| `CHROME_BIN` | `/usr/bin/chromium` | Chrome binary path |
| `REDIS_URL` | `${{Redis.REDIS_URL}}` | Redis connection |
| `RUST_LOG` | `info` | Logging level |

---

## Troubleshooting

### Brain Service Won't Start

**Error**: `ModuleNotFoundError: No module named 'chimera_brain'`

**Fix**: Check that Root Directory is set to `/chimera-brain`

---

### Swarm Service Can't Connect to Brain

**Error**: `Failed to connect to vision service`

**Fix**: 
1. Verify `CHIMERA_VISION_ADDR` is set to `http://chimera-brain.railway.internal:50052`
2. Ensure both services are in the same Railway project
3. Check that Brain service is running (green status)

---

### Chrome/Chromium Not Found

**Error**: `Failed to launch browser`

**Fix**: 
1. Verify `CHROME_BIN=/usr/bin/chromium` is set
2. Check Dockerfile includes Chromium installation
3. Ensure runtime stage copied Chromium correctly

---

### Proto Files Not Generated

**Error**: `proto file not found`

**Fix**: 
1. Check that proto files exist in `/proto` directory
2. Verify Dockerfile proto generation step
3. For Brain: Ensure proto files are accessible during build

---

## Scaling Strategy

### Brain Service:
- **Replicas**: 1-3 (AI is CPU/RAM intensive)
- **Resources**: High RAM (4GB+), CPU (2+ cores)

### Swarm Service:
- **Replicas**: 50+ (stateless workers)
- **Resources**: Medium RAM (1-2GB), CPU (1 core)

### Redis:
- **Plan**: Standard (shared across all services)
- **Usage**: Hive Mind vectors, experience cache

---

## Monitoring

### Key Metrics:

1. **Brain Service**:
   - Request latency (vision processing)
   - Memory usage (PyTorch models)
   - gRPC connection count

2. **Swarm Service**:
   - Active workers
   - Request throughput
   - Chrome process count

3. **Redis**:
   - Experience vector count (`experience:*` keys)
   - Memory usage
   - Connection count

---

## Next Steps

Once both services are green:

1. ✅ Brain is online and waiting for visual input
2. ✅ Swarm workers are connected to Brain
3. ✅ Redis is storing Hive Mind vectors
4. ✅ Phantom Sidecar is intercepting traffic
5. ✅ All workers share collective intelligence

**You are now running the Apex Predator on Railway.**

---

*Last Updated: 2024-01-15*
