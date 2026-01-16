"""
gRPC Server for Vision Service

This server exposes the vision model as a gRPC service that the Rust core can call.
"""

import os
import logging
import asyncio
from concurrent import futures
import grpc
from chimera_brain import vision_pb2, vision_pb2_grpc
from chimera_brain.vision_service import VisualIntentProcessor, SimpleCoordinateDetector

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class VisionServiceImpl(vision_pb2_grpc.VisionServiceServicer):
    """
    Implementation of the Vision Service gRPC interface.
    """
    
    def __init__(self, use_simple: bool = False):
        """
        Initialize the vision service.
        
        Args:
            use_simple: If True, use SimpleCoordinateDetector instead of full VLM
        """
        if use_simple:
            logger.info("Using simple coordinate detector")
            self.processor = SimpleCoordinateDetector()
        else:
            logger.info("Using full vision model")
            model_name = os.getenv("CHIMERA_VISION_MODEL", None)
            device = os.getenv("CHIMERA_VISION_DEVICE", None)
            try:
                self.processor = VisualIntentProcessor(model_name=model_name, device=device)
            except Exception as e:
                logger.warning(f"Failed to load full model, falling back to simple: {e}")
                self.processor = SimpleCoordinateDetector()
    
    def GetCoordinates(
        self, 
        request: vision_pb2.CoordinateRequest, 
        context: grpc.ServicerContext
    ) -> vision_pb2.CoordinateResponse:
        """
        Get coordinates for a visual intent.
        
        This is the main method called by the Rust core.
        """
        try:
            logger.info(f"Processing coordinate request: '{request.text_command}'")
            
            x, y, confidence = self.processor.get_click_coordinates(
                request.image,
                request.text_command
            )
            
            logger.info(f"Found coordinates: ({x}, {y}) with confidence: {confidence}")
            
            return vision_pb2.CoordinateResponse(
                found=True,
                x=x,
                y=y,
                width=50,  # Default width for click target
                height=50,  # Default height for click target
                confidence=confidence
            )
            
        except Exception as e:
            logger.error(f"Error processing coordinate request: {e}", exc_info=True)
            context.set_code(grpc.StatusCode.INTERNAL)
            context.set_details(f"Error processing request: {str(e)}")
            return vision_pb2.CoordinateResponse(
                found=False,
                x=0,
                y=0,
                width=0,
                height=0,
                confidence=0.0
            )


def serve(port: int = 50052, use_simple: bool = False):
    """
    Start the gRPC server.
    
    Args:
        port: Port to listen on
        use_simple: Use simple detector instead of full VLM
    """
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    vision_pb2_grpc.add_VisionServiceServicer_to_server(
        VisionServiceImpl(use_simple=use_simple),
        server
    )
    
    listen_addr = f"[::]:{port}"
    server.add_insecure_port(listen_addr)
    
    logger.info(f"Starting Vision Service on {listen_addr}")
    server.start()
    
    try:
        server.wait_for_termination()
    except KeyboardInterrupt:
        logger.info("Shutting down Vision Service")
        server.stop(0)


if __name__ == "__main__":
    import sys
    
    use_simple = "--simple" in sys.argv or os.getenv("CHIMERA_USE_SIMPLE", "false").lower() == "true"
    # Railway uses PORT environment variable, fallback to CHIMERA_VISION_PORT
    port = int(os.getenv("PORT", os.getenv("CHIMERA_VISION_PORT", "50052")))
    
    serve(port=port, use_simple=use_simple)
