# Quick Start Guide

Get Project Chimera running in 5 minutes.

## Prerequisites

- Rust (install from https://rustup.rs/)
- Python 3.8+
- Chrome/Chromium browser

## Setup

### 1. Build Everything

```bash
make setup
```

This will:
- Build the Rust core
- Set up Python virtual environment
- Install dependencies
- Generate gRPC code

### 2. Run the Services

**Terminal 1 - Vision Service:**
```bash
make run-vision
```

**Terminal 2 - Agent Service:**
```bash
make run-agent
```

**Terminal 3 (Optional) - REST API:**
```bash
pip install flask flask-cors
make run-api
```

## Test It

### Using REST API

```bash
curl -X POST http://localhost:8080/api/v1/agent/run \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": "test_1",
    "start_url": "https://example.com",
    "instruction": "Click the More information link",
    "headless": true
  }'
```

### Using Python gRPC Client

```python
import grpc
# After generating proto files:
# from chimera_brain import vision_pb2, vision_pb2_grpc

channel = grpc.insecure_channel('localhost:50051')
# Use the stub to call methods
```

## Troubleshooting

**"Proto files not found"**
```bash
make proto-python
```

**"Vision service connection failed"**
- Make sure vision service is running on port 50052
- Check `CHIMERA_VISION_ADDR` environment variable

**"Browser launch failed"**
- Make sure Chrome/Chromium is installed
- On Linux, you may need: `apt-get install chromium-browser`

## Next Steps

1. Replace the simple vision detector with a real VLM model
2. Fine-tune the model on your UI datasets
3. Customize the agent loop for your use case
4. Add monitoring and logging

See README.md for full documentation.
