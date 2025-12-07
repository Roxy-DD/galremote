use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::path::Path;
use std::fs;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VndbResult {
    pub id: String,
    pub title: String,
    pub original_title: Option<String>,
    pub cover_url: Option<String>,
    pub description: Option<String>,
    pub developer: Option<String>,
    pub release_date: Option<String>,
}

#[derive(Serialize)]
struct VndbQuery {
    filters: serde_json::Value,
    fields: String,
    results: u32,
}

pub async fn search_vndb(query: &str) -> Result<Vec<VndbResult>, String> {
    let client = Client::new();
    let filter = serde_json::json!(["search", "=", query]);
    
    // VNDB Kana API request
    let body = VndbQuery {
        filters: filter,
        // Note: developers is an array, we request name of the first one (or all)
        fields: "id, title, image.url, description, released, developers.name".to_string(),
        results: 10,
    };

    let resp = client.post("https://api.vndb.org/kana/vn")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let error_text = resp.text().await.unwrap_or_default();
        log::error!("VNDB API Error {}: {}", status, error_text);
        return Err(format!("VNDB API error: {} - {}", status, error_text));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
    
    let mut results = Vec::new();
    if let Some(items) = json["results"].as_array() {
        for item in items {
            let id = item["id"].as_str().unwrap_or_default().to_string();
            let title = item["title"].as_str().unwrap_or_default().to_string();
            let original_title = None; // Field 'original' removed from query
            let cover_url = item["image"]["url"].as_str().map(|s| s.to_string());
            let description = item["description"].as_str().map(|s| s.to_string());
            let release_date = item["released"].as_str().map(|s| s.to_string());
            
            let developer = item["developers"].as_array()
                .and_then(|arr| arr.first())
                .and_then(|dev| dev["name"].as_str())
                .map(|s| s.to_string());

            results.push(VndbResult {
                id, title, original_title, cover_url, description, developer, release_date
            });
        }
    }
    Ok(results)
}

/// Download cover image ensuring it is saved locally
pub async fn download_cover(url: &str, target_path: &Path) -> Result<(), String> {
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
    }

    let response = reqwest::get(url).await.map_err(|e| format!("Download failed: {}", e))?;
    let bytes = response.bytes().await.map_err(|e| format!("Failed to read bytes: {}", e))?;

    let mut file = fs::File::create(target_path).map_err(|e| format!("Failed to create file: {}", e))?;
    file.write_all(&bytes).map_err(|e| format!("Failed to write file: {}", e))?;
    
    Ok(())
}
