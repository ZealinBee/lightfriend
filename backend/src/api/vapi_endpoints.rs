
use axum::{
    Json,
    extract::State,
    response::Response,
};
use std::future::Future;
use std::sync::Arc;
use crate::AppState;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::error::Error;
use tracing::{error, info};


pub async fn vapi_server(
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    tracing::info!("Debug Payload Received: {:#?}", payload);
    println!("Payload: {:#?}",payload);
    Json(payload)
}

use crate::api::vapi_dtos::{MessageResponse, ServerResponse};

pub async fn handle_phone_call_event_print(Json(payload): Json<serde_json::Value>) -> Json<serde_json::Value> {
    match serde_json::from_value::<MessageResponse>(payload) {
        Ok(event) => {
            let phone_number = event.get_phone_number();
            let request_type = event.get_request_type();
            
            println!("Received call from: {:?}", phone_number);
            println!("Request type: {:?}", request_type);

            if let Some(tool_calls) = event.get_tool_calls() {
                println!("Tool Calls:");
                for tool_call in tool_calls {
                    println!("  - Tool Call ID: {}", tool_call.id);
                    println!("    Type: {}", tool_call.r#type);
                    println!("    Function: {}", tool_call.function.name);
                    println!("    Arguments: {:?}", tool_call.function.arguments);
                }
            }
            
            Json(json!({
                "status": "success",
                "message": "Phone number and request type extracted successfully",
                "phone_number": phone_number,
                "request_type": request_type
            }))
        }
        Err(e) => {
            eprintln!("Error parsing payload: {}", e);
            Json(json!({
                "status": "error",
                "message": "Invalid payload format",
                "error": format!("{:#?}", e)
            }))
        }
    }
}
   

pub async fn handle_tool_calls(event: &MessageResponse) -> ServerResponse {
    if let Some(tool_calls) = event.get_tool_calls() {
        for tool_call in tool_calls {
            match tool_call.function.name.as_str() {
                "perplexity-ask" => {
                    if let Some(arguments) = tool_call.function.arguments.as_object() {
                        let message = arguments.get("message").and_then(|v| v.as_str()).unwrap_or("");
                        match ask_perplexity(message).await {
                            Ok(result) => {
                                println!("Perplexity response: {}", result);
                            },
                            Err(e) => {
                                eprintln!("Error making Perplexity request: {}", e);
                            }
                        }
                    }
                },
                "system-command" => {
                    // Handle system commands here
                    println!("Received system command: {:?}", tool_call);
                },
                _ => {
                    println!("Unknown function type: {}", tool_call.function.name);
                }
            }
        }
    }
    ServerResponse {
        status: "success".to_string(),
        message: "Tool calls processed successfully".to_string(),
        data: None,
    }
}

pub async fn handle_assistant_request(event: &MessageResponse, state: &Arc<AppState>) -> Json<serde_json::Value> {
    println!("Entering handle_assistant_request");
    println!("Event: {:#?}", event);
    
    if let Some(phone_number) = event.get_phone_number() {
        println!("Found phone number: {}", phone_number);
        
        match state.user_repository.find_by_phone_number(&phone_number) {
            Ok(Some(user)) => {
                println!("User found for phone number: {}", phone_number);
                
                if let Some(nickname) = user.nickname {
                    println!("User has nickname: {}", nickname);
                    let response = json!({
                        "messageResponse": {
                            "assistantId": "d60f5e83-3d90-4604-9d7d-06cb5decdc36",
                            "assistantOverrides": {
                                "firstMessage": format!("Hello! {}", nickname),
                                "variableValues": {
                                    "name": nickname
                                }
                            }
                        }
                    });
                    println!("Returning response: {:#?}", response);
                    Json(response)
                } else {
                    println!("User does NOT have nickname");
                    let resp = json!({
                        "messageResponse": {
                            "assistantId": "d60f5e83-3d90-4604-9d7d-06cb5decdc36",
                            "assistantOverrides": {
                                "firstMessage": "Hello! {{name}}",
                                "variableValues": {
                                    "name": "nickname"
                                }
                            }
                        }
                    });
                    println!("Returning response: {:#?}", resp);
                    Json(resp)
                }
            },
            Ok(None) => {
                println!("No user found for phone number: {}", phone_number);
                let resp = json!({
                    "messageResponse": {
                        "assistantId": "d60f5e83-3d90-4604-9d7d-06cb5decdc36",
                        "assistantOverrides": {
                            "firstMessage": "Hello! {{name}}",
                            "variableValues": {
                                "name": "nickname"
                            }
                        }
                    }
                });
                println!("Returning response: {:#?}", resp);
                Json(resp)
            },
            Err(e) => {
                println!("Database error while finding user: {}", e);
                let resp = json!({
                    "messageResponse": {
                        "assistantId": "d60f5e83-3d90-4604-9d7d-06cb5decdc36",
                        "assistantOverrides": {
                            "firstMessage": "Hello! {{name}}",
                            "variableValues": {
                                "name": "nickname"
                            }
                        }
                    }
                });
                println!("Returning response: {:#?}", resp);
                Json(resp)
            }
        }
    } else {
        println!("No phone number found in event");
        let resp = json!({
            "messageResponse": {
                "assistantId": "d60f5e83-3d90-4604-9d7d-06cb5decdc36",
                "assistantOverrides": {
                    "firstMessage": "Hello! {{name}}",
                    "variableValues": {
                        "name": "nickname"
                    }
                }
            }
        });
        println!("Returning response: {:#?}", resp);
        Json(resp)
    }
}

pub async fn handle_status_update(event: &MessageResponse) -> ServerResponse {

    println!("Processing status update");

    // Add your status update handling logic here
    
    ServerResponse {
        status: "success".to_string(),
        message: "Status update received".to_string(),
        data: None,
    }
}

pub async fn handle_phone_call_event(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    match serde_json::from_value::<MessageResponse>(payload) {
        Ok(event) => {
            let request_type = event.get_request_type();
            
            println!("Received call from: {:?}", event.get_phone_number());
            println!("Request type: {:?}", request_type);

            match request_type.as_str() {
                //"tool-calls" => {
                 //   handle_tool_calls(&event).await
                //},
                "assistant-request" => {
                    println!("Calling handle_assistant_request for assistant request");
                    handle_assistant_request(&event, &state).await
                },
                //"status-update" => {
                 //   handle_status_update(&event).await
                //},
                _ => Json(json!({
                    "status": "error",
                    "message": "Unknown request type",
                    "data": null
                }))
            }
        }
        Err(e) => {
            eprintln!("Error parsing payload: {}", e);
            Json(json!({
                "status": "error",
                "message": "Error parsing payload",
                "data": null
            }))

        }
    }
}


pub async fn ask_perplexity(message: &str) -> Result<String, reqwest::Error> {
    let api_key = std::env::var("PERPLEXITY_API_KEY").expect("PERPLEXITY_API_KEY must be set");
    let client = reqwest::Client::new();
    
    let payload = json!({
        "model": "llama-3.1-sonar-small-128k-online",
        "messages": [
            {
                "role": "system",
                "content": "Be precise and concise."
            },
            {
                "role": "user",
                "content": message
            }
        ]
    });

    let response = client
        .post("https://api.perplexity.ai/chat/completions")
        .header("accept", "application/json")
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&payload)
        .send()
        .await?;

    let result = response.text().await?;
    println!("{}", result);
    Ok(result)
}
