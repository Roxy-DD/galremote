use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::path::Path;
use std::fs;
use std::io::Write;

// ── 统一搜刮结果 (对齐 Vnite GameMetadata) ──

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetadataResult {
    pub id: String,
    pub title: String,
    pub original_title: Option<String>,
    pub cover_url: Option<String>,
    pub description: Option<String>,
    pub developer: Option<String>,
    /// 开发商列表
    #[serde(default)]
    pub developers: Vec<String>,
    /// 发行商列表
    #[serde(default)]
    pub publishers: Vec<String>,
    pub release_date: Option<String>,
    /// 标签列表
    #[serde(default)]
    pub tags: Vec<String>,
    /// 类型列表
    #[serde(default)]
    pub genres: Vec<String>,
    /// 平台列表
    #[serde(default)]
    pub platforms: Vec<String>,
    /// 评分
    #[serde(default)]
    pub rating: Option<f32>,
    pub source: Option<String>,
    /// 是否包含 NSFW 内容
    #[serde(default)]
    pub nsfw: bool,
}

// ── VNDB ──

#[derive(Serialize)]
struct VndbQuery {
    filters: serde_json::Value,
    fields: String,
    results: u32,
}

pub async fn search_vndb(query: &str) -> Result<Vec<MetadataResult>, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Client build error: {}", e))?;

    let filter = serde_json::json!(["search", "=", query]);

    let body = VndbQuery {
        filters: filter,
        fields: "id, title, image.url, description, released, developers.name, tags.name, tags.rating, rating, platforms".to_string(),
        results: 15,
    };

    let resp = client.post("https://api.vndb.org/kana/vn")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("VNDB network error: {}", e))?;

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
            let cover_url = item["image"]["url"].as_str().map(|s| s.to_string());
            let description = item["description"].as_str().map(|s| {
                // Strip VNDB BBCode formatting
                let re = regex::Regex::new(r"\[/?[a-zA-Z]+\]").unwrap_or_else(|_| regex::Regex::new(r"$^").unwrap());
                re.replace_all(s, "").to_string()
            });
            let release_date = item["released"].as_str().map(|s| s.to_string());

            // Developers list
            let developers: Vec<String> = item["developers"].as_array()
                .map(|arr| arr.iter()
                    .filter_map(|dev| dev["name"].as_str().map(|s| s.to_string()))
                    .collect())
                .unwrap_or_default();

            let developer = developers.first().cloned();

            // Tags (sorted by rating, top 15)
            let tags: Vec<String> = item["tags"].as_array()
                .map(|arr| {
                    let mut tag_pairs: Vec<(&str, f64)> = arr.iter()
                        .filter_map(|t| {
                            let name = t["name"].as_str()?;
                            let rating = t["rating"].as_f64().unwrap_or(0.0);
                            Some((name, rating))
                        })
                        .collect();
                    tag_pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                    tag_pairs.into_iter().take(15).map(|(name, _)| name.to_string()).collect()
                })
                .unwrap_or_default();

            // Rating
            let rating = item["rating"].as_f64().map(|r| r as f32);

            // Platforms
            let platforms: Vec<String> = item["platforms"].as_array()
                .map(|arr| arr.iter().filter_map(|p| p.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();

            // NSFW check based on tags
            let nsfw = tags.iter().any(|t| t.to_lowercase().contains("sexual content") || t.to_lowercase().contains("nude"));

            results.push(MetadataResult {
                id,
                title,
                original_title: None,
                cover_url,
                description,
                developer,
                developers,
                publishers: Vec::new(),
                release_date,
                tags,
                genres: Vec::new(),
                platforms,
                rating,
                source: Some("VNDB".to_string()),
                nsfw,
            });
        }
    }
    Ok(results)
}

// ── Bangumi ──

#[derive(Debug, Deserialize)]
struct BangumiSearchResponse {
    #[serde(default)]
    data: Vec<BangumiItem>,
}

#[derive(Debug, Deserialize)]
struct BangumiItem {
    id: u64,
    name: Option<String>,
    name_cn: Option<String>,
    summary: Option<String>,
    date: Option<String>,
    images: Option<BangumiImages>,
    #[serde(default)]
    rating: Option<BangumiRating>,
    #[serde(default)]
    tags: Vec<BangumiTag>,
}

#[derive(Debug, Deserialize)]
struct BangumiImages {
    large: Option<String>,
    common: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BangumiRating {
    score: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct BangumiTag {
    name: Option<String>,
}

pub async fn search_bangumi(query: &str) -> Result<Vec<MetadataResult>, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Client build error: {}", e))?;

    let endpoint = "https://api.bgm.tv/v0/search/subjects";

    let body = serde_json::json!({
        "keyword": query,
        "filter": { "type": [4] }
    });

    let resp = client
        .post(endpoint)
        .header("User-Agent", "sunshine-gui/1.1 (+https://github.com/qiin2333/sunshine)")
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Bangumi request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Bangumi API error: {} - {}", status, body));
    }

    let parsed: BangumiSearchResponse = resp
        .json()
        .await
        .map_err(|e| format!("Bangumi parse failed: {}", e))?;

