# chimera-brain/Dockerfile
# The AI Cortex - Vision + Logic Service
# Optimized for Railway's build system

FROM python:3.11-slim-bookworm

WORKDIR /app

# 1. Install System Deps (Minimal)
# We need build tools for some Python packages (e.g., sentence-transformers)
RUN apt-get update && apt-get install -y \
    build-essential \
    git \
    && rm -rf /var/lib/apt/lists/*

# 2. Install Python Deps
# We use a large timeout because PyTorch/sentence-transformers are huge
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt --timeout 1000

# 3. Copy Code
COPY . .

# 4. Generate Proto Code (Auto-transplant)
# Railway's private network requires gRPC proto files
# If proto files are in parent directory, adjust path accordingly
RUN if [ -d "../proto" ]; then \
        python -m grpc_tools.protoc \
            -I../proto \
            --python_out=chimera_brain \
            --grpc_python_out=chimera_brain \
            ../proto/chimera.proto; \
    elif [ -f "proto/chimera.proto" ]; then \
        python -m grpc_tools.protoc \
            -Iproto \
            --python_out=chimera_brain \
            --grpc_python_out=chimera_brain \
            proto/chimera.proto; \
    else \
        echo "Warning: Proto files not found, assuming pre-generated"; \
    fi

# 5. Set Environment
ENV PYTHONUNBUFFERED=1
ENV CHIMERA_VISION_PORT=50052

# 6. Ignite
# Use PORT environment variable (Railway convention) or default to 50052
EXPOSE 50052
CMD python -m chimera_brain.server || python chimera_brain/server.py
