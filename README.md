# Project Chimera

A modular, backend-only browser automation engine built with Rust (for speed) and Python (for AI vision), exposing a gRPC API for seamless integration.

## Architecture

Project Chimera is a **Computer Vision Control Loop** - not a browser wrapper. It uses:

1. **The Body (Rust)**: High-performance browser control via Chrome DevTools Protocol
2. **The Brain (Python)**: Vision-Language Model for understanding visual intent
3. **The Nervous System (OODA Loop)**: Self-healing visual feedback loop
4. **The Protocol (gRPC)**: Fast, type-safe communication between components

### Why This Approach?

- **Unbreakable**: Works even when websites change their HTML/CSS - it uses visual understanding, not selectors
- **Stealth**: Bezier curve mouse movements make it indistinguishable from human behavior
- **Self-Healing**: OODA loop (Observe-Orient-Decide-Act) verifies actions and retries on failure
- **Universal**: Works on Canvas apps, WebGL games, and highly dynamic React apps
- **Fast**: Rust core provides zero garbage collection pauses and raw speed
- **Anti-Fingerprinting**: Random variations in mouse paths prevent pattern detection

## Project Structure

```
CHIMERA/
├── chimera-core/          # Rust core (browser control, gRPC server)
│   ├── src/
│   │   ├── agent.rs      # Main agent service
│   │   ├── browser.rs    # Browser session management
│   │   ├── vision_client.rs  # Vision service client
│   │   └── main.rs       # Entry point
│   └── Cargo.toml
├── chimera-brain/         # Python vision service
│   ├── chimera_brain/
│   │   ├── vision_service.py  # VLM integration
│   │   └── server.py     # gRPC server
│   └── requirements.txt
├── proto/                 # gRPC protocol definitions
│   └── chimera.proto
└── api_server.py          # Optional REST API wrapper
```

## Setup

### Prerequisites

- Rust (latest stable)
- Python 3.8+
- Chrome/Chromium browser
- (Optional) CUDA-capable GPU for faster vision processing

### 1. Build the Rust Core

```bash
cd chimera-core
cargo build --release
```

### 2. Set Up Python Brain Service

```bash
cd chimera-brain
python3 -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
pip install -r requirements.txt

# Generate gRPC Python code
./generate_proto.sh
```

### 3. Run the Services

**Terminal 1 - Vision Service (Python):**
```bash
cd chimera-brain
source venv/bin/activate
python -m chimera_brain.server
# Or use simple mode (faster, less accurate):
# CHIMERA_USE_SIMPLE=true python -m chimera_brain.server
```

**Terminal 2 - Agent Service (Rust):**
```bash
cd chimera-core
CHIMERA_VISION_ADDR=http://127.0.0.1:50052 cargo run --release
```

**Terminal 3 - (Optional) REST API:**
```bash
pip install flask flask-cors
python api_server.py
```

## Usage

### gRPC API (Recommended)

The Rust core exposes a gRPC service. You can interact with it using any gRPC client.

**Example: Start a session and run an objective**

```python
import grpc
from chimera_brain import vision_pb2, vision_pb2_grpc

# Connect to agent service
channel = grpc.insecure_channel('localhost:50051')
stub = vision_pb2_grpc.ChimeraAgentStub(channel)

# Start session
session_req = vision_pb2.StartSessionRequest(
    session_id="my_session",
    headless=True
)
stub.StartSession(session_req)

# Run objective
objective_req = vision_pb2.ObjectiveRequest(
    session_id="my_session",
    start_url="https://example.com",
    instruction="Click the login button",
    headless=True
)

# Stream updates
for update in stub.RunObjective(objective_req):
    print(f"Status: {update.status} - {update.message}")
    if update.status == "complete":
        break
```

### REST API (Simpler)

