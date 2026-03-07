// Tauri IPC commands for Galgame save management
use tauri::command;
use opendal;

use super::game::{Game, Snapshot, PlaySession, BackupMode, GameStatus};
use super::archive::{self, get_backup_dir};
use super::cloud::CloudBackend;
use super::config::{self, GalgameConfig, load_config, save_config};
use super::scanner::{SavePathScanner, SaveCandidate};

type CmdResult<T> = Result<T, String>;

fn normalize_game_name(name: &str) -> String {
    name.split_whitespace()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn find_game_index_by_name(games: &[Game], name: &str) -> Option<usize> {
    let normalized_target = normalize_game_name(name);
    games
        .iter()
        .position(|g| normalize_game_name(&g.name) == normalized_target)
}

#[command]
pub fn galgame_get_config() -> CmdResult<GalgameConfig> {
    load_config().map_err(|e| format!("Failed to load config: {}", e))
}

#[command]
pub fn galgame_save_config(config: GalgameConfig) -> CmdResult<()> {
    save_config(&config).map_err(|e| format!("Failed to save config: {}", e))
}

#[command]
pub fn galgame_add_game(mut game: Game, update: bool, old_name: Option<String>) -> CmdResult<()> {
    let mut cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;

    game.name = game.name.trim().to_string();
    if game.name.is_empty() {
        return Err("游戏名称不能为空".to_string());
    }

    // Handle cover image auto-backup
    if let Some(cover) = &game.cover_image {
        if !cover.is_empty() {
            let src_path = std::path::Path::new(cover);
            if src_path.exists() {
                let backup_dir = get_backup_dir().join(&game.name);
                if let Err(e) = std::fs::create_dir_all(&backup_dir) {
                    log::error!("Failed to create backup dir for cover: {}", e);
                } else {
                    if let Some(ext) = src_path.extension() {
                        let dest_path = backup_dir.join(format!("cover.{}", ext.to_string_lossy()));
                        // Check if source is already the destination (avoid copy onto self)
                        let is_same = src_path.canonicalize().ok() == dest_path.canonicalize().ok();
                        if !is_same {
                            match std::fs::copy(src_path, &dest_path) {
                                Ok(_) => {
                                    log::info!("Cover image copied to {:?}", dest_path);
                                    game.cover_image = Some(dest_path.to_string_lossy().to_string());
                                },
                                Err(e) => log::error!("Failed to copy cover image: {}", e),
                            }
                        }
                    }
                }
            }
        }
    }

    let target_index = if update {
        let old_name_trimmed = old_name
            .as_deref()
            .map(str::trim)
            .filter(|name| !name.is_empty());
        old_name_trimmed
            .and_then(|name| find_game_index_by_name(&cfg.games, name))
            .or_else(|| find_game_index_by_name(&cfg.games, &game.name))
    } else {
        None
    };

    if let Some(existing_index) = find_game_index_by_name(&cfg.games, &game.name) {
        if Some(existing_index) != target_index {
            return Err(format!("已存在同名游戏: {}", game.name));
        }
    }

    if let Some(index) = target_index {
        cfg.games[index] = game;
    } else {
        cfg.games.push(game);
    }

    save_config(&cfg).map_err(|e| format!("Failed to save config: {}", e))
}

#[command]
pub fn galgame_launch_game(app_handle: tauri::AppHandle, game_name: String) -> CmdResult<()> {
    use tauri::Emitter;

    let cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    let game = cfg.games.iter().find(|g| g.name == game_name)
        .ok_or_else(|| format!("Game not found: {}", game_name))?;

    if let Some(exe_path) = &game.exe_path {
        if exe_path.is_empty() {
             return Err("Launch path is not configured".to_string());
        }
        
        #[cfg(target_os = "windows")]
        {
             // Capture start time
             let start_time = chrono::Utc::now();

             let mut child = std::process::Command::new(exe_path)
                .spawn()
                .map_err(|e| format!("Failed to launch game: {}", e))?;
            
            let game_clone = game.clone();
            let device_id = cfg.device_id.clone();
            let app_handle = app_handle.clone();

            std::thread::spawn(move || {
                match child.wait() {
                    Ok(status) => {
                        log::info!("Game {} exited with status {}", game_clone.name, status);
                        let end_time = chrono::Utc::now();
                        let duration = end_time.signed_duration_since(start_time).num_seconds();

                        let mut current_game_state = game_clone.clone();

                        // Update playtime stats
                        if let Ok(mut current_cfg) = load_config() {
                            let updated = if let Some(g) = current_cfg.games.iter_mut().find(|g| g.name == game_clone.name) {
                                // Ensure stats are initialized if missing (default handle by serde but good to be safe)
                                g.total_play_time += duration.max(0) as u64; 
                                g.last_played = Some(end_time.timestamp());
                                g.play_history.push(PlaySession {
                                    start_time: start_time.timestamp(),
                                    duration_seconds: duration.max(0) as u64,
                                    device_id: device_id.clone(),
                                });
                                current_game_state = g.clone(); // Capture latest state for backup check
                                true
                            } else {
                                false
                            };

                            if updated {
                                if let Err(e) = save_config(&current_cfg) {
                                    log::error!("Failed to save playtime stats: {}", e);
                                } else {
                                    log::info!("Updated playtime for {}: +{}s", game_clone.name, duration);
                                    let _ = app_handle.emit("galgame-playtime-update", current_game_state.clone());
                                }
                            }
                        }

                        if current_game_state.backup_mode == BackupMode::OnGameExit || current_game_state.backup_mode == BackupMode::Both {
                             log::info!("Triggering auto-backup for {}", current_game_state.name);
                             let backup_dir = get_backup_dir();
                             match archive::create_snapshot(&current_game_state, &device_id, &backup_dir, "自动备份 (游戏退出)") {
                                 Ok(_) => {
                                     log::info!("Auto-backup successful");
                                     let _ = app_handle.emit("galgame-auto-backup", format!("{} 自动备份完成", current_game_state.name));
                                 },
                                 Err(e) => {
                                     log::error!("Auto-backup failed: {}", e);
                                     let _ = app_handle.emit("galgame-auto-backup-error", format!("{} 自动备份失败: {}", current_game_state.name, e));
                                 }
                             }
                        }
                    }
                    Err(e) => log::error!("Failed to wait on child process: {}", e),
                }
            });
        }
        #[cfg(not(target_os = "windows"))]
        {
             return Err("Launch not supported on this OS".to_string());
        }
        Ok(())
    } else {
        Err("Launch path not set".to_string())
    }
}

#[command]
pub fn galgame_delete_game(game_name: String) -> CmdResult<bool> {
    let mut cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    
    // Also delete the local backup directory
    let backup_dir = get_backup_dir().join(&game_name);
    if backup_dir.exists() {
        if let Err(e) = std::fs::remove_dir_all(&backup_dir) {
            log::warn!("Failed to delete backup dir for {}: {}", game_name, e);
            // We proceed to delete from config anyway, effectively "orphaning" the folder if it fails,
            // but usually it succeeds.
        } else {
            log::info!("Deleted backup dir for {}", game_name);
        }
    }

    config::remove_game(&mut cfg, &game_name).map_err(|e| format!("Failed to delete game: {}", e))
}

#[command]
pub fn galgame_list_games() -> CmdResult<Vec<Game>> {
    let mut cfg = load_config().map_err(|e| format!("Failed to load games: {}", e))?;
    let mut modified = false;

    // Auto-discover cover images from backup dir if missing in config
    let backup_root = get_backup_dir();
    // Auto-discover cover images from backup dir if missing in config
    for game in &mut cfg.games {
        let needs_cover = match &game.cover_image {
            Some(s) => s.is_empty(),
            None => true,
        };
        
        if needs_cover {
            let game_dir = backup_root.join(&game.name);
            if game_dir.exists() {
                // Check for cover.png/jpg/etc
                let extensions = ["png", "jpg", "jpeg", "webp"];
                for ext in extensions {
                    let cover_path = game_dir.join(format!("cover.{}", ext));
                    if cover_path.exists() {
                        game.cover_image = Some(cover_path.to_string_lossy().to_string());
                        modified = true;
                        break;
                    }
                }
            }
        }
    }

    if modified {
        let _ = save_config(&cfg);
    }

    Ok(cfg.games)
}

#[command]
pub fn galgame_create_snapshot(game_name: String, describe: String) -> CmdResult<Snapshot> {
    let cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    let game = cfg.games.iter().find(|g| g.name == game_name)
        .ok_or_else(|| format!("Game not found: {}", game_name))?;
    let backup_dir = get_backup_dir();
    archive::create_snapshot(game, &cfg.device_id, &backup_dir, &describe)
        .map_err(|e| format!("Failed to create snapshot: {}", e))
}

#[command]
pub fn galgame_restore_snapshot(game_name: String, snapshot_date: String) -> CmdResult<()> {
    let cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    let game = cfg.games.iter().find(|g| g.name == game_name)
        .ok_or_else(|| format!("Game not found: {}", game_name))?;
    let snapshots = archive::list_snapshots(&game_name)
        .map_err(|e| format!("Failed to list snapshots: {}", e))?;
    let snapshot = snapshots.iter().find(|s| s.date == snapshot_date)
        .ok_or_else(|| format!("Snapshot not found: {}", snapshot_date))?;
    archive::restore_snapshot(game, &cfg.device_id, snapshot, true)
        .map_err(|e| format!("Failed to restore snapshot: {}", e))
}

#[command]
pub fn galgame_list_snapshots(game_name: String) -> CmdResult<Vec<Snapshot>> {
    archive::list_snapshots(&game_name).map_err(|e| format!("Failed to list snapshots: {}", e))
}

#[command]
pub fn galgame_delete_snapshot(game_name: String, snapshot_date: String) -> CmdResult<()> {
    let snapshots = archive::list_snapshots(&game_name)
        .map_err(|e| format!("Failed to list snapshots: {}", e))?;
    let snapshot = snapshots.iter().find(|s| s.date == snapshot_date)
        .ok_or_else(|| format!("Snapshot not found: {}", snapshot_date))?;
    archive::delete_snapshot(snapshot).map_err(|e| format!("Failed to delete snapshot: {}", e))
}

#[command]
pub async fn galgame_check_cloud_connection(backend: CloudBackend) -> CmdResult<()> {
    let cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    backend.check_connection(&cfg.cloud_settings.root_path).await
        .map_err(|e| format!("Connection failed: {}", e))
}

#[command]
pub async fn galgame_sync_to_cloud(force: Option<String>) -> CmdResult<u32> {
    let cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    let mut total_count = 0;
    
    // 1. Sync files per game (Delta Sync)
    for game in &cfg.games {
        match super::cloud::sync_all_to_cloud(&cfg.cloud_settings.backend, &cfg.cloud_settings, game, &cfg.device_id, force.as_deref()).await {
            Ok(count) => total_count += count,
            Err(e) => {
                log::error!("Failed to sync game {}: {}", game.name, e);
                if e.to_string().contains("SYNC_CONFLICT") {
                    return Err(format!("SYNC_CONFLICT: Conflict detected for game: {}", game.name));
                } else {
                    return Err(format!("Failed to sync {}: {}", game.name, e));
                }
            }
        }
    }

    // 2. Sync config (Metadata, Status, PlayTime)
    sync_config_to_cloud(&cfg).await?;
    
    Ok(total_count)
}

#[command]
pub async fn galgame_sync_from_cloud(force: Option<String>) -> CmdResult<u32> {
    // Reload config in case it changed
    let mut cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    let mut total_count = 0;
    
    // 1. Sync files per game (Delta Sync)
    for game in &cfg.games {
        match super::cloud::sync_all_from_cloud(&cfg.cloud_settings.backend, &cfg.cloud_settings, game, &cfg.device_id, force.as_deref()).await {
            Ok(count) => total_count += count,
            Err(e) => {
                log::error!("Failed to sync from cloud for game {}: {}", game.name, e);
                if e.to_string().contains("SYNC_CONFLICT") {
                    return Err(format!("SYNC_CONFLICT: Conflict detected for game: {}", game.name));
                } else {
                    return Err(format!("Failed to download {}: {}", game.name, e));
                }
            }
        }
    }

    // 2. Sync config (Metadata, Status, PlayTime)
    if sync_config_from_cloud(&mut cfg).await? {
        save_config(&cfg).map_err(|e| format!("Failed to save merged config: {}", e))?;
    }
    
    Ok(total_count)
}

async fn sync_config_to_cloud(cfg: &GalgameConfig) -> CmdResult<()> {
    let backend = &cfg.cloud_settings.backend;
    if let CloudBackend::Disabled = backend {
        return Ok(());
    }

    // Serialize config (Sensitive data like passwords in CloudSettings are included, 
    // but the file is stored in the users private bucket/repo. This is acceptable for personal sync.)
    let json = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;

    let op = backend.get_operator(&cfg.cloud_settings.root_path)
        .map_err(|e| e.to_string())?;
    
    log::info!("Uploading galgame_config.json to cloud, size: {} bytes", json.len());
    
    op.write("galgame_config.json", json.as_bytes().to_vec())
        .await
        .map_err(|e| format!("Failed to upload config: {}", e))?;
        
    log::info!("Config upload successful");
        
    Ok(())
}

async fn sync_config_from_cloud(local_cfg: &mut GalgameConfig) -> CmdResult<bool> {
    let backend = &local_cfg.cloud_settings.backend;
    if let CloudBackend::Disabled = backend {
        return Ok(false);
    }
    
    let op = backend.get_operator(&local_cfg.cloud_settings.root_path)
        .map_err(|e| e.to_string())?;
        
    log::info!("Attempting to download galgame_config.json from cloud...");
    
    // Attempt to read directly instead of checking exists() to save an RPC and avoid race
    let bytes = match op.read("galgame_config.json").await {
        Ok(b) => b,
        Err(e) => {
            if e.kind() == opendal::ErrorKind::NotFound {
                log::info!("galgame_config.json not found on cloud, skipping config sync.");
                return Ok(false);
            }
            return Err(format!("Failed to download config: {}", e));
        }
    };
    
    log::info!("Downloaded config, size: {} bytes", bytes.len());
    let cloud_cfg_str = String::from_utf8(bytes.to_vec()).map_err(|e| format!("Invalid UTF-8 in config: {}", e))?;
    let cloud_cfg: GalgameConfig = serde_json::from_str(&cloud_cfg_str).map_err(|e| format!("Failed to parse cloud config: {}", e))?;
    
    log::info!("Parsed cloud config, games count: {}", cloud_cfg.games.len());
    
    // Merge Logic
    let mut modified = false;
    for cloud_game in cloud_cfg.games {
        if let Some(local_index) = find_game_index_by_name(&local_cfg.games, &cloud_game.name) {
            let local_game = &mut local_cfg.games[local_index];
            // Intelligent Play Time Merge
            // Calculate legacy time (time recorded before history tracking)
            let local_history_sum: u64 = local_game.play_history.iter().map(|s| s.duration_seconds).sum();
            let cloud_history_sum: u64 = cloud_game.play_history.iter().map(|s| s.duration_seconds).sum();
            
            let local_legacy = local_game.total_play_time.saturating_sub(local_history_sum);
            let cloud_legacy = cloud_game.total_play_time.saturating_sub(cloud_history_sum);
            let max_legacy = local_legacy.max(cloud_legacy);

            // Merge Play History (Union + duration reconciliation)
            let mut existing_keys: std::collections::HashMap<_, usize> = local_game
                .play_history
                .iter()
                .enumerate()
                .map(|(idx, p)| ((p.start_time, p.device_id.clone()), idx))
                .collect();

            let mut history_changed = false;
            for session in &cloud_game.play_history {
                let key = (session.start_time, session.device_id.clone());
                if let Some(existing_idx) = existing_keys.get(&key).copied() {
                    if session.duration_seconds > local_game.play_history[existing_idx].duration_seconds {
                        local_game.play_history[existing_idx].duration_seconds = session.duration_seconds;
                        history_changed = true;
                    }
                } else {
                    existing_keys.insert(key, local_game.play_history.len());
                    local_game.play_history.push(session.clone());
                    history_changed = true;
                }
            }
            
            // Recalculate Total Play Time
            let new_history_sum: u64 = local_game.play_history.iter().map(|s| s.duration_seconds).sum();
            let new_total = max_legacy + new_history_sum;
            
            if new_total != local_game.total_play_time {
                local_game.total_play_time = new_total;
                modified = true;
            }
            if history_changed {
                modified = true;
            }
            
            // Sort history by time just in case
            if history_changed {
                local_game.play_history.sort_by(|a, b| a.start_time.cmp(&b.start_time));
            }

            // Merge Last Played (Latest)
            match (local_game.last_played, cloud_game.last_played) {
                (Some(l), Some(c)) => {
                    if c > l {
                         local_game.last_played = Some(c);
                         modified = true;
                    }
                },
                (None, Some(c)) => {
                    local_game.last_played = Some(c);
                    modified = true;
                },
                _ => {}
            }

            // Merge Metadata if missing locally
            if local_game.description.is_none() && cloud_game.description.is_some() {
                local_game.description = cloud_game.description.clone();
                modified = true;
            }
            if local_game.developer.is_none() && cloud_game.developer.is_some() {
                local_game.developer = cloud_game.developer.clone();
                modified = true;
            }
            if local_game.release_date.is_none() && cloud_game.release_date.is_some() {
                local_game.release_date = cloud_game.release_date.clone();
                modified = true;
            }
            
            // Merge Status
            let merged_status = merge_game_status(
                &local_game.status,
                &cloud_game.status,
                local_game.last_played,
                cloud_game.last_played,
            );
            if merged_status != local_game.status {
                local_game.status = merged_status;
                modified = true;
            }
            
        } else {
             // Game exists in cloud but not local. Add it!
             // Note: Paths will be invalid on this machine, but metadata is valuable.
             // User can fix paths.
             let new_game = cloud_game;
             // Clear paths to avoid confusion? Or keep them as reference?
             // Keep them.
             local_cfg.games.push(new_game);
             modified = true;
        }
    }
    
    Ok(modified)
}

fn merge_game_status(
    local_status: &GameStatus,
    cloud_status: &GameStatus,
    local_last_played: Option<i64>,
    cloud_last_played: Option<i64>,
) -> GameStatus {
    use GameStatus::*;

    if local_status == cloud_status {
        return local_status.clone();
    }

    if *local_status == Finished || *cloud_status == Finished {
        return Finished;
    }

    if *local_status == NotStarted {
        return cloud_status.clone();
    }

    if *cloud_status == NotStarted {
        return local_status.clone();
    }

    match (local_status, cloud_status) {
        (Playing, Shelved) | (Shelved, Playing) => {
            match (local_last_played, cloud_last_played) {
                (Some(local_time), Some(cloud_time)) if cloud_time > local_time => cloud_status.clone(),
                (Some(_), Some(_)) => local_status.clone(),
                (None, Some(_)) => cloud_status.clone(),
                _ => local_status.clone(),
            }
        }
        _ => local_status.clone(),
    }
}

#[command]
pub async fn galgame_delete_cloud_game(game_name: String) -> CmdResult<()> {
    let cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    // game_name is the folder name in root
    super::cloud::delete_directory(&cfg.cloud_settings.backend, &cfg.cloud_settings, &game_name).await
        .map_err(|e| format!("Failed to delete cloud game: {}", e))
}

#[command]
pub fn galgame_scan_save_paths() -> CmdResult<Vec<SaveCandidate>> {
    let locations = SavePathScanner::get_common_save_locations();
    let mut all_candidates = Vec::new();
    for location in &locations {
        let candidates = SavePathScanner::scan_directory(location, 3);
        all_candidates.extend(candidates);
    }
    all_candidates.sort_by(|a, b| a.path.cmp(&b.path));
    all_candidates.dedup_by(|a, b| a.path == b.path);
    all_candidates.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
    Ok(all_candidates)
}

#[command]
pub fn galgame_get_common_locations() -> CmdResult<Vec<String>> {
    let locations = SavePathScanner::get_common_save_locations();
    Ok(locations.into_iter().map(|p| p.to_string_lossy().to_string()).collect())
}

#[command]
pub fn galgame_open_backup_folder(game_name: String) -> CmdResult<()> {
    let backup_dir = get_backup_dir().join(&game_name);
    if !backup_dir.exists() {
        return Err("Backup folder does not exist".to_string());
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer").arg(&backup_dir).spawn().ok();
    }
    Ok(())
}

#[command]
pub fn galgame_open_save_folder(path: String) -> CmdResult<()> {
    let resolved = archive::resolve_path_variables(&path);
    let p = std::path::PathBuf::from(&resolved);
    if !p.exists() {
        return Err("Save folder does not exist".to_string());
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer").arg(&p).spawn().ok();
    }
    Ok(())
}

#[command]
pub async fn galgame_sync_clipboard_to_cloud(text: String) -> CmdResult<()> {
    let cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    let backend = cfg.cloud_settings.backend;
    
    match backend {
        CloudBackend::Disabled => Err("Cloud sync is disabled".to_string()),
        _ => backend.upload_clipboard(&cfg.cloud_settings.root_path, &text).await.map_err(|e| format!("Upload failed: {}", e))
    }
}

#[command]
pub async fn galgame_sync_clipboard_from_cloud() -> CmdResult<String> {
    let cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    let backend = cfg.cloud_settings.backend;
    
    match backend {
        CloudBackend::Disabled => Err("Cloud sync is disabled".to_string()),
        _ => backend.download_clipboard(&cfg.cloud_settings.root_path).await.map_err(|e| format!("Download failed: {}", e))
    }
}


#[command]
pub async fn galgame_search_metadata(keyword: String) -> CmdResult<Vec<super::scraper::VndbResult>> {
    super::scraper::search_vndb(&keyword).await.map_err(|e| e)
}

#[command]
pub async fn galgame_apply_metadata(game_name: String, data: super::scraper::VndbResult) -> CmdResult<()> {
     let mut cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
     
     if let Some(game) = cfg.games.iter_mut().find(|g| g.name == game_name) {
         game.description = data.description;
         game.developer = data.developer;
         game.release_date = data.release_date;

         // Handle cover download
         if let Some(url) = data.cover_url {
             if !url.is_empty() {
                 let config_path = config::get_config_path();
                 let covers_dir = config_path.parent().unwrap_or(std::path::Path::new("."))
                     .join("covers");
                 // Basic sanitize filename (game name might have bad chars)
                 // But game_name is usually safe? If contains :, /, etc.
                 // We should use UUID or data.id + safe name.
                 // Here we use game_name, trusting it's somewhat safe or OS handles it (if not valid path).
                 // Better to sanitize.
                 let safe_game_name: String = game_name.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect();
                 
                 let ext = if url.to_lowercase().ends_with(".png") { "png" } else { "jpg" };
                 let filename = format!("{}_{}.{}", safe_game_name, data.id, ext);
                 let target_path = covers_dir.join(filename);
                 
                 match super::scraper::download_cover(&url, &target_path).await {
                     Ok(_) => {
                         game.cover_image = Some(target_path.to_string_lossy().to_string());
                         log::info!("Downloaded cover to {:?}", target_path);
                     },
                     Err(e) => {
                         log::error!("Failed to download cover: {}", e);
                         return Err(format!("Failed to download cover: {}", e));
                     }
                 }
             }
         }
         
         config::save_config(&cfg).map_err(|e| format!("Failed to save config: {}", e))?;
         Ok(())
     } else {
         Err(format!("Game not found: {}", game_name))
     }
}

#[command]
pub async fn galgame_github_oauth_request() -> CmdResult<crate::network::github_oauth::DeviceCodeResponse> {
    crate::network::github_oauth::request_device_code()
        .await
        .map_err(|e| format!("Request failed: {}", e))
}

#[command]
pub async fn galgame_github_oauth_poll(device_code: String, interval: u64, expires_in: u64) -> CmdResult<String> {
    crate::network::github_oauth::poll_for_access_token(device_code, interval, expires_in)
        .await
        .map_err(|e| format!("Poll failed: {}", e))
}

#[command]
pub async fn galgame_github_setup_repo(token: String) -> CmdResult<String> {
    crate::network::github_oauth::setup_github_repository(&token)
        .await
        .map_err(|e| format!("Setup failed: {}", e))
}
