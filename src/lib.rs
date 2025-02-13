mod bindings;

use bindings::exports::ntwk::theater::actor::Guest as ActorGuest;
use bindings::exports::ntwk::theater::message_server_client::Guest as MessageServerClient;
use bindings::ntwk::theater::filesystem::{read_file, write_file, list_files, create_dir, delete_file, delete_dir};
use serde::{Deserialize, Serialize};
use serde_json::json;

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
    fn init() -> Vec<u8> {
        let state = State {
            permissions: vec!["read".to_string()], // Default to read-only
        };
        serde_json::to_vec(&state).unwrap()
    }
}

impl MessageServerClient for Component {
    fn handle(message: Vec<u8>, state: Vec<u8>) -> (Vec<u8>, Vec<u8>) {
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

        let response = match request.operation.as_str() {
            "read-file" => {
                if !state.permissions.contains(&"read".to_string()) {
                    FsResponse {
                        success: false,
                        data: None,
                        error: Some("Read permission denied".to_string()),
                    }
                } else {
                    match read_file(&request.path) {
                        Ok(content) => {
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
            "write-file" => {
                if !state.permissions.contains(&"write".to_string()) {
                    FsResponse {
                        success: false,
                        data: None,
                        error: Some("Write permission denied".to_string()),
                    }
                } else {
                    match request.content {
                        Some(content) => match write_file(&request.path, &content) {
                            Ok(_) => FsResponse {
                                success: true,
                                data: None,
                                error: None,
                            },
                            Err(e) => FsResponse {
                                success: false,
                                data: None,
                                error: Some(format!("Failed to write file: {}", e)),
                            },
                        },
                        None => FsResponse {
                            success: false,
                            data: None,
                            error: Some("No content provided for write operation".to_string()),
                        },
                    }
                }
            }
            "list-files" => {
                if !state.permissions.contains(&"read".to_string()) {
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
            "create-dir" => {
                if !state.permissions.contains(&"write".to_string()) {
                    FsResponse {
                        success: false,
                        data: None,
                        error: Some("Write permission denied".to_string()),
                    }
                } else {
                    match create_dir(&request.path) {
                        Ok(_) => FsResponse {
                            success: true,
                            data: None,
                            error: None,
                        },
                        Err(e) => FsResponse {
                            success: false,
                            data: None,
                            error: Some(format!("Failed to create directory: {}", e)),
                        },
                    }
                }
            }
            "delete-file" => {
                if !state.permissions.contains(&"delete".to_string()) {
                    FsResponse {
                        success: false,
                        data: None,
                        error: Some("Delete permission denied".to_string()),
                    }
                } else {
                    match delete_file(&request.path) {
                        Ok(_) => FsResponse {
                            success: true,
                            data: None,
                            error: None,
                        },
                        Err(e) => FsResponse {
                            success: false,
                            data: None,
                            error: Some(format!("Failed to delete file: {}", e)),
                        },
                    }
                }
            }
            "delete-dir" => {
                if !state.permissions.contains(&"delete".to_string()) {
                    FsResponse {
                        success: false,
                        data: None,
                        error: Some("Delete permission denied".to_string()),
                    }
                } else {
                    match delete_dir(&request.path) {
                        Ok(_) => FsResponse {
                            success: true,
                            data: None,
                            error: None,
                        },
                        Err(e) => FsResponse {
                            success: false,
                            data: None,
                            error: Some(format!("Failed to delete directory: {}", e)),
                        },
                    }
                }
            }
            _ => FsResponse {
                success: false,
                data: None,
                error: Some(format!("Unknown operation: {}", request.operation)),
            },
        };

        (
            serde_json::to_vec(&response).unwrap(),
            serde_json::to_vec(&state).unwrap(),
        )
    }
}

bindings::export!(Component with_types_in bindings);