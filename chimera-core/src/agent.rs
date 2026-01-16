use crate::browser::BrowserSession;
use crate::error::{ChimeraError, Result};
use crate::vision_client::VisionClient;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};
use tracing::{debug, error, info, warn};
use std::sync::Mutex;

pub mod proto {
    tonic::include_proto!("chimera");
}

use proto::{
    chimera_agent_server::ChimeraAgent, ActionRequest, ActionResponse, ActionType,
    CloseSessionRequest, CloseSessionResponse, GetStateRequest, GetStateResponse,
    NavigateRequest, NavigateResponse, ObjectiveRequest, ObjectiveUpdate,
    StartSessionRequest, StartSessionResponse,
};

pub struct ChimeraAgentService {
    sessions: Arc<RwLock<HashMap<String, Arc<Mutex<BrowserSession>>>>>,
    vision_client: Arc<RwLock<Option<VisionClient>>>,
    vision_service_addr: String,
}

impl ChimeraAgentService {
    pub fn new(vision_service_addr: String) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            vision_client: Arc::new(RwLock::new(None)),
            vision_service_addr,
        }
    }

    async fn get_vision_client(&self) -> Result<VisionClient> {
        // For simplicity, create a new connection each time
        // In production, you'd want connection pooling
        VisionClient::connect(self.vision_service_addr.clone())
            .await
            .map_err(|e| ChimeraError::Vision(format!("Failed to connect: {}", e)))
    }
}

#[tonic::async_trait]
impl ChimeraAgent for ChimeraAgentService {
    async fn start_session(
        &self,
        request: Request<StartSessionRequest>,
    ) -> Result<Response<StartSessionResponse>, Status> {
        let req = request.into_inner();
        info!("Starting session: {}", req.session_id);

        let session = BrowserSession::new(req.session_id.clone(), req.headless)
            .map_err(|e| Status::internal(format!("Failed to start session: {}", e)))?;

        let mut sessions = self.sessions.write().await;
        sessions.insert(req.session_id.clone(), Arc::new(Mutex::new(session)));

        Ok(Response::new(StartSessionResponse {
            success: true,
            message: "Session started".to_string(),
        }))
    }

    async fn navigate(
        &self,
        request: Request<NavigateRequest>,
    ) -> Result<Response<NavigateResponse>, Status> {
        let req = request.into_inner();
        let sessions = self.sessions.read().await;
        
        let session = sessions
            .get(&req.session_id)
            .ok_or_else(|| Status::not_found(format!("Session not found: {}", req.session_id)))?
            .clone();
        
        drop(sessions);
        
        let session = session.lock().unwrap();
        session
            .navigate(&req.url)
            .map_err(|e| Status::internal(format!("Navigation failed: {}", e)))?;

        Ok(Response::new(NavigateResponse {
            success: true,
            message: "Navigation successful".to_string(),
        }))
    }

