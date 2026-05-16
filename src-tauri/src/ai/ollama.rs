use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Emitter};

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
}

#[derive(Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OllamaResponse {
    message: OllamaResponseMessage,
}

#[derive(Deserialize)]
struct OllamaResponseMessage {
    content: String,
}

#[derive(Deserialize)]
struct AgentResponse {
    #[serde(rename = "type")]
    response_type: String,
    tool: Option<String>,
    args: Option<Value>,
    answer: Option<String>,
}

async fn call_ollama(prompt: String) -> Result<String, String> {
    let system_prompt = include_str!("prompts/agent_rule.txt");

    let request_body = OllamaRequest {
        model: "qwen2.5-coder:7b".to_string(),
        messages: vec![
            OllamaMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            OllamaMessage {
                role: "user".to_string(),
                content: prompt,
            },
        ],
        stream: false,
    };

    let client = reqwest::Client::new();

    let response = client
        .post("http://localhost:11434/api/chat")
        .json(&request_body)
        .send()
        .await
        .map_err(|error| error.to_string())?;

    let ollama_response: OllamaResponse = response
        .json()
        .await
        .map_err(|error| error.to_string())?;

    Ok(ollama_response.message.content)
}

fn clean_ai_json(content: &str) -> String {
    content
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim()
        .to_string()
}

#[tauri::command]
pub async fn ask_ollama(app: AppHandle, prompt: String) -> Result<String, String> {
    let mut current_prompt = prompt.clone();
    let max_steps = 5;

    for step in 0..max_steps {
        app.emit("agent-log", format!("Step {}: Asking Ollama...\nPrompt: {}", step + 1, current_prompt))
            .map_err(|error| error.to_string())?;

        let ai_content:String = call_ollama(current_prompt.clone()).await?;

        app.emit("agent-log", format!("AI: {}", ai_content))
            .map_err(|error| error.to_string())?;

        let cleaned_content = clean_ai_json(&ai_content);

        let agent_response_result: Result<AgentResponse, _> =
            serde_json::from_str(&cleaned_content);

        match agent_response_result {
            Ok(agent_response) => {
                match agent_response.response_type.as_str() {
                    "final" => {
                        return Ok(agent_response.answer.unwrap_or(ai_content));
                    }

                    "tool_call" => {
                        let tool_name = agent_response
                            .tool
                            .ok_or("Missing tool name".to_string())?;

                        let args = agent_response
                            .args
                            .ok_or("Missing tool args".to_string())?;

                        let tool_result = match tool_name.as_str() {
                            "browser_open" => {
                                let url = args
                                    .get("url")
                                    .and_then(|value| value.as_str())
                                    .ok_or("Missing url".to_string())?;

                                crate::tools::browser_tools::browser_open(url.to_string()).await?
                            }

                            "search_google" => {
                                let query = args
                                    .get("query")
                                    .and_then(|value| value.as_str())
                                    .ok_or("Missing query".to_string())?;

                                crate::tools::browser_tools::search_google(query.to_string()).await?
                            }

                            "browser_open_and_read" => {
                                let url = args
                                    .get("url")
                                    .and_then(|value| value.as_str())
                                    .ok_or("Missing url".to_string())?;

                                crate::tools::browser_tools::browser_open_and_read(url.to_string()).await?
                            }

                            _ => {
                                return Err(format!("Unknown tool: {}", tool_name));
                            }
                        };

                        app.emit("agent-log", format!("Tool result: {}", tool_result))
                            .map_err(|error| error.to_string())?;

                        current_prompt = format!(
                            "Original user request:\n{}\n\nTool used:\n{}\n\nTool result:\n{}\n\nUse this tool result and continue. If you have enough information, return final JSON.",
                            prompt,
                            tool_name,
                            tool_result
                        );
                    }

                    _ => {
                        return Ok(ai_content);
                    }
                }
            }

            Err(_) => {
                current_prompt = format!(
                    "Your response was invalid. Return raw JSON only.\n\
                    Valid formats are:\n\
                    {{\"type\":\"tool_call\",\"tool\":\"browser_open_and_read\",\"args\":{{\"url\":\"https://example.com\"}}}}\n\
                    or\n\
                    {{\"type\":\"final\",\"answer\":\"your answer here\"}}\n\n\
                    Previous response:\n{}",
                    ai_content
                );

                continue;
            }
        }
    }

    Err("Agent stopped because it reached the maximum number of steps.".to_string())
}