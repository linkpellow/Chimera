#!/bin/bash
# Generate Python gRPC code from proto files

python3 -m grpc_tools.protoc \
    -I../proto \
    --python_out=. \
    --grpc_python_out=. \
    ../proto/chimera.proto

# Rename the generated file to match our package structure
if [ -f chimera_pb2.py ]; then
    mv chimera_pb2.py chimera_brain/vision_pb2.py
fi

if [ -f chimera_pb2_grpc.py ]; then
    mv chimera_pb2_grpc.py chimera_brain/vision_pb2_grpc.py
fi

echo "Proto files generated successfully!"
