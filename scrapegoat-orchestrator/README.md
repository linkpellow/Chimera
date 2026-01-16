# Scrapegoat Orchestrator

The control center for the Scrapegoat swarm. This manages:

- Mission queue (what to scrape)
- Worker pool (who does the work)
- World model (shared knowledge via Redis)
- Identity rotation (which digital identity to use)

## Architecture

```
Orchestrator (This Service)
    ├── API Server (REST/gRPC)
    ├── Mission Queue (Redis)
    ├── Worker Manager
    └── Identity Rotator

Workers (Chimera APEX)
    ├── Phantom (TLS mimicry)
    ├── Cortex (Dual-sense)
    └── Ghost (Neuromotor mouse)
```

## World Model (Redis)

The orchestrator maintains a "World Model" in Redis that all workers can read:

```json
{
  "amazon.com": {
    "login_button": {
      "last_seen": "2024-01-15T10:30:00Z",
      "coordinates": [450, 320],
      "ax_role": "button",
      "ax_name": "Sign in"
    },
    "cart_button": {
      "last_seen": "2024-01-15T10:31:00Z",
      "coordinates": [1200, 100]
    }
  }
}
```

When a worker discovers a new element location, it updates Redis. Other workers instantly know about it.

## Identity Rotation

Each worker can use a different "digital identity":

- **Identity A**: Windows 11, Chrome 124, Residential IP, 1920x1080
- **Identity B**: macOS 14, Safari 17, Mobile IP, Retina display
- **Identity C**: Linux, Firefox 120, Datacenter IP, 2560x1440

The orchestrator assigns identities to workers to avoid pattern detection.

## API Endpoints

### POST /api/v1/missions
Create a new scraping mission:

```json
{
  "target_url": "https://amazon.com",
  "objective": "Find the cheapest 4k monitor and add to cart",
  "priority": "high",
  "identity_profile": "windows_chrome_124"
}
```

### GET /api/v1/missions/:id
Get mission status

### GET /api/v1/workers
List all workers and their status

### POST /api/v1/world-model/update
Update the world model (called by workers)

## Implementation

This is a placeholder structure. In production, you would implement:

1. **Mission Queue**: Redis-based job queue
2. **Worker Pool**: Health checks and load balancing
3. **World Model**: Redis key-value store with TTL
4. **Identity Manager**: Rotate profiles across workers
5. **Metrics**: Prometheus integration

See `docker-compose.apex.yml` for deployment.
