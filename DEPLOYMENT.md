# Deployment Guide

## Quick Start with Docker

```bash
# 1. Generate proto files (required before building)
make proto-python

# 2. Build and start all services
docker-compose up -d

# 3. Check status
docker-compose ps

# 4. View logs
docker-compose logs -f chimera-core
docker-compose logs -f chimera-brain
```

## Manual Deployment

### Prerequisites

- Rust 1.75+
- Python 3.8+
- Chrome/Chromium
- (Optional) CUDA for GPU acceleration

### Step 1: Build Rust Core

```bash
cd chimera-core
cargo build --release
```

### Step 2: Set Up Python Brain

```bash
cd chimera-brain
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
make proto-python  # Generate gRPC code
```

### Step 3: Run Services

**Terminal 1 - Vision Service:**
```bash
cd chimera-brain
source venv/bin/activate
python -m chimera_brain.server
```

**Terminal 2 - Agent Service:**
```bash
cd chimera-core
CHIMERA_VISION_ADDR=http://127.0.0.1:50052 ./target/release/chimera-core
```

## Production Considerations

### Security

- Both Dockerfiles run as non-root user (`chimera`)
- Services communicate over private Docker network
- No exposed admin interfaces by default

### Performance

- **CPU**: Use `CHIMERA_USE_SIMPLE=true` for faster, less accurate detection
- **GPU**: Set `CHIMERA_VISION_DEVICE=cuda` for model acceleration
- **Memory**: Chrome needs ~2GB shared memory (set in docker-compose.yml)

### Scaling

- Run multiple `chimera-core` instances behind a load balancer
- Vision service can handle multiple concurrent requests
- Use session affinity for stateful operations

### Monitoring

- Health checks are built into Dockerfiles
- Logs are structured (JSON format recommended)
- Metrics can be exported via gRPC reflection

## Troubleshooting

**"Connection refused" errors:**
- Check services are running: `docker-compose ps`
- Verify ports aren't blocked: `netstat -tuln | grep 50051`

**"Browser launch failed":**
- Ensure Chrome dependencies are installed
- Check `/dev/shm` size (needs 2GB+)
- Verify DISPLAY variable if running headless

**"Vision service timeout":**
- Check GPU availability: `nvidia-smi`
- Use simple mode: `CHIMERA_USE_SIMPLE=true`
- Increase timeout in client code

## Kubernetes Deployment

See `k8s/` directory for Kubernetes manifests (coming soon).
