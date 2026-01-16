use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChimeraError {
    #[error("Browser error: {0}")]
    Browser(String),
    
    #[error("Vision service error: {0}")]
    Vision(String),
    
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    
    #[error("Action failed: {0}")]
    ActionFailed(String),
    
    #[error("gRPC error: {0}")]
    Grpc(#[from] tonic::Status),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, ChimeraError>;