```bash
# Run an agent objective
curl -X POST http://localhost:8080/api/v1/agent/run \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": "job_123",
    "start_url": "https://amazon.com",
    "instruction": "Find the cheapest 4k monitor and add to cart",
    "headless": true
  }'

# Check status
curl http://localhost:8080/api/v1/agent/status/job_123
```

## How It Works

### The OODA Loop (Nervous System)

1. **Observe**: Capture screenshot and hash it to get visual state
2. **Orient**: Vision model analyzes screenshot + command → returns (x, y) coordinates
3. **Decide**: Determine action (click, type, scroll) based on intent
4. **Act**: Execute with human-like Bezier curve mouse movement
5. **Loop**: Verify screen changed by comparing hashes, retry if not

### Stealth Layer

- **Bezier Curves**: Mouse movements follow curved paths (not straight lines)
- **Random Variations**: Each movement is unique (no pattern matching)
- **Human Timing**: Variable delays and acceleration/deceleration
- **Anti-Fingerprinting**: Impossible to detect as a bot by movement alone

### Example Flow

```
User: "Click the big green button"
  ↓
Rust: Captures screenshot → sends to Python
  ↓
Python: VLM analyzes image + command → returns (450, 320)
  ↓
Rust: Moves mouse to (450, 320) → clicks
  ↓
Rust: Captures new screenshot → verifies action
```

## Vision Model Integration

The default implementation uses simple heuristics. For production, you should:

1. **Fine-tune a Vision-Language Model** (like LLaVA, Fuyu, or BLIP) to output bounding boxes
2. **Replace the model** in `chimera_brain/vision_service.py`
3. **Train on UI element datasets** for better accuracy

Example models to consider:
- `microsoft/kosmos-2-patch14-224` - Good for UI understanding
- `Salesforce/blip-image-captioning-base` - Lightweight option
- Fine-tuned LLaVA for coordinate detection

## Docker Deployment

The easiest way to deploy Chimera is using Docker Compose:

```bash
# Build and start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

This will start:
- `chimera-brain`: Vision service on port 50052
- `chimera-core`: Agent service on port 50051
- `chimera-api`: REST API on port 8080

### GPU Support

For GPU-accelerated vision processing:

```bash
# Build with GPU support
docker-compose build --build-arg target=gpu chimera-brain

# Set device to CUDA
docker-compose up -d -e CHIMERA_VISION_DEVICE=cuda
```

## Configuration

### Environment Variables

**Rust Core:**
- `CHIMERA_AGENT_ADDR`: gRPC server address (default: `0.0.0.0:50051`)
- `CHIMERA_VISION_ADDR`: Vision service address (default: `http://127.0.0.1:50052`)

**Python Brain:**
- `CHIMERA_VISION_PORT`: gRPC server port (default: `50052`)
- `CHIMERA_USE_SIMPLE`: Use simple detector (default: `false`)
- `CHIMERA_VISION_MODEL`: Model name to load
- `CHIMERA_VISION_DEVICE`: Device (`cuda` or `cpu`)

**API Server:**
- `CHIMERA_API_PORT`: REST API port (default: `8080`)

## Development

### Building Proto Files

**Rust (automatic on build):**
```bash
cd chimera-core
cargo build
```

**Python:**
```bash
cd chimera-brain
./generate_proto.sh
```

### Testing

```bash
# Test vision service
python -m chimera_brain.server --simple

# Test Rust core
cd chimera-core
cargo test
```

## Roadmap

- [ ] Fine-tuned VLM model for coordinate detection
- [ ] Multi-step objective planning
- [ ] Better verification logic
- [ ] Session persistence
- [ ] Distributed execution
- [ ] WebSocket streaming for real-time updates

## License

MIT

## Contributing

This is a blueprint implementation. For production use, you'll want to:
- Fine-tune the vision model on UI datasets
- Add proper error handling and retries
- Implement session management
- Add monitoring and logging
- Optimize for your specific use case

---

**Built for speed. Built for stealth. Built for the future of browser automation.**
