use std::process::{Command, Stdio};

async fn start_browser_server() -> Result<(), String> {
    let client = reqwest::Client::new();

    let server_is_running = client
        .get("http://localhost:3333/health")
        .send()
        .await
        .is_ok();

    if server_is_running {
        return Ok(());
    }

    Command::new("node")
        .arg("../scripts/browser_server.cjs")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| error.to_string())?;

    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    Ok(())
}

#[tauri::command]
pub async fn browser_open(url: String) -> Result<String, String> {
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return Err("Only http and https URLs are allowed.".to_string());
    }

    start_browser_server().await?;

    let client = reqwest::Client::new();

    let response = client
        .post("http://localhost:3333/open")
        .json(&serde_json::json!({
            "url": url
        }))
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        let error_text = response.text().await.map_err(|error| error.to_string())?;
        return Err(error_text);
    }

    Ok(format!("Opened {} successfully.", url))
}

#[tauri::command]
pub async fn search_google(query: String) -> Result<String, String> {
    let encoded_query = urlencoding::encode(&query);
    let url = format!("https://www.google.com/search?q={}", encoded_query);
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return Err("Only http and https URLs are allowed.".to_string());
    }

    start_browser_server().await?;

    let client = reqwest::Client::new();

    let response = client
        .post("http://localhost:3333/open")
        .json(&serde_json::json!({
            "url": url
        }))
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        let error_text = response.text().await.map_err(|error| error.to_string())?;
        return Err(error_text);
    }

    Ok(format!("Searched Google for {}", query))
}

#[tauri::command]
pub async fn browser_open_and_read(url: String) -> Result<String, String> {
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return Err("Only http and https URLs are allowed.".to_string());
    }

    start_browser_server().await?;

    let client = reqwest::Client::new();

    let response = client
        .post("http://localhost:3333/open")
        .json(&serde_json::json!({
            "url": url
        }))
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        let error_text = response.text().await.map_err(|error| error.to_string())?;
        return Err(error_text);
    }

    let page_result = response.text().await.map_err(|error| error.to_string())?;

    Ok(page_result)
}

pub async fn browser_close() -> Result<String, String> {
    let client = reqwest::Client::new();

    let response = client
        .post("http://localhost:3333/close")
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        let error_text = response.text().await.map_err(|error| error.to_string())?;
        return Err(error_text);
    }

    Ok("Browser closed.".to_string())
}