    async fn perform_action(
        &self,
        request: Request<ActionRequest>,
    ) -> Result<Response<ActionResponse>, Status> {
        let req = request.into_inner();
        debug!("Performing action: {} for session: {}", req.intent, req.session_id);

        let sessions = self.sessions.read().await;
        let session = sessions
            .get(&req.session_id)
            .ok_or_else(|| Status::not_found(format!("Session not found: {}", req.session_id)))?
            .clone();
        
        drop(sessions);

        // Capture current state
        let screenshot = {
            let session = session.lock().unwrap();
            session
                .capture_screenshot()
                .map_err(|e| Status::internal(format!("Screenshot failed: {}", e)))?
        };

        // Get coordinates from vision service
        let mut vision = self.get_vision_client().await
            .map_err(|e| Status::internal(format!("Vision service error: {}", e)))?;
        
        let (x, y, confidence) = vision
            .get_coordinates(screenshot.clone(), req.intent.clone())
            .await
            .map_err(|e| Status::internal(format!("Vision service error: {}", e)))?;

        debug!("Found element at ({}, {}) with confidence: {}", x, y, confidence);

        // Perform the action with OODA loop verification
        let new_screenshot = match req.action_type() {
            ActionType::Click => {
                // Use OODA loop for self-healing clicks
                let session_ref = session.clone();
                crate::ooda::execute_with_verification(
                    &*session_ref.lock().unwrap(),
                    &mut vision,
                    &req.intent,
                    3, // max retries
                )
                .await
                .map_err(|e| Status::internal(format!("OODA loop failed: {}", e)))?;
                
                // Capture new state after successful action
                session_ref.lock().unwrap()
                    .capture_screenshot()
                    .map_err(|e| Status::internal(format!("Screenshot failed: {}", e)))?
            }
            ActionType::Type => {
                let session_ref = session.clone();
                if let Some(text) = req.text {
                    crate::ooda::type_with_verification(
                        &*session_ref.lock().unwrap(),
                        &mut vision,
                        &req.intent,
                        &text,
                        3,
                    )
                    .await
                    .map_err(|e| Status::internal(format!("Type with verification failed: {}", e)))?;
                }
                
                // Capture new state
                session_ref.lock().unwrap()
                    .capture_screenshot()
                    .map_err(|e| Status::internal(format!("Screenshot failed: {}", e)))?
            }
            ActionType::Scroll => {
                {
                    let session = session.lock().unwrap();
                    // Default scroll down
                    session
                        .scroll(x, y, 0, 500)
                        .map_err(|e| Status::internal(format!("Scroll failed: {}", e)))?;
                }
                
                // Capture new state
                session.lock().unwrap()
                    .capture_screenshot()
                    .map_err(|e| Status::internal(format!("Screenshot failed: {}", e)))?
            }
            ActionType::Wait => {
                drop(session);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                
                // Re-acquire session for screenshot
                let sessions = self.sessions.read().await;
                let session = sessions
                    .get(&req.session_id)
                    .ok_or_else(|| Status::not_found(format!("Session not found: {}", req.session_id)))?
                    .clone();
                drop(sessions);
                
                session.lock().unwrap()
                    .capture_screenshot()
                    .map_err(|e| Status::internal(format!("Screenshot failed: {}", e)))?
            }
        };

        let new_state = format!("Action completed at ({}, {})", x, y);

        Ok(Response::new(ActionResponse {
            success: true,
            message: format!("Action completed with confidence: {}", confidence),
            new_state,
            screenshot: new_screenshot,
        }))
    }

    async fn get_state(
        &self,
        request: Request<GetStateRequest>,
    ) -> Result<Response<GetStateResponse>, Status> {
        let req = request.into_inner();
        let sessions = self.sessions.read().await;
        
        let session = sessions
            .get(&req.session_id)
            .ok_or_else(|| Status::not_found(format!("Session not found: {}", req.session_id)))?
            .clone();
        
        drop(sessions);
        
        let session = session.lock().unwrap();
        let screenshot = session
            .capture_screenshot()
            .map_err(|e| Status::internal(format!("Screenshot failed: {}", e)))?;
        
        let url = session
            .get_url()
            .map_err(|e| Status::internal(format!("Get URL failed: {}", e)))?;
        
        let title = session
            .get_title()
            .map_err(|e| Status::internal(format!("Get title failed: {}", e)))?;

        Ok(Response::new(GetStateResponse {
            screenshot,
            url,
            title,
        }))
    }

    type RunObjectiveStream = tokio_stream::wrappers::ReceiverStream<Result<ObjectiveUpdate, Status>>;

