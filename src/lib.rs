mod bindings;

use bindings::exports::ntwk::theater::actor::Guest as ActorGuest;
use bindings::exports::ntwk::theater::message_server_client::Guest as MessageServerClient;
use bindings::ntwk::theater::filesystem::{
    create_dir, delete_dir, delete_file, list_files, read_file, write_file,
};
use bindings::ntwk::theater::runtime::log;
use bindings::ntwk::theater::types::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
struct InitData {
    permissions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct State {
    permissions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FsRequest {
    operation: String,
    path: String,
    content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FsResponse {
    success: bool,
    data: Option<serde_json::Value>,
    error: Option<String>,
}

struct Component;

impl ActorGuest for Component {
    fn init(data: Option<Vec<u8>>) -> Vec<u8> {
        log("Initializing");
        let init_data: InitData = if let Some(data) = data {
            serde_json::from_slice(&data).unwrap_or(InitData {
                permissions: vec!["read".to_string()],
            })
        } else {
            InitData {
                permissions: vec!["read".to_string()],
            }
        };

        log(&format!("Permissions: {:?}", init_data.permissions));

        let state = State {
            permissions: init_data.permissions,
        };
        serde_json::to_vec(&state).unwrap()
    }
}

impl MessageServerClient for Component {
    fn handle_request(message: Json, state: Json) -> (Json, Json) {
        log("Handling request");
        log(&format!("Message: {:?}", message));
        let state: State = serde_json::from_slice(&state).unwrap();
        let request: FsRequest = match serde_json::from_slice(&message) {
            Ok(req) => req,
            Err(e) => {
                let response = FsResponse {
                    success: false,
                    data: None,
                    error: Some(format!("Invalid request format: {}", e)),
                };
                return (
                    serde_json::to_vec(&response).unwrap(),
                    serde_json::to_vec(&state).unwrap(),
                );
            }
        };

        // Handle operations that need responses
        let response = match request.operation.as_str() {
            "read-file" => {
                log(&format!("Reading file: {}", request.path));
                if !state.permissions.contains(&"read".to_string()) {
                    log("Read permission denied");
                    FsResponse {
                        success: false,
                        data: None,
                        error: Some("Read permission denied".to_string()),
                    }
                } else {
                    match read_file(&request.path) {
                        Ok(content) => {
                            log(&format!("Read file: {}", request.path));
                            let content_str = String::from_utf8_lossy(&content).to_string();
                            FsResponse {
                                success: true,
                                data: Some(json!(content_str)),
                                error: None,
                            }
                        }
                        Err(e) => FsResponse {
                            success: false,
                            data: None,
                            error: Some(format!("Failed to read file: {}", e)),
                        },
                    }
                }
            }
            "list-files" => {
                log(&format!("Listing files in: {}", request.path));
                if !state.permissions.contains(&"read".to_string()) {
                    log("Read permission denied");
                    FsResponse {
                        success: false,
                        data: None,
                        error: Some("Read permission denied".to_string()),
                    }
                } else {
                    match list_files(&request.path) {
                        Ok(files) => FsResponse {
                            success: true,
                            data: Some(json!(files)),
                            error: None,
                        },
                        Err(e) => FsResponse {
                            success: false,
                            data: None,
                            error: Some(format!("Failed to list files: {}", e)),
                        },
                    }
                }
            }
            _ => {
                log("Operation not supported");
                FsResponse {
                    success: false,
                    data: None,
                    error: Some("Operation not supported for request type".to_string()),
                }
            }
        };

        (
            serde_json::to_vec(&response).unwrap(),
            serde_json::to_vec(&state).unwrap(),
        )
    }

    fn handle_send(message: Json, state: Json) -> Json {
        log("Handling send");
        log(&format!("Message: {:?}", message));
        let state: State = serde_json::from_slice(&state).unwrap();
        let request: FsRequest = match serde_json::from_slice(&message) {
            Ok(req) => req,
            Err(_) => return serde_json::to_vec(&state).unwrap(),
        };

        // Handle operations that don't need responses
        match request.operation.as_str() {
            "write-file" => {
                log(&format!("Writing file: {}", request.path));
                if state.permissions.contains(&"write".to_string()) {
                    if let Some(content) = request.content {
                        log("Checks passed");
                        let _ = write_file(&request.path, &content);
                    }
                }
            }
            "create-dir" => {
                log(&format!("Creating directory: {}", request.path));
                if state.permissions.contains(&"write".to_string()) {
                    log("Checks passed");
                    let _ = create_dir(&request.path);
                }
            }
            "delete-file" => {
                log(&format!("Deleting file: {}", request.path));
                if state.permissions.contains(&"delete".to_string()) {
                    log("Checks passed");
                    let _ = delete_file(&request.path);
                }
            }
            "delete-dir" => {
                log(&format!("Deleting directory: {}", request.path));
                if state.permissions.contains(&"delete".to_string()) {
                    log("Checks passed");
                    let _ = delete_dir(&request.path);
                }
            }
            _ => {
                log("Operation not supported");
            }
        }

        serde_json::to_vec(&state).unwrap()
    }
}

bindings::export!(Component with_types_in bindings);
