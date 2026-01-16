.PHONY: build-rust build-python proto-rust proto-python run-vision run-agent run-api help

help:
	@echo "Project Chimera - Build and Run Commands"
	@echo ""
	@echo "Build:"
	@echo "  make build-rust      - Build Rust core"
	@echo "  make build-python    - Install Python dependencies"
	@echo "  make proto-python    - Generate Python gRPC code"
	@echo ""
	@echo "Run:"
	@echo "  make run-vision      - Start vision service (Python)"
	@echo "  make run-agent       - Start agent service (Rust)"
	@echo "  make run-api         - Start REST API server"
	@echo ""
	@echo "All:"
	@echo "  make setup           - Full setup (build + proto generation)"

setup: build-rust build-python proto-python
	@echo "Setup complete!"

build-rust:
	@echo "Building Rust core..."
	cd chimera-core && cargo build --release

build-python:
	@echo "Setting up Python environment..."
	cd chimera-brain && \
		python3 -m venv venv || true && \
		. venv/bin/activate && \
		pip install -r requirements.txt

proto-python:
	@echo "Generating Python gRPC code..."
	cd chimera-brain && \
		python3 -m grpc_tools.protoc \
			-I../proto \
			--python_out=. \
			--grpc_python_out=. \
			../proto/chimera.proto && \
		mv chimera_pb2.py chimera_brain/vision_pb2.py 2>/dev/null || true && \
		mv chimera_pb2_grpc.py chimera_brain/vision_pb2_grpc.py 2>/dev/null || true
	@echo "Proto files generated!"

run-vision:
	@echo "Starting vision service..."
	cd chimera-brain && \
		. venv/bin/activate && \
		python -m chimera_brain.server

run-agent:
	@echo "Starting agent service..."
	cd chimera-core && \
		CHIMERA_VISION_ADDR=http://127.0.0.1:50052 cargo run --release

run-api:
	@echo "Starting REST API server..."
	python3 api_server.py