    async fn run_objective(
        &self,
        request: Request<ObjectiveRequest>,
    ) -> Result<Response<Self::RunObjectiveStream>, Status> {
        let req = request.into_inner();
        info!("Running objective: {} for session: {}", req.instruction, req.session_id);

        let (tx, rx) = tokio::sync::mpsc::channel(128);

        // Start the objective loop in a background task
        let sessions = Arc::clone(&self.sessions);
        let vision_client = Arc::clone(&self.vision_client);
        let session_id = req.session_id.clone();
        let start_url = req.start_url.clone();
        let instruction = req.instruction.clone();

        let vision_service_addr = self.vision_service_addr.clone();
        tokio::spawn(async move {
            // Start session if needed
            let session_arc = if !sessions.read().await.contains_key(&session_id) {
                let new_session = BrowserSession::new(session_id.clone(), req.headless)
                    .expect("Failed to start session");
                let mut sessions_write = sessions.write().await;
                let arc = Arc::new(Mutex::new(new_session));
                sessions_write.insert(session_id.clone(), arc.clone());
                arc
            } else {
                sessions.read().await.get(&session_id).unwrap().clone()
            };

            // Navigate to start URL
            let _ = tx.send(Ok(ObjectiveUpdate {
                status: "observing".to_string(),
                message: format!("Navigating to {}", start_url),
                screenshot: vec![],
                last_action: None,
            })).await;

            {
                let session = session_arc.lock().unwrap();
                if let Err(e) = session.navigate(&start_url) {
                    let _ = tx.send(Err(Status::internal(format!("Navigation failed: {}", e)))).await;
                    return;
                }
            }

            // Main agent loop: Observe -> Think -> Act -> Verify
            let max_iterations = 20;
            for iteration in 0..max_iterations {
                // Observe
                let screenshot = {
                    let session = session_arc.lock().unwrap();
                    match session.capture_screenshot() {
                        Ok(s) => s,
                        Err(e) => {
                            let _ = tx.send(Err(Status::internal(format!("Screenshot failed: {}", e)))).await;
                            break;
                        }
                    }
                };

                let _ = tx.send(Ok(ObjectiveUpdate {
                    status: "observing".to_string(),
                    message: format!("Iteration {}: Observing current state", iteration + 1),
                    screenshot: screenshot.clone(),
                    last_action: None,
                })).await;

                // Think (get coordinates)
                // While thinking, perform micro-fidgeting to avoid "dead mouse" detection
                let thinking_task = {
                    let session_ref = session_arc.clone();
                    tokio::spawn(async move {
                        let mut fidget_count = 0;
                        loop {
                            if let Ok(session) = session_ref.lock() {
                                if let Err(e) = session.perform_micro_fidget().await {
                                    debug!("Micro-fidget error (non-fatal): {}", e);
                                }
                                fidget_count += 1;
                                if fidget_count > 10 {
                                    break; // Stop after reasonable number of fidgets
                                }
                            }
                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        }
                    })
                };
                
                let mut vision = match VisionClient::connect(vision_service_addr.clone()).await {
                    Ok(v) => v,
                    Err(e) => {
                        thinking_task.abort();
                        let _ = tx.send(Ok(ObjectiveUpdate {
                            status: "error".to_string(),
                            message: format!("Vision service error: {}", e),
                            screenshot: vec![],
                            last_action: None,
                        })).await;
                        break;
                    }
                };
                
                // Wait for vision response (fidgeting continues in background)
                // Abort fidgeting task once we have coordinates
                thinking_task.abort();
                
                let (x, y, confidence) = match vision.get_coordinates(screenshot, instruction.clone()).await {
                    Ok(coords) => coords,
                    Err(e) => {
                        let _ = tx.send(Ok(ObjectiveUpdate {
                            status: "error".to_string(),
                            message: format!("Vision service error: {}", e),
                            screenshot: vec![],
                            last_action: None,
                        })).await;
                        break;
                    }
                };

                let _ = tx.send(Ok(ObjectiveUpdate {
                    status: "thinking".to_string(),
                    message: format!("Found target at ({}, {}) with confidence: {}", x, y, confidence),
                    screenshot: vec![],
                    last_action: None,
                })).await;

                // Act
                {
                    let session = session_arc.lock().unwrap();
                    if let Err(e) = session.click(x, y) {
                        let _ = tx.send(Err(Status::internal(format!("Click failed: {}", e)))).await;
                        break;
                    }
                }

                let new_screenshot = {
                    let session = session_arc.lock().unwrap();
                    session.capture_screenshot().unwrap_or_default()
                };

                let action_response = ActionResponse {
                    success: true,
                    message: "Action completed".to_string(),
                    new_state: format!("Clicked at ({}, {})", x, y),
                    screenshot: new_screenshot.clone(),
                };

                let _ = tx.send(Ok(ObjectiveUpdate {
                    status: "acting".to_string(),
                    message: "Action executed".to_string(),
                    screenshot: action_response.screenshot.clone(),
                    last_action: Some(action_response),
                })).await;

                // Verify (simple: wait and check)
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                // For now, we'll assume success after one action
                // In a real implementation, you'd verify the objective was met
                if iteration == 0 {
                    let _ = tx.send(Ok(ObjectiveUpdate {
                        status: "complete".to_string(),
                        message: "Objective completed".to_string(),
                        screenshot: new_screenshot,
                        last_action: None,
                    })).await;
                    break;
                }
            }
        });

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }

    async fn close_session(
        &self,
        request: Request<CloseSessionRequest>,
    ) -> Result<Response<CloseSessionResponse>, Status> {
        let req = request.into_inner();
        info!("Closing session: {}", req.session_id);

        let mut sessions = self.sessions.write().await;
        sessions.remove(&req.session_id);

        Ok(Response::new(CloseSessionResponse { success: true }))
    }
}