    let mut out = Vec::new();
    for item in parsed.data {
        let title = item.name_cn.clone().or(item.name.clone()).unwrap_or_else(|| "Unknown".to_string());
        let original = item.name.as_ref().filter(|n| !n.is_empty()).cloned();
        let cover = item.images.as_ref().and_then(|img| img.large.clone().or(img.common.clone()));
        let rating = item.rating.as_ref().and_then(|r| r.score).map(|s| s as f32);
        let tags: Vec<String> = item.tags.iter().filter_map(|t| t.name.clone()).collect();

        out.push(MetadataResult {
            id: format!("bgm-{}", item.id),
            title,
            original_title: original,
            cover_url: cover,
            description: item.summary,
            developer: None,
            developers: Vec::new(),
            publishers: Vec::new(),
            release_date: item.date,
            tags,
            genres: Vec::new(),
            platforms: Vec::new(),
            rating,
            source: Some("Bangumi".to_string()),
            nsfw: false,
        });
    }

    Ok(out)
}

// ── Steam ──

#[derive(Debug, Deserialize)]
struct SteamSearchResponse {
    #[serde(default)]
    items: Vec<SteamSearchItem>,
}

#[derive(Debug, Deserialize)]
struct SteamSearchItem {
    id: u64,
    name: Option<String>,
    tiny_image: Option<String>,
}

/// Steam app detail (from appdetails API)
#[derive(Debug, Deserialize)]
struct SteamAppDetailWrapper {
    #[serde(default)]
    success: bool,
    data: Option<SteamAppDetail>,
}

#[derive(Debug, Deserialize)]
struct SteamAppDetail {
    steam_appid: Option<u64>,
    name: Option<String>,
    short_description: Option<String>,
    header_image: Option<String>,
    developers: Option<Vec<String>>,
    publishers: Option<Vec<String>>,
    release_date: Option<SteamReleaseDate>,
    genres: Option<Vec<SteamGenre>>,
    metacritic: Option<SteamMetacritic>,
}

#[derive(Debug, Deserialize)]
struct SteamReleaseDate {
    date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SteamGenre {
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SteamMetacritic {
    score: Option<i32>,
}

pub async fn search_steam_store(query: &str) -> Result<Vec<MetadataResult>, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Client build error: {}", e))?;

    let endpoint = "https://store.steampowered.com/api/storesearch";

    let resp = client
        .get(endpoint)
        .query(&[("term", query), ("l", "schinese"), ("cc", "cn")])
        .send()
        .await
        .map_err(|e| format!("Steam request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Steam API error: {} - {}", status, body));
    }

    let parsed: SteamSearchResponse = resp
        .json()
        .await
        .map_err(|e| format!("Steam parse failed: {}", e))?;

    let mut out = Vec::new();
    for item in parsed.items.into_iter().take(10) {
        let title = item.name.unwrap_or_else(|| "Unknown".to_string());
        let app_id = item.id;

        // Try to fetch app details for richer data
        let detail = get_steam_app_detail(&client, app_id).await.ok().flatten();

        let (description, developers, publishers, release_date, genres, rating, cover) = match detail {
            Some(d) => {
                let devs = d.developers.unwrap_or_default();
                let pubs = d.publishers.unwrap_or_default();
                let rd = d.release_date.and_then(|r| r.date);
                let g: Vec<String> = d.genres.unwrap_or_default().iter()
                    .filter_map(|g| g.description.clone())
                    .collect();
                let r = d.metacritic.and_then(|m| m.score).map(|s| s as f32);
                let cover = d.header_image.or(item.tiny_image);
                (d.short_description, devs, pubs, rd, g, r, cover)
            }
            None => (None, Vec::new(), Vec::new(), None, Vec::new(), None, item.tiny_image),
        };

        out.push(MetadataResult {
            id: format!("steam-{}", app_id),
            title,
            original_title: None,
            cover_url: cover,
            description,
            developer: developers.first().cloned(),
            developers,
            publishers,
            release_date,
            tags: Vec::new(),
            genres,
            platforms: Vec::new(),
            rating,
            source: Some("Steam".to_string()),
            nsfw: false,
        });
    }

    Ok(out)
}

