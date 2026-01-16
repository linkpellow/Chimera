# Railway CLI Setup - Full Control Guide

## Current Status

✅ **Authenticated**: `linkpellowinsurance@gmail.com`  
✅ **Project Linked**: `scrape-goat` (production environment)  
⚠️  **Services**: None found (need to create)

---

## Creating Services via CLI

Since Railway CLI requires interactive prompts for some operations, here's how to set up both services:

### Option 1: Create Services via Dashboard (Recommended)

1. **Create chimera-brain service:**
   - Go to Railway Dashboard → scrape-goat project
   - Click **"New Service"** → **"GitHub Repo"**
   - Select your Chimera repository
   - **Settings** → **Root Directory**: `/chimera-brain`
   - **Settings** → **Variables**:
     ```
     PORT=50052
     CHIMERA_VISION_PORT=50052
     REDIS_URL=${{Redis.REDIS_URL}}
     PYTHONUNBUFFERED=1
     ```
   - Rename service to: `chimera-brain`

2. **Create/Upgrade scrapegoat-worker-swarm:**
   - If service exists: Click it → **Settings**
   - If not: **"New Service"** → **"GitHub Repo"**
   - **Settings** → **Root Directory**: `/chimera-core`
   - **Settings** → **Variables**:
     ```
     CHIMERA_VISION_ADDR=http://chimera-brain.railway.internal:50052
     CHIMERA_AGENT_ADDR=0.0.0.0:50051
     CHIMERA_PROXY_PORT=8080
     CHROME_BIN=/usr/bin/chromium
     REDIS_URL=${{Redis.REDIS_URL}}
     RUST_LOG=info
     ```
   - **Settings** → **Scaling**: Set replicas to 50+

---

## Option 2: Create Services via CLI

### Create chimera-brain Service

```bash
# Get your GitHub repo URL
REPO_URL=$(git remote get-url origin)

# Create service (will prompt for repo selection)
cd /Users/linkpellow/CHIMERA/chimera-brain
railway add --service chimera-brain --repo $REPO_URL

# Link to service
railway service link chimera-brain

# Set root directory (via dashboard or API)
# Set variables
railway variables set PORT=50052
railway variables set CHIMERA_VISION_PORT=50052
railway variables set REDIS_URL='${{Redis.REDIS_URL}}'
railway variables set PYTHONUNBUFFERED=1
```

### Create scrapegoat-worker-swarm Service

```bash
# Create service
cd /Users/linkpellow/CHIMERA/chimera-core
railway add --service scrapegoat-worker-swarm --repo $REPO_URL

# Link to service
railway service link scrapegoat-worker-swarm

# Set variables
railway variables set CHIMERA_VISION_ADDR=http://chimera-brain.railway.internal:50052
railway variables set CHIMERA_AGENT_ADDR=0.0.0.0:50051
railway variables set CHIMERA_PROXY_PORT=8080
railway variables set CHROME_BIN=/usr/bin/chromium
railway variables set REDIS_URL='${{Redis.REDIS_URL}}'
railway variables set RUST_LOG=info
```

**Note**: Root directory must be set via Dashboard (Settings → Root Directory)

---

## Managing Services via CLI

### View Services

```bash
railway service status
```

### View Logs

```bash
# Brain service logs
railway logs --service chimera-brain

# Swarm service logs
railway logs --service scrapegoat-worker-swarm

# Follow logs in real-time
railway logs --service chimera-brain --follow
```

### Set Environment Variables

```bash
# Link to service first
railway service link chimera-brain

# Set variable
railway variables set KEY=value

# View all variables
railway variables

# Delete variable
railway variables delete KEY
```

### Deploy Services

```bash
# Deploy from current directory
railway up

# Redeploy latest
railway redeploy

# Deploy specific service
railway up --service chimera-brain
```

### Scale Services

```bash
# Scale service (via dashboard or API)
# CLI doesn't have direct scale command, use dashboard
```

---

## Monitoring & Debugging

### Check Service Status

```bash
railway status
railway service status
```

### SSH into Service

```bash
railway ssh --service chimera-brain
railway ssh --service scrapegoat-worker-swarm
```

### View Deployment History

```bash
railway deployment list
railway deployment logs <deployment-id>
```

---

## Quick Commands Reference

```bash
# Authentication
railway login
railway whoami

# Project Management
railway link -p scrape-goat
railway status
railway unlink

# Service Management
railway service link <service-name>
railway service status
railway add --service <name> --repo <repo-url>

# Variables
railway variables                    # List all
railway variables set KEY=value     # Set variable
railway variables delete KEY        # Delete variable

# Deployment
railway up                          # Deploy
railway redeploy                    # Redeploy latest
railway logs                        # View logs
railway logs --service <name>       # Service-specific logs

# Debugging
railway ssh --service <name>        # SSH into service
railway connect                     # Connect to database
```

---

## Next Steps

1. ✅ Project is linked (`scrape-goat`)
2. ⏳ Create `chimera-brain` service (via dashboard or CLI)
3. ⏳ Create/upgrade `scrapegoat-worker-swarm` service
4. ⏳ Set root directories (dashboard only)
5. ⏳ Configure environment variables
6. ⏳ Scale swarm to 50+ replicas
7. ⏳ Monitor logs and verify connections

---

## Troubleshooting

### "No service linked"
```bash
railway service link <service-name>
```

### "No project linked"
```bash
railway link -p scrape-goat
```

### Services not showing up
- Check Railway Dashboard
- Services may need to be created first
- Use `railway add` to create new services

---

*Last Updated: 2024-01-15*
