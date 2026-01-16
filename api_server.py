#!/usr/bin/env python3
"""
REST API Server for Project Chimera

This provides a simple REST API wrapper around the gRPC services
for easier integration with existing systems.
"""

import os
import json
import logging
from typing import Optional
from flask import Flask, request, jsonify, Response
from flask_cors import CORS
import grpc

# Try to import proto files, but handle gracefully if they don't exist
try:
    from chimera_brain import vision_pb2, vision_pb2_grpc
    PROTO_AVAILABLE = True
except ImportError:
    PROTO_AVAILABLE = False
    logging.warning("Proto files not found. Run 'make proto-python' to generate them.")

# Import the Rust-generated proto (we'll need to generate Python bindings)
# For now, we'll use a simple HTTP client approach

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

app = Flask(__name__)
CORS(app)

# Configuration
AGENT_GRPC_ADDR = os.getenv("CHIMERA_AGENT_ADDR", "127.0.0.1:50051")
VISION_GRPC_ADDR = os.getenv("CHIMERA_VISION_ADDR", "127.0.0.1:50052")


@app.route("/health", methods=["GET"])
def health():
    """Health check endpoint"""
    return jsonify({"status": "healthy", "service": "chimera-api"})


@app.route("/api/v1/agent/run", methods=["POST"])
def run_agent():
    """
    Run an agent objective.
    
    Request body:
    {
        "session_id": "job_123",
        "start_url": "https://example.com",
        "instruction": "Click the login button",
        "headless": true
    }
    """
    try:
        data = request.get_json()
        
        session_id = data.get("session_id", f"session_{os.urandom(8).hex()}")
        start_url = data.get("start_url")
        instruction = data.get("instruction")
        headless = data.get("headless", True)
        
        if not start_url or not instruction:
            return jsonify({
                "error": "Missing required fields: start_url and instruction"
            }), 400
        
        # For now, return a simple response
        # In production, you'd connect to the Rust gRPC service here
        logger.info(f"Agent request: {session_id} - {instruction} on {start_url}")
        
        return jsonify({
            "session_id": session_id,
            "status": "started",
            "message": "Objective started. Use /api/v1/agent/status/{session_id} to check progress."
        }), 202
        
    except Exception as e:
        logger.error(f"Error processing agent request: {e}", exc_info=True)
        return jsonify({"error": str(e)}), 500


@app.route("/api/v1/agent/status/<session_id>", methods=["GET"])
def get_agent_status(session_id: str):
    """Get status of an agent session"""
    # In production, this would query the Rust gRPC service
    return jsonify({
        "session_id": session_id,
        "status": "running",
        "message": "Agent is processing the objective"
    })


@app.route("/api/v1/agent/session/<session_id>", methods=["DELETE"])
def close_session(session_id: str):
    """Close an agent session"""
    logger.info(f"Closing session: {session_id}")
    # In production, this would call the Rust gRPC service
    return jsonify({
        "session_id": session_id,
        "status": "closed"
    })


@app.route("/api/v1/vision/coordinates", methods=["POST"])
def get_coordinates():
    """
    Get coordinates for a visual intent (direct vision service call).
    
    Request body (multipart/form-data or JSON with base64 image):
    {
        "image": "<base64_encoded_image>",
        "text_command": "Click the big green button"
    }
    """
    if not PROTO_AVAILABLE:
        return jsonify({
            "error": "Proto files not available. Run 'make proto-python' to generate them."
        }), 503
    
    try:
        data = request.get_json()
        
        if "image" not in data or "text_command" not in data:
            return jsonify({
                "error": "Missing required fields: image and text_command"
            }), 400
        
        # Decode base64 image
        import base64
        image_bytes = base64.b64decode(data["image"])
        text_command = data["text_command"]
        
        # Connect to vision service
        with grpc.insecure_channel(VISION_GRPC_ADDR) as channel:
            stub = vision_pb2_grpc.VisionServiceStub(channel)
            
            request_msg = vision_pb2.CoordinateRequest(
                image=image_bytes,
                text_command=text_command
            )
            
            response = stub.GetCoordinates(request_msg)
            
            return jsonify({
                "found": response.found,
                "x": response.x,
                "y": response.y,
                "width": response.width,
                "height": response.height,
                "confidence": response.confidence
            })
            
    except Exception as e:
        logger.error(f"Error processing vision request: {e}", exc_info=True)
        return jsonify({"error": str(e)}), 500


if __name__ == "__main__":
    # Check if flask is installed
    try:
        import flask
    except ImportError:
        logger.error("Flask not installed. Install with: pip install flask flask-cors")
        exit(1)
    
    port = int(os.getenv("CHIMERA_API_PORT", "8080"))
    logger.info(f"Starting Chimera API server on port {port}")
    app.run(host="0.0.0.0", port=port, debug=True)
