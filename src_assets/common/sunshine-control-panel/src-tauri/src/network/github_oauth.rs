use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::time::Duration;


const GITHUB_CLIENT_ID: &str = "Ov23liw1VYcNjvCjvAo2";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct AccessTokenRequest {
    client_id: String,
    device_code: String,
    grant_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AccessTokenSuccess {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AccessTokenError {
    pub error: String,
    pub error_description: String,
    pub error_uri: String,
}

fn create_client(proxy: Option<&str>) -> Result<Client, String> {
    let mut builder = Client::builder()
        .timeout(Duration::from_secs(60))
        .user_agent("GalRemote-App");
    
    if let Some(p) = proxy {
        if !p.trim().is_empty() {
            let reqwest_proxy = reqwest::Proxy::all(p)
                .map_err(|e| format!("Invalid proxy format: {}", e))?;
            builder = builder.proxy(reqwest_proxy);
        }
    }
    
    builder.build().map_err(|e| format!("Failed to build client: {}", e))
}

pub async fn request_device_code(proxy: Option<&str>) -> Result<DeviceCodeResponse, String> {
    let client = create_client(proxy)?;
    let resp = client
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .form(&[("client_id", GITHUB_CLIENT_ID), ("scope", "repo")])
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Bad status code: {}", resp.status()));
    }

    let code_resp: DeviceCodeResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(code_resp)
}

pub async fn poll_for_access_token(device_code: String, interval: u64, expires_in: u64, proxy: Option<&str>) -> Result<String, String> {
    let client = create_client(proxy).unwrap_or_else(|_| Client::new());
    let start_time = std::time::Instant::now();
    let duration_limit = Duration::from_secs(expires_in);
    
    // Safety cushion
    let mut current_interval = interval;
    if current_interval < 5 {
        current_interval = 5;
    }

    while start_time.elapsed() < duration_limit {
        tokio::time::sleep(Duration::from_secs(current_interval)).await;

        let req = AccessTokenRequest {
            client_id: GITHUB_CLIENT_ID.to_string(),
            device_code: device_code.clone(),
            grant_type: "urn:ietf:params:oauth:grant-type:device_code".to_string(),
        };

        let resp = client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .json(&req)
            .send()
            .await;

        match resp {
            Ok(response) => {
                if let Ok(text) = response.text().await {
                    // Try parsing as success first
                    if let Ok(success) = serde_json::from_str::<AccessTokenSuccess>(&text) {
                        return Ok(success.access_token);
                    }
                    
                    // Try parsing as error
                    if let Ok(err_resp) = serde_json::from_str::<AccessTokenError>(&text) {
                        if err_resp.error == "authorization_pending" {
                            // Keep waiting
                            continue;
                        } else if err_resp.error == "slow_down" {
                            current_interval += 5;
                            continue;
                        } else {
                            return Err(format!("GitHub error: {}", err_resp.error_description));
                        }
                    }
                    
                    log::warn!("Unknown response format from GitHub: {}", text);
                }
            }
            Err(e) => {
                log::warn!("Polling error: {}", e);
                // Keep trying until timeout
            }
        }
    }

    Err("Timeout waiting for authorization".into())
}

pub async fn setup_github_repository(token: &str, proxy: Option<&str>) -> Result<String, String> {
    let client = create_client(proxy).unwrap_or_else(|_| Client::new());
    let repo_name = "galremote-cloud-saves";
    
    // 1. Get User Profile to find out owner name
    let user_resp = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "GalRemote-App")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch user: {}", e))?;
        
    let user_json: serde_json::Value = user_resp.json().await.map_err(|e| e.to_string())?;
    let owner = user_json["login"].as_str().ok_or("Could not get username")?.to_string();

    // 2. Check if repo exists
    let repo_url = format!("https://api.github.com/repos/{}/{}", owner, repo_name);
    let check_resp = client
        .get(&repo_url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "GalRemote-App")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if check_resp.status().is_success() {
        // Repo exists
        return Ok(format!("{}/{}", owner, repo_name));
    }

    // 3. Create repo if it doesn't exist (Private)
    let create_body = serde_json::json!({
        "name": repo_name,
        "description": "Auto-generated repository for GalRemote save backups",
        "private": true,
        "auto_init": true
    });

    let create_resp = client
        .post("https://api.github.com/user/repos")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "GalRemote-App")
        .header("Accept", "application/vnd.github.v3+json")
        .json(&create_body)
        .send()
        .await
        .map_err(|e| format!("Failed to create repo request: {}", e))?;

    if !create_resp.status().is_success() {
        return Err(format!("Failed to create repo, status: {}", create_resp.status()));
    }

    Ok(format!("{}/{}", owner, repo_name))
}
