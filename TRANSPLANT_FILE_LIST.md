# Chimera Transplant - File List

## Exact Files to Copy to `my-lead-engine/chimera-brain/`

### Directory Structure

```
chimera-brain/
├── chimera_brain/
│   ├── __init__.py
│   ├── server.py
│   ├── vision_service.py
│   ├── hive_mind.py
│   ├── metacognitive.py
│   └── world_model.py
├── Dockerfile
├── requirements.txt
├── generate_proto.sh
└── setup.py
```

### File Manifest

#### 1. Core Python Package (`chimera_brain/`)

| File | Source Path | Description |
|------|-------------|-------------|
| `__init__.py` | `chimera-brain/chimera_brain/__init__.py` | Package initialization |
| `server.py` | `chimera-brain/chimera_brain/server.py` | gRPC Vision Service server |
| `vision_service.py` | `chimera-brain/chimera_brain/vision_service.py` | VLM coordinate detection |
| `hive_mind.py` | `chimera-brain/chimera_brain/hive_mind.py` | Redis vector experience storage |
| `metacognitive.py` | `chimera-brain/chimera_brain/metacognitive.py` | Cognitive layer |
| `world_model.py` | `chimera-brain/chimera_brain/world_model.py` | State management |

#### 2. Build & Configuration Files

| File | Source Path | Description |
|------|-------------|-------------|
| `Dockerfile` | `chimera-brain/Dockerfile` | Python 3.11 service container |
| `requirements.txt` | `chimera-brain/requirements.txt` | Python dependencies (PyTorch, gRPC, Redis) |
| `generate_proto.sh` | `chimera-brain/generate_proto.sh` | Proto code generation script |
| `setup.py` | `chimera-brain/setup.py` | Python package setup |

#### 3. Shared Proto Definitions

| File | Source Path | Description |
|------|-------------|-------------|
| `chimera.proto` | `proto/chimera.proto` | gRPC service definitions |

## Quick Copy Commands

### From CHIMERA Repository Root

```bash
# Set target monorepo path
TARGET_REPO="/path/to/my-lead-engine"

# Create directory structure
mkdir -p $TARGET_REPO/chimera-brain/chimera_brain
mkdir -p $TARGET_REPO/proto

# Copy Python package
cp -r chimera-brain/chimera_brain/* $TARGET_REPO/chimera-brain/chimera_brain/

# Copy build files
cp chimera-brain/Dockerfile $TARGET_REPO/chimera-brain/
cp chimera-brain/requirements.txt $TARGET_REPO/chimera-brain/
cp chimera-brain/generate_proto.sh $TARGET_REPO/chimera-brain/
cp chimera-brain/setup.py $TARGET_REPO/chimera-brain/

# Copy proto definitions
cp proto/chimera.proto $TARGET_REPO/proto/

# Make script executable
chmod +x $TARGET_REPO/chimera-brain/generate_proto.sh
```

## Post-Copy Steps

1. **Update Proto Paths** (if monorepo structure differs):
   - Edit `generate_proto.sh` to match proto location
   - Edit `Dockerfile` proto path if needed

2. **Generate Proto Code**:
   ```bash
   cd $TARGET_REPO/chimera-brain
   ./generate_proto.sh
   ```

3. **Verify Structure**:
   ```bash
   cd $TARGET_REPO/chimera-brain
   ls -la chimera_brain/
   # Should show: __init__.py, server.py, vision_service.py, hive_mind.py, etc.
   ```

## File Count Summary

- **Python modules**: 6 files (`chimera_brain/`)
- **Build files**: 4 files (Dockerfile, requirements.txt, generate_proto.sh, setup.py)
- **Proto definitions**: 1 file (`proto/chimera.proto`)
- **Total**: 11 files

## Dependencies

### External (Required)
- Redis (for Hive Mind)
- Proto definitions location

### Internal (Included)
- All Python dependencies in `requirements.txt`
- All Python source code in `chimera_brain/`

---

**Ready for Transplant**: All files identified and listed  
**Target Location**: `my-lead-engine/chimera-brain/`  
**Service Port**: 50052