async fn get_steam_app_detail(client: &Client, app_id: u64) -> Result<Option<SteamAppDetail>, String> {
    let url = format!("https://store.steampowered.com/api/appdetails?appids={}&l=schinese&cc=cn", app_id);
    let resp = client.get(&url)
        .send()
        .await
        .map_err(|e| format!("Steam detail error: {}", e))?;

    if !resp.status().is_success() {
        return Ok(None);
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
    let key = app_id.to_string();
    if let Some(wrapper) = json.get(&key) {
        if let Ok(detail) = serde_json::from_value::<SteamAppDetailWrapper>(wrapper.clone()) {
            if detail.success {
                return Ok(detail.data);
            }
        }
    }
    Ok(None)
}

// ── YMGal (月幕 Galgame) ──

#[derive(Debug, Deserialize)]
struct YMGalTokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct YMGalApiResponse<T> {
    success: bool,
    #[serde(default)]
    msg: Option<String>,
    data: Option<T>,
}

#[derive(Debug, Deserialize)]
struct YMGalSearchData {
    #[serde(default)]
    result: Vec<YMGalSearchItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YMGalSearchItem {
    id: u64,
    name: Option<String>,
    chinese_name: Option<String>,
    release_date: Option<String>,
    #[serde(default)]
    org_name: Option<String>,
    #[serde(default)]
    main_img: Option<String>,
}

#[derive(Debug, Deserialize)]
struct YMGalDetailData {
    game: YMGalGameDetail,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YMGalGameDetail {
    #[serde(default)]
    name: String,
    #[serde(default)]
    chinese_name: Option<String>,
    #[serde(default)]
    introduction: Option<String>,
    #[serde(default)]
    release_date: Option<String>,
    #[serde(default)]
    developer_id: Option<u64>,
    #[serde(default)]
    main_img: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
}

async fn ymgal_get_token(client: &Client) -> Result<String, String> {
    let url = "https://www.ymgal.games/oauth/token";
    let params = [
        ("grant_type", "client_credentials"),
        ("client_id", "ymgal"),
        ("client_secret", "luna0327"),
        ("scope", "public"),
    ];

    let resp = client.get(url)
        .query(&params)
        .send()
        .await
        .map_err(|e| format!("YMGal token error: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("YMGal token HTTP {}", resp.status()));
    }

    let token: YMGalTokenResponse = resp.json().await
        .map_err(|e| format!("YMGal token parse error: {}", e))?;
    Ok(token.access_token)
}

pub async fn search_ymgal(query: &str) -> Result<Vec<MetadataResult>, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Client build error: {}", e))?;

    let token = ymgal_get_token(&client).await?;

    let resp = client.get("https://www.ymgal.games/open/archive/search-game")
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/json;charset=utf-8")
        .header("version", "1")
        .query(&[
            ("mode", "list"),
            ("keyword", query),
            ("pageNum", "1"),
            ("pageSize", "15"),
        ])
        .send()
        .await
        .map_err(|e| format!("YMGal search error: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("YMGal API error: {} - {}", status, body));
    }

    let api_resp: YMGalApiResponse<YMGalSearchData> = resp.json().await
        .map_err(|e| format!("YMGal parse error: {}", e))?;

    if !api_resp.success {
        return Err(format!("YMGal API failed: {}", api_resp.msg.unwrap_or_default()));
    }

    let data = api_resp.data.unwrap_or(YMGalSearchData { result: Vec::new() });

    let mut out = Vec::new();
    for item in data.result.into_iter().take(15) {
        let title = item.chinese_name.clone()
            .or(item.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());
        let original = item.name.clone();
        let developer = item.org_name.clone();

        out.push(MetadataResult {
            id: format!("ymgal-{}", item.id),
            title,
            original_title: original,
            cover_url: item.main_img,
            description: None,
            developer: developer.clone(),
            developers: developer.map(|d| vec![d]).unwrap_or_default(),
            publishers: Vec::new(),
            release_date: item.release_date,
            tags: Vec::new(),
            genres: Vec::new(),
            platforms: Vec::new(),
            rating: None,
            source: Some("YMGal".to_string()),
            nsfw: false,
        });
    }

    Ok(out)
}

// ── 多源聚合搜索 ──

pub async fn search_metadata_multi(query: &str, source: &str) -> Result<Vec<MetadataResult>, String> {
    let keyword = query.trim();
    if keyword.is_empty() {
        return Ok(Vec::new());
    }

    match source {
        "vndb" => search_vndb(keyword).await,
        "steam" => search_steam_store(keyword).await,
        "bangumi" => search_bangumi(keyword).await,
        "ymgal" => search_ymgal(keyword).await,
        "all" | _ => {
            let mut merged: Vec<MetadataResult> = Vec::new();

            // Run all searches, ignore individual failures
            let vndb_results = search_vndb(keyword).await.unwrap_or_default();
            merged.extend(vndb_results);

            let bangumi_results = search_bangumi(keyword).await.unwrap_or_default();
            merged.extend(bangumi_results);

            let steam_results = search_steam_store(keyword).await.unwrap_or_default();
            merged.extend(steam_results);

            let ymgal_results = search_ymgal(keyword).await.unwrap_or_default();
            merged.extend(ymgal_results);

            let normalized_keyword = keyword.to_lowercase();
            merged.sort_by(|a, b| {
                let a_title = a.title.to_lowercase();
                let b_title = b.title.to_lowercase();

                let a_exact = (a_title == normalized_keyword) as i32;
                let b_exact = (b_title == normalized_keyword) as i32;
                if a_exact != b_exact {
                    return b_exact.cmp(&a_exact);
                }

                let source_rank = |s: &Option<String>| match s.as_deref() {
                    Some("VNDB") => 4,
                    Some("Bangumi") => 3,
                    Some("YMGal") => 2,
                    Some("Steam") => 1,
                    _ => 0,
                };
                source_rank(&b.source).cmp(&source_rank(&a.source))
            });

            merged.truncate(30);
            Ok(merged)
        }
    }
}

// ── 封面下载 ──

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
