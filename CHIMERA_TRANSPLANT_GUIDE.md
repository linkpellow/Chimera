# Chimera Transplant Guide - my-lead-engine Monorepo Integration

## Overview

This guide provides instructions for transplanting the Chimera AI Brain service into your `my-lead-engine` monorepo as a dedicated subdirectory.

## Target Structure

After transplant, your monorepo should have:

```
my-lead-engine/
├── chimera-brain/          # AI Brain service (this transplant)
│   ├── chimera_brain/      # Core Python logic
│   │   ├── __init__.py
│   │   ├── server.py       # gRPC server
│   │   ├── vision_service.py
│   │   ├── hive_mind.py
│   │   ├── metacognitive.py
│   │   └── world_model.py
│   ├── Dockerfile          # Python 3.11 service definition
│   ├── requirements.txt    # AI-specific dependencies
│   ├── generate_proto.sh   # Proto code generation
│   ├── setup.py            # Python package setup
│   └── README.md           # Service documentation
├── proto/                  # Shared proto definitions (if not already present)
│   └── chimera.proto
└── ... (other services)
```

## Files to Transplant

### Core Files (Required)

1. **`chimera_brain/` folder** - Complete Python package
   - `__init__.py`
   - `server.py` - gRPC Vision Service server
   - `vision_service.py` - VLM coordinate detection
   - `hive_mind.py` - Redis vector experience storage
   - `metacognitive.py` - Cognitive layer
   - `world_model.py` - State management

2. **`Dockerfile`** - Service container definition
   - Python 3.11 slim-bookworm base
   - System dependencies
   - Proto generation during build
   - Port 50052 exposure

3. **`requirements.txt`** - Python dependencies
   - gRPC libraries
   - PyTorch & Transformers
   - Redis & Vector search
   - Image processing

4. **`generate_proto.sh`** - Proto code generator
   - Generates Python gRPC stubs
   - Handles proto file location

5. **`setup.py`** - Python package configuration
   - Package metadata
   - Dependency definitions

### Optional Files

- `README.md` - Service documentation
- `.dockerignore` - Docker build exclusions
- `.gitignore` - Git exclusions

## Transplant Steps

### Step 1: Create Target Directory

In your `my-lead-engine` monorepo:

```bash
mkdir -p chimera-brain/chimera_brain
```

### Step 2: Copy Core Files

```bash
# From CHIMERA repository root
cp -r chimera-brain/chimera_brain/* my-lead-engine/chimera-brain/chimera_brain/
cp chimera-brain/Dockerfile my-lead-engine/chimera-brain/
cp chimera-brain/requirements.txt my-lead-engine/chimera-brain/
cp chimera-brain/generate_proto.sh my-lead-engine/chimera-brain/
cp chimera-brain/setup.py my-lead-engine/chimera-brain/
```

### Step 3: Copy Proto Definitions

```bash
# Ensure proto directory exists in monorepo
mkdir -p my-lead-engine/proto
cp proto/chimera.proto my-lead-engine/proto/
```

### Step 4: Update Proto Paths

After transplant, update `generate_proto.sh` if proto location differs:

```bash
# If proto is at monorepo root:
python3 -m grpc_tools.protoc \
    -I../../proto \
    --python_out=. \
    --grpc_python_out=. \
    ../../proto/chimera.proto
```

### Step 5: Update Dockerfile Proto Path

In `Dockerfile`, adjust proto path if needed:

```dockerfile
# If proto is at monorepo root (one level up from chimera-brain):
RUN if [ -d "../../proto" ]; then \
        python -m grpc_tools.protoc \
            -I../../proto \
            --python_out=chimera_brain \
            --grpc_python_out=chimera_brain \
            ../../proto/chimera.proto; \
    fi
```

### Step 6: Generate Proto Code

```bash
cd my-lead-engine/chimera-brain
chmod +x generate_proto.sh
./generate_proto.sh
```

### Step 7: Verify Installation

```bash
cd my-lead-engine/chimera-brain
pip install -r requirements.txt
python -m chimera_brain.server --help
```

## Monorepo Integration

### Railway Deployment

If using Railway, ensure your `railway.json` or service configuration points to:

```json
{
  "services": {
    "chimera-brain": {
      "source": "chimera-brain",
      "dockerfile": "chimera-brain/Dockerfile",
      "port": 50052
    }
  }
}
```

### Docker Compose

If using docker-compose:

```yaml
services:
  chimera-brain:
    build:
      context: .
      dockerfile: chimera-brain/Dockerfile
    ports:
      - "50052:50052"
    environment:
      - CHIMERA_VISION_PORT=50052
      - REDIS_URL=redis://redis:6379
    depends_on:
      - redis
```

### Environment Variables

Required environment variables:

- `CHIMERA_VISION_PORT` (default: 50052)
- `REDIS_URL` (for Hive Mind)
- `CHIMERA_VISION_MODEL` (optional, VLM model name)
- `CHIMERA_VISION_DEVICE` (optional, cuda/cpu)

## Service Dependencies

### External Services

1. **Redis** - Required for Hive Mind vector storage
   - Connection: `REDIS_URL` environment variable
   - Used for: Vector experience caching, profile storage

2. **Proto Definitions** - Shared gRPC contract
   - Location: `proto/chimera.proto` (monorepo root or relative path)
   - Regenerated during Docker build

### Internal Dependencies

- Python 3.11+
- System: `build-essential`, `git` (for some Python packages)

## Verification Checklist

After transplant, verify:

- [ ] All Python files copied to `chimera_brain/`
- [ ] `Dockerfile` present and paths updated
- [ ] `requirements.txt` present
- [ ] `generate_proto.sh` executable and paths correct
- [ ] Proto files generated (`chimera_brain/vision_pb2.py`, `vision_pb2_grpc.py`)
- [ ] Docker build succeeds
- [ ] Service starts on port 50052
- [ ] gRPC endpoint responds to coordinate requests
- [ ] Redis connection works (if configured)

## Troubleshooting

### Proto Generation Fails

**Issue**: `generate_proto.sh` can't find proto files

**Solution**: Update proto path in `generate_proto.sh` to match monorepo structure

### Import Errors

**Issue**: `from chimera_brain import ...` fails

**Solution**: Ensure `chimera_brain/__init__.py` exists and package is installed:
```bash
pip install -e .
```

### Docker Build Fails

**Issue**: Proto generation fails during Docker build

**Solution**: Check proto path in Dockerfile matches monorepo structure

### Redis Connection Fails

**Issue**: Hive Mind can't connect to Redis

**Solution**: Verify `REDIS_URL` environment variable is set correctly

## Post-Transplant Configuration

### Update Service Discovery

If other services need to connect to chimera-brain, update service discovery:

- **Railway**: Use `chimera-brain.railway.internal:50052`
- **Docker Compose**: Use `chimera-brain:50052`
- **Kubernetes**: Use service name and port

### Update Rust Core Connection

In `chimera-core/src/main.rs`, ensure connection string matches:

```rust
const CHIMERA_VISION_ADDR: &str = "http://chimera-brain.railway.internal:50052";
```

## Success Criteria

✅ Service builds successfully  
✅ Proto code generated correctly  
✅ Service starts and listens on port 50052  
✅ gRPC endpoint responds to requests  
✅ Redis integration works (if configured)  
✅ Docker container runs without errors  

---

**Transplant Status**: Ready for execution  
**Target**: `my-lead-engine/chimera-brain/`  
**Service Port**: 50052  
**Dependencies**: Redis, Proto definitions
