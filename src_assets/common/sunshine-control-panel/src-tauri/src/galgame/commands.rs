// Tauri IPC commands for Galgame save management
use opendal;
use tauri::command;

use super::archive::{self, get_backup_dir};
use super::cloud::CloudBackend;
use super::config::{self, GalgameConfig, load_config, save_config};
use super::game::{BackupMode, Game, GameStatus, PlaySession, Snapshot};
use super::scanner::{SaveCandidate, SavePathScanner};

type CmdResult<T> = Result<T, String>;

fn normalize_game_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
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
pub fn galgame_add_game(mut game: Game, update: bool, old_name: Option<String>) -> CmdResult<()> {
    let mut cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;

    game.name = game.name.trim().to_string();
    if game.name.is_empty() {
        return Err("游戏名称不能为空".to_string());
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
        let mut old_game = cfg.games[index].clone();
        let old_game_name = old_game.name.clone();

        // MERGE LOGIC: Preserve everything that is NOT in the basic edit form
        // 1. Stats and History
        game.total_play_time = old_game.total_play_time;
        game.play_history = old_game.play_history.clone();
        game.last_played = old_game.last_played;

        // 2. Metadata (these are not in the current VniteAddGame form)
        game.description = old_game.description.clone();
        game.developers = old_game.developers.clone();
        game.publishers = old_game.publishers.clone();
        game.release_date = old_game.release_date.clone();
        game.genres = old_game.genres.clone();
        game.tags = old_game.tags.clone();
        game.platforms = old_game.platforms.clone();
        game.score = old_game.score;
        game.rating = old_game.rating;
        game.status = old_game.status.clone(); // Usually edited via status dropdown, not AddGame form
        game.nsfw = old_game.nsfw;

        // 3. IDs
        game.steam_id = old_game.steam_id.clone();
        game.vndb_id = old_game.vndb_id.clone();
        game.igdb_id = old_game.igdb_id.clone();
        game.ymgal_id = old_game.ymgal_id.clone();
        game.bangumi_id = old_game.bangumi_id.clone();

        // 4. Extra images (background, logo)
        game.background_image = old_game.background_image.clone();
        game.logo_image = old_game.logo_image.clone();

        // 5. Original Name / Sort Name
        game.original_name = old_game.original_name.clone();
        game.sort_name = old_game.sort_name.clone();

        // If name changed, we need to move files and update references
        if game.name != old_game_name {
            log::info!("Game name changed: {} -> {}", old_game_name, game.name);

            // 1. Move backup folder
            let old_backup_dir = get_backup_dir().join(&old_game_name);
            if old_backup_dir.exists() {
                let new_backup_dir = get_backup_dir().join(&game.name);
                if let Err(e) = std::fs::rename(&old_backup_dir, &new_backup_dir) {
                    log::error!("Failed to rename backup directory: {}", e);
                }
            }

            // 2. Update cover image path if it was inside the old backup dir
            if let Some(ref cover) = game.cover_image {
                if cover.contains(&format!("galgame-backups/{}", old_game_name)) {
                    game.cover_image = Some(cover.replace(
                        &format!("galgame-backups/{}", old_game_name),
                        &format!("galgame-backups/{}", game.name),
                    ));
                }
            }

            // 3. Update collection references
            for col in &mut cfg.collections.collections {
                for g_name in &mut col.games {
                    if *g_name == old_game_name {
                        *g_name = game.name.clone();
                    }
                }
            }
        }

        cfg.games[index] = game.clone();
    } else {
        cfg.games.push(game.clone());
    }

    // Handle cover image auto-backup (for new games or if cover path was updated to a local temp file)
    if let Some(cover) = &game.cover_image {
        if !cover.is_empty() {
            let src_path = std::path::Path::new(cover);
            // Only backup if the file exists and is NOT already in the managed backups folder
            if src_path.exists() && !cover.contains("galgame-backups") {
                let backup_dir = get_backup_dir().join(&game.name);
                if let Err(e) = std::fs::create_dir_all(&backup_dir) {
                    log::error!("Failed to create backup dir for cover: {}", e);
                } else {
                    if let Some(ext) = src_path.extension() {
                        let dest_path = backup_dir.join(format!("cover.{}", ext.to_string_lossy()));
                        if src_path.canonicalize().ok() != dest_path.canonicalize().ok() {
                            match std::fs::copy(src_path, &dest_path) {
                                Ok(_) => {
                                    log::info!("Cover image copied to {:?}", dest_path);
                                    if let Some(idx) =
                                        find_game_index_by_name(&cfg.games, &game.name)
                                    {
                                        cfg.games[idx].cover_image =
                                            Some(dest_path.to_string_lossy().to_string());
                                    }
                                }
                                Err(e) => log::error!("Failed to copy cover image: {}", e),
                            }
                        }
                    }
                }
            }
        }
    }

    save_config(&cfg).map_err(|e| format!("Failed to save config: {}", e))?;

    // Trigger background cloud sync
    spawn_config_sync();

    Ok(())
}

/// Helper to trigger a non-blocking cloud sync of the configuration
pub fn spawn_config_sync() {
    tauri::async_runtime::spawn(async move {
        // Give local FS a tiny bit of time to settle if just saved
        std::thread::sleep(std::time::Duration::from_millis(100));

        match load_config() {
            Ok(cfg) => {
                let proxy_str = cfg.settings.http_proxy.clone();
                let proxy = if proxy_str.is_empty() {
                    None
                } else {
                    Some(proxy_str.as_str())
                };
                log::info!("Background auto-sync: Starting...");
                if let Err(e) = sync_config_to_cloud(&cfg, proxy).await {
                    log::error!("Background auto-sync FAILED: {}", e);
                } else {
                    log::info!("Background auto-sync: Finished successfully.");
                }
            }
            Err(e) => log::error!("Background auto-sync could not load config: {}", e),
        }
    });
}

#[command]
pub async fn galgame_launch_game(app_handle: tauri::AppHandle, game_name: String) -> CmdResult<()> {
    use tauri::Emitter;

    let cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;

    // PRE-LAUNCH SYNC CHECK
    let proxy_str = cfg.settings.http_proxy.clone();
    let proxy = if proxy_str.is_empty() {
        None
    } else {
        Some(proxy_str.as_str())
    };

    if let crate::galgame::cloud::CloudBackend::GitHub { .. } = &cfg.cloud_settings.backend {
        log::info!("Pre-launch sync check for {}", game_name);
        match crate::galgame::commands::galgame_get_cloud_config_timestamp(&cfg, proxy).await {
            Ok(Some(cloud_ts)) => {
                if cloud_ts > cfg.last_updated {
                    log::warn!(
                        "SYNC_CONFLICT: Cloud config (ts:{}) is newer than local (ts:{})",
                        cloud_ts,
                        cfg.last_updated
                    );
                    return Err(format!(
                        "SYNC_CONFLICT: Cloud progress is newer. Please sync from cloud first to avoid overwriting your save."
                    ));
                }
            }
            Ok(None) => log::info!("Cloud config not found, proceeding."),
            Err(e) => log::warn!(
                "Pre-launch sync check failed (network?): {}. Proceeding anyway.",
                e
            ),
        }
    }

    let game = cfg
        .games
        .iter()
        .find(|g| g.name == game_name)
        .cloned()
        .ok_or_else(|| format!("Game not found: {}", game_name))?;

    // 1. Pre-launch sync (if auto-sync is enabled)
    if cfg.cloud_settings.always_sync && cfg.cloud_settings.backend.type_name() != "disabled" {
        log::info!("Pre-launch sync starting for {}", game_name);
        match super::cloud::sync_all_from_cloud(
            &cfg.cloud_settings.backend,
            &cfg.cloud_settings,
            &game,
            &cfg.device_id,
            None,
            proxy,
        )
        .await
        {
            Ok(count) => {
                if count > 0 {
                    log::info!(
                        "Pre-launch sync: downloaded {} files for {}",
                        count,
                        game_name
                    );
                }
            }
            Err(e) => {
                let err_msg = e.to_string();
                if err_msg.contains("SYNC_CONFLICT") {
                    log::warn!(
                        "Pre-launch sync conflict for {}. Launch aborted.",
                        game_name
                    );
                    return Err(format!("SYNC_CONFLICT:{}", game_name));
                } else {
                    log::error!(
                        "Pre-launch sync failed for {}: {}. Continuing launch...",
                        game_name,
                        err_msg
                    );
                }
            }
        }
    }

    if let Some(exe_path) = &game.exe_path {
        if exe_path.is_empty() {
            return Err("Launch path is not configured".to_string());
        }

        let exe_path_buf = std::path::PathBuf::from(exe_path);
        let exe_dir = exe_path_buf.parent().unwrap_or(std::path::Path::new("."));

        #[cfg(target_os = "windows")]
        {
            // Capture start time
            let start_time = chrono::Utc::now();

            let mut child = std::process::Command::new(&exe_path_buf)
                .current_dir(exe_dir) // IMPORTANT: Set CWD
                .spawn()
                .map_err(|e| format!("Failed to launch game: {}", e))?;

            let app_handle_clone = app_handle.clone();
            let game_name_clone = game.name.clone();
            let device_id_inner = cfg.device_id.clone();
            let exe_dir_clone = exe_dir.to_path_buf();
            let game_clone = game.clone();
            let cloud_settings_clone = cfg.cloud_settings.clone();
            let proxy_clone = proxy_str.clone();

            std::thread::spawn(move || {
                // Notify frontend
                let _ = app_handle_clone.emit("galgame-game-running", game_name_clone.clone());

                // Keep the handle for the main spawned process
                let _ = child.wait();
                log::info!(
                    "Root process for {} exited, starting directory monitoring",
                    game_name_clone
                );

                // Give it a moment for child processes to stabilize (increased to 5s)
                std::thread::sleep(std::time::Duration::from_secs(5));

                let mut is_running = true;
                let mut retry_count = 0;

                // Use a longer grace period (6 checks * 5s = 30s) to handle slow launchers
                let mut added_this_session = 0u64;

                while is_running {
                    match is_any_process_in_directory(&exe_dir_clone) {
                        Ok(running) => {
                            if !running {
                                if retry_count == 0 {
                                    // First time we detect game closed, notify UI it's finishing
                                    let _ = app_handle_clone
                                        .emit("galgame-game-closing", game_name_clone.clone());
                                }
                                retry_count += 1;
                                if (retry_count >= 3) {
                                    // 3 * 5s = 15s grace period
                                    is_running = false;
                                }
                            } else {
                                retry_count = 0;
                            }
                        }
                        Err(e) => {
                            log::error!(
                                "Monitoring error for {}: {}. Retrying...",
                                game_name_clone,
                                e
                            );
                            retry_count += 1;
                            if retry_count >= 12 {
                                log::error!(
                                    "Persistent monitoring error for {}. Stopping.",
                                    game_name_clone
                                );
                                is_running = false;
                            }
                            std::thread::sleep(std::time::Duration::from_secs(5));
                        }
                    }

                    if is_running {
                        let total_elapsed = chrono::Utc::now()
                            .signed_duration_since(start_time)
                            .num_seconds()
                            .max(0) as u64;
                        let to_add = total_elapsed.saturating_sub(added_this_session);

                        // Periodic save every minute
                        if to_add >= 60 {
                            if let Ok(mut current_cfg) = load_config() {
                                if let Some(g) = current_cfg
                                    .games
                                    .iter_mut()
                                    .find(|g| g.name == game_name_clone)
                                {
                                    g.total_play_time += to_add;
                                    g.last_played = Some(chrono::Utc::now().timestamp());
                                    let game_to_emit = g.clone();

                                    // Update heartbeat file for crash recovery
                                    if let Err(e) = super::session::write_heartbeat(
                                        &game_name_clone,
                                        start_time.timestamp(),
                                        &device_id_inner,
                                    ) {
                                        log::warn!("Heartbeat write failed: {}", e);
                                    }

                                    if let Ok(_) = save_config(&current_cfg) {
                                        added_this_session += to_add;
                                        log::info!(
                                            "Periodic save for {}: +{}s (Total: {}s)",
                                            game_name_clone,
                                            to_add,
                                            game_to_emit.total_play_time
                                        );
                                        let _ = app_handle_clone
                                            .emit("galgame-playtime-update", game_to_emit);
                                    }
                                }
                            }
                        }

                        // Initial heartbeat if not already written
                        if added_this_session == 0 {
                            let _ = super::session::write_heartbeat(
                                &game_name_clone,
                                start_time.timestamp(),
                                &device_id_inner,
                            );
                        }

                        std::thread::sleep(std::time::Duration::from_secs(5));
                    }
                }

                log::info!("Game {} and all child processes exited", game_name_clone);
                let _ = app_handle_clone.emit("galgame-game-stopped", game_name_clone.clone());

                // Clear heartbeat file on normal exit
                super::session::clear_session(&game_name_clone);

                let end_time = chrono::Utc::now();
                let total_elapsed = end_time
                    .signed_duration_since(start_time)
                    .num_seconds()
                    .max(0) as u64;
                let final_to_add = total_elapsed.saturating_sub(added_this_session);

                let mut current_game_state = game_clone.clone();
                if let Ok(mut current_cfg) = load_config() {
                    let updated = if let Some(g) = current_cfg
                        .games
                        .iter_mut()
                        .find(|g| g.name == game_name_clone)
                    {
                        g.total_play_time += final_to_add;
                        g.last_played = Some(end_time.timestamp());
                        g.play_history.push(PlaySession {
                            start_time: start_time.timestamp(),
                            duration_seconds: total_elapsed,
                            device_id: device_id_inner.clone(),
                        });
                        current_game_state = g.clone();
                        true
                    } else {
                        false
                    };

                    if updated {
                        if let Err(e) = save_config(&current_cfg) {
                            log::error!("Failed to save stats: {}", e);
                        } else {
                            log::info!(
                                "Updated playtime for {}: +{}s (Total: {}s)",
                                game_name_clone,
                                total_elapsed,
                                current_game_state.total_play_time
                            );
                            let _ = app_handle_clone
                                .emit("galgame-playtime-update", current_game_state.clone());

                            // Trigger background cloud sync after session
                            spawn_config_sync();
                        }
                    }
                }

                // Auto-backup (local snapshot)
                if current_game_state.backup_mode == BackupMode::OnGameExit
                    || current_game_state.backup_mode == BackupMode::Both
                {
                    log::info!("Triggering auto-backup for {}", current_game_state.name);
                    let backup_dir = get_backup_dir();
                    match archive::create_snapshot(
                        &current_game_state,
                        &device_id_inner,
                        &backup_dir,
                        "自动备份 (游戏退出)",
                    ) {
                        Ok(_) => {
                            log::info!("Auto-backup successful");
                            let _ = app_handle_clone.emit(
                                "galgame-auto-backup",
                                format!("{} 自动备份完成", current_game_state.name),
                            );
                        }
                        Err(e) => {
                            log::error!("Auto-backup failed: {}", e);
                            let _ = app_handle_clone.emit(
                                "galgame-auto-backup-error",
                                format!("{} 自动备份失败: {}", current_game_state.name, e),
                            );
                        }
                    }
                }

                // 2. Post-exit cloud sync (if auto-sync is enabled)
                if cloud_settings_clone.always_sync
                    && cloud_settings_clone.backend.type_name() != "disabled"
                {
                    log::info!("Post-exit cloud sync starting for {}", game_name_clone);
                    let proxy = if proxy_clone.is_empty() {
                        None
                    } else {
                        Some(proxy_clone.as_str())
                    };

                    // We need a runtime to call the async sync function from a sync thread
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        match super::cloud::sync_all_to_cloud(
                            &cloud_settings_clone.backend,
                            &cloud_settings_clone,
                            &current_game_state,
                            &device_id_inner,
                            None,
                            proxy,
                        )
                        .await
                        {
                            Ok(count) => {
                                log::info!(
                                    "Post-exit cloud sync: uploaded {} files for {}",
                                    count,
                                    game_name_clone
                                );
                                let _ = app_handle_clone.emit(
                                    "galgame-auto-sync",
                                    format!("{} 自动同步完成 ({} 文件)", game_name_clone, count),
                                );
                            }
                            Err(e) => {
                                log::error!(
                                    "Post-exit cloud sync failed for {}: {}",
                                    game_name_clone,
                                    e
                                );
                                let _ = app_handle_clone.emit(
                                    "galgame-auto-sync-error",
                                    format!("{} 自动同步失败: {}", game_name_clone, e),
                                );
                            }
                        }
                    });
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

    config::remove_game(&mut cfg, &game_name)
        .map_err(|e| format!("Failed to delete game: {}", e))?;

    // Trigger background cloud sync
    spawn_config_sync();

    Ok(true)
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
pub fn galgame_save_config(config: GalgameConfig) -> CmdResult<()> {
    let mut current_cfg =
        load_config().map_err(|e| format!("Failed to load current config: {}", e))?;

    // Merge important fields that might be missing from frontend payload or should be preserved
    current_cfg.games = config.games;
    current_cfg.cloud_settings = config.cloud_settings;
    current_cfg.settings = config.settings; // Includes the new theme and other settings
    current_cfg.collections = config.collections;
    current_cfg.device_name = config.device_name;

    save_config(&current_cfg).map_err(|e| format!("Failed to save config: {}", e))?;
    Ok(())
}

#[command]
pub fn galgame_get_config() -> CmdResult<GalgameConfig> {
    load_config().map_err(|e| format!("Failed to load config: {}", e))
}

#[command]
pub fn galgame_restore_snapshot(game_name: String, snapshot_date: String) -> CmdResult<()> {
    let cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    let game = cfg
        .games
        .iter()
        .find(|g| g.name == game_name)
        .ok_or_else(|| format!("Game not found: {}", game_name))?;
    let snapshots = archive::list_snapshots(&game_name)
        .map_err(|e| format!("Failed to list snapshots: {}", e))?;
    let snapshot = snapshots
        .iter()
        .find(|s| s.date == snapshot_date)
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
    let snapshot = snapshots
        .iter()
        .find(|s| s.date == snapshot_date)
        .ok_or_else(|| format!("Snapshot not found: {}", snapshot_date))?;
    archive::delete_snapshot(snapshot).map_err(|e| format!("Failed to delete snapshot: {}", e))
}

#[command]
pub async fn galgame_check_cloud_connection(backend: CloudBackend) -> CmdResult<()> {
    let cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    let proxy = if cfg.settings.http_proxy.is_empty() {
        None
    } else {
        Some(cfg.settings.http_proxy.as_str())
    };
    backend
        .check_connection(&cfg.cloud_settings.root_path, proxy)
        .await
        .map_err(|e| format!("Connection failed: {}", e))
}

#[command]
pub async fn galgame_sync_to_cloud(force: Option<String>) -> CmdResult<u32> {
    let cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    let mut total_count = 0;

    // 1. Sync files per game (Delta Sync)
    let proxy_str = cfg.settings.http_proxy.clone();
    let proxy = if proxy_str.is_empty() {
        None
    } else {
        Some(proxy_str.as_str())
    };
    for game in &cfg.games {
        match super::cloud::sync_all_to_cloud(
            &cfg.cloud_settings.backend,
            &cfg.cloud_settings,
            game,
            &cfg.device_id,
            force.as_deref(),
            proxy,
        )
        .await
        {
            Ok(count) => total_count += count,
            Err(e) => {
                log::error!("Failed to sync game {}: {}", game.name, e);
                if e.to_string().contains("SYNC_CONFLICT") {
                    return Err(format!(
                        "SYNC_CONFLICT: Conflict detected for game: {}",
                        game.name
                    ));
                } else {
                    return Err(format!("Failed to sync {}: {}", game.name, e));
                }
            }
        }
    }

    // 2. Sync config (Metadata, Status, PlayTime)
    sync_config_to_cloud(&cfg, proxy).await?;

    // Update last sync time
    if let Ok(mut current_cfg) = load_config() {
        current_cfg.last_sync_time = chrono::Utc::now().timestamp();
        let _ = save_config(&current_cfg);
    }

    Ok(total_count)
}

#[command]
pub async fn galgame_sync_from_cloud(force: Option<String>) -> CmdResult<u32> {
    // Reload config in case it changed
    let mut cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    let mut total_count = 0;

    // 1. Sync files per game (Delta Sync)
    let proxy_str = cfg.settings.http_proxy.clone();
    let proxy = if proxy_str.is_empty() {
        None
    } else {
        Some(proxy_str.as_str())
    };
    for game in &cfg.games {
        match super::cloud::sync_all_from_cloud(
            &cfg.cloud_settings.backend,
            &cfg.cloud_settings,
            game,
            &cfg.device_id,
            force.as_deref(),
            proxy,
        )
        .await
        {
            Ok(count) => total_count += count,
            Err(e) => {
                log::error!("Failed to sync from cloud for game {}: {}", game.name, e);
                if e.to_string().contains("SYNC_CONFLICT") {
                    return Err(format!(
                        "SYNC_CONFLICT: Conflict detected for game: {}",
                        game.name
                    ));
                } else {
                    return Err(format!("Failed to download {}: {}", game.name, e));
                }
            }
        }
    }

    // 2. Sync config from cloud (Merge into local)
    let modified = sync_config_from_cloud(&mut cfg, proxy).await?;

    if modified {
        cfg.last_sync_time = chrono::Utc::now().timestamp();
        save_config(&cfg).map_err(|e| format!("Failed to save merged config: {}", e))?;
    } else {
        // Even if no data changed, update the last sync time to show success
        cfg.last_sync_time = chrono::Utc::now().timestamp();
        save_config(&cfg).map_err(|e| format!("Failed to update sync timestamp: {}", e))?;
    }

    Ok(total_count)
}

async fn sync_config_to_cloud(cfg: &GalgameConfig, proxy: Option<&str>) -> CmdResult<()> {
    let backend = &cfg.cloud_settings.backend;
    if let CloudBackend::Disabled = backend {
        return Ok(());
    }

    // Serialize config (Sensitive data like passwords in CloudSettings are included,
    // but the file is stored in the users private bucket/repo. This is acceptable for personal sync.)
    let json = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;

    let op = backend
        .get_operator(&cfg.cloud_settings.root_path, proxy)
        .map_err(|e| e.to_string())?;

    log::info!(
        "Uploading galgame_config.json to cloud, size: {} bytes",
        json.len()
    );

    op.write("galgame_config.json", json.as_bytes().to_vec())
        .await
        .map_err(|e| format!("Failed to upload config: {}", e))?;

    log::info!("Config upload successful");

    Ok(())
}

async fn sync_config_from_cloud(
    local_cfg: &mut GalgameConfig,
    proxy: Option<&str>,
) -> CmdResult<bool> {
    let backend = &local_cfg.cloud_settings.backend;
    if let CloudBackend::Disabled = backend {
        return Ok(false);
    }

    let op = backend
        .get_operator(&local_cfg.cloud_settings.root_path, proxy)
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
    let cloud_cfg_str =
        String::from_utf8(bytes.to_vec()).map_err(|e| format!("Invalid UTF-8 in config: {}", e))?;
    let cloud_cfg: GalgameConfig = serde_json::from_str(&cloud_cfg_str)
        .map_err(|e| format!("Failed to parse cloud config: {}", e))?;

    log::info!(
        "Parsed cloud config, games count: {}",
        cloud_cfg.games.len()
    );

    // Merge Logic
    let mut modified = false;
    for cloud_game in cloud_cfg.games {
        if let Some(local_index) = find_game_index_by_name(&local_cfg.games, &cloud_game.name) {
            let local_game = &mut local_cfg.games[local_index];

            // 1. Calculate legacy time (time recorded before history tracking)
            let local_history_sum: u64 = local_game
                .play_history
                .iter()
                .map(|s| s.duration_seconds)
                .sum();
            let cloud_history_sum: u64 = cloud_game
                .play_history
                .iter()
                .map(|s| s.duration_seconds)
                .sum();
            let local_legacy = local_game.total_play_time.saturating_sub(local_history_sum);
            let cloud_legacy = cloud_game.total_play_time.saturating_sub(cloud_history_sum);
            let max_legacy = local_legacy.max(cloud_legacy);

            // 2. Sync status and nsfw
            if local_game.status == GameStatus::NotStarted
                && cloud_game.status != GameStatus::NotStarted
            {
                local_game.status = cloud_game.status.clone();
                modified = true;
            }
            if local_game.nsfw != cloud_game.nsfw {
                local_game.nsfw = cloud_game.nsfw;
                modified = true;
            }

            // 3. Merge Play History (Union + duration reconciliation)
            let existing_keys: std::collections::HashMap<_, usize> = local_game
                .play_history
                .iter()
                .enumerate()
                .map(|(idx, p)| ((p.start_time, p.device_id.clone()), idx))
                .collect();

            let mut history_changed = false;
            for session in &cloud_game.play_history {
                let key = (session.start_time, session.device_id.clone());
                if let Some(existing_idx) = existing_keys.get(&key).copied() {
                    if session.duration_seconds
                        > local_game.play_history[existing_idx].duration_seconds
                    {
                        local_game.play_history[existing_idx].duration_seconds =
                            session.duration_seconds;
                        history_changed = true;
                    }
                } else {
                    local_game.play_history.push(session.clone());
                    history_changed = true;
                }
            }

            // 4. Recalculate Total Play Time
            let new_history_sum: u64 = local_game
                .play_history
                .iter()
                .map(|s| s.duration_seconds)
                .sum();
            let new_total = max_legacy + new_history_sum;

            if new_total != local_game.total_play_time {
                local_game.total_play_time = new_total;
                modified = true;
            }

            // Sort history by time just in case
            if history_changed {
                local_game
                    .play_history
                    .sort_by(|a, b| a.start_time.cmp(&b.start_time));
                modified = true;
            }

            // Merge Last Played (Latest)
            match (local_game.last_played, cloud_game.last_played) {
                (Some(l), Some(c)) => {
                    if c > l {
                        local_game.last_played = Some(c);
                        modified = true;
                    }
                }
                (None, Some(c)) => {
                    local_game.last_played = Some(c);
                    modified = true;
                }
                _ => {}
            }

            // [Phase 4] Merge Status (Advanced logic)
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

            // [Phase 4] Merge metadata fields
            if local_game.original_name.is_none() && cloud_game.original_name.is_some() {
                local_game.original_name = cloud_game.original_name.clone();
                modified = true;
            }
            if local_game.sort_name.is_none() && cloud_game.sort_name.is_some() {
                local_game.sort_name = cloud_game.sort_name.clone();
                modified = true;
            }
            if local_game.description.is_none() && cloud_game.description.is_some() {
                local_game.description = cloud_game.description.clone();
                modified = true;
            }
            if local_game.developer.is_none() && cloud_game.developer.is_some() {
                local_game.developer = cloud_game.developer.clone();
                modified = true;
            }
            if local_game.developers.is_empty() && !cloud_game.developers.is_empty() {
                local_game.developers = cloud_game.developers.clone();
                modified = true;
            }
            if local_game.publishers.is_empty() && !cloud_game.publishers.is_empty() {
                local_game.publishers = cloud_game.publishers.clone();
                modified = true;
            }
            if local_game.release_date.is_none() && cloud_game.release_date.is_some() {
                local_game.release_date = cloud_game.release_date.clone();
                modified = true;
            }
            if local_game.genres.is_empty() && !cloud_game.genres.is_empty() {
                local_game.genres = cloud_game.genres.clone();
                modified = true;
            }
            if local_game.tags.is_empty() && !cloud_game.tags.is_empty() {
                local_game.tags = cloud_game.tags.clone();
                modified = true;
            }
            if local_game.platforms.is_empty() && !cloud_game.platforms.is_empty() {
                local_game.platforms = cloud_game.platforms.clone();
                modified = true;
            }

            // [Phase 4] Merge IDs
            if local_game.steam_id.is_none() && cloud_game.steam_id.is_some() {
                local_game.steam_id = cloud_game.steam_id.clone();
                modified = true;
            }
            if local_game.vndb_id.is_none() && cloud_game.vndb_id.is_some() {
                local_game.vndb_id = cloud_game.vndb_id.clone();
                modified = true;
            }
            if local_game.igdb_id.is_none() && cloud_game.igdb_id.is_some() {
                local_game.igdb_id = cloud_game.igdb_id.clone();
                modified = true;
            }
            if local_game.ymgal_id.is_none() && cloud_game.ymgal_id.is_some() {
                local_game.ymgal_id = cloud_game.ymgal_id.clone();
                modified = true;
            }
            if local_game.bangumi_id.is_none() && cloud_game.bangumi_id.is_some() {
                local_game.bangumi_id = cloud_game.bangumi_id.clone();
                modified = true;
            }

            // Merge scores/ratings
            if local_game.score.is_none() && cloud_game.score.is_some() {
                local_game.score = cloud_game.score;
                modified = true;
            }
            if local_game.rating.is_none() && cloud_game.rating.is_some() {
                local_game.rating = cloud_game.rating;
                modified = true;
            }

            // Merge images
            if local_game.cover_image.is_none() && cloud_game.cover_image.is_some() {
                local_game.cover_image = cloud_game.cover_image.clone();
                modified = true;
            }
            if local_game.background_image.is_none() && cloud_game.background_image.is_some() {
                local_game.background_image = cloud_game.background_image.clone();
                modified = true;
            }
            if local_game.logo_image.is_none() && cloud_game.logo_image.is_some() {
                local_game.logo_image = cloud_game.logo_image.clone();
                modified = true;
            }
        } else {
            // Game exists in cloud but not local. Add it!
            local_cfg.games.push(cloud_game);
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
        (Playing, Shelved) | (Shelved, Playing) => match (local_last_played, cloud_last_played) {
            (Some(local_time), Some(cloud_time)) if cloud_time > local_time => cloud_status.clone(),
            (Some(_), Some(_)) => local_status.clone(),
            (None, Some(_)) => cloud_status.clone(),
            _ => local_status.clone(),
        },
        _ => local_status.clone(),
    }
}

#[command]
pub async fn galgame_delete_cloud_game(game_name: String) -> CmdResult<()> {
    let cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    let proxy = if cfg.settings.http_proxy.is_empty() {
        None
    } else {
        Some(cfg.settings.http_proxy.as_str())
    };
    // game_name is the folder name in root
    super::cloud::delete_directory(
        &cfg.cloud_settings.backend,
        &cfg.cloud_settings,
        &game_name,
        proxy,
    )
    .await
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
    Ok(locations
        .into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect())
}

#[command]
pub fn galgame_open_backup_folder(game_name: String) -> CmdResult<()> {
    let backup_dir = get_backup_dir().join(&game_name);
    if !backup_dir.exists() {
        return Err("Backup folder does not exist".to_string());
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&backup_dir)
            .spawn()
            .ok();
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
    let proxy = if cfg.settings.http_proxy.is_empty() {
        None
    } else {
        Some(cfg.settings.http_proxy.as_str())
    };

    match backend {
        CloudBackend::Disabled => Err("Cloud sync is disabled".to_string()),
        _ => backend
            .upload_clipboard(&cfg.cloud_settings.root_path, &text, proxy)
            .await
            .map_err(|e| format!("Upload failed: {}", e)),
    }
}

#[command]
pub async fn galgame_sync_clipboard_from_cloud() -> CmdResult<String> {
    let cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    let backend = cfg.cloud_settings.backend;
    let proxy = if cfg.settings.http_proxy.is_empty() {
        None
    } else {
        Some(cfg.settings.http_proxy.as_str())
    };

    match backend {
        CloudBackend::Disabled => Err("Cloud sync is disabled".to_string()),
        _ => backend
            .download_clipboard(&cfg.cloud_settings.root_path, proxy)
            .await
            .map_err(|e| format!("Download failed: {}", e)),
    }
}

#[command]
pub async fn galgame_github_oauth_request()
-> CmdResult<crate::network::github_oauth::DeviceCodeResponse> {
    let cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    let proxy = if cfg.settings.http_proxy.is_empty() {
        None
    } else {
        Some(cfg.settings.http_proxy.as_str())
    };

    crate::network::github_oauth::request_device_code(proxy)
        .await
        .map_err(|e| format!("Request failed: {}", e))
}

#[command]
pub async fn galgame_github_oauth_poll(
    device_code: String,
    interval: u64,
    expires_in: u64,
) -> CmdResult<String> {
    let cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    let proxy = if cfg.settings.http_proxy.is_empty() {
        None
    } else {
        Some(cfg.settings.http_proxy.as_str())
    };

    crate::network::github_oauth::poll_for_access_token(device_code, interval, expires_in, proxy)
        .await
        .map_err(|e| format!("Poll failed: {}", e))
}

#[command]
pub async fn galgame_github_setup_repo(token: String) -> CmdResult<String> {
    let cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    let proxy = if cfg.settings.http_proxy.is_empty() {
        None
    } else {
        Some(cfg.settings.http_proxy.as_str())
    };

    crate::network::github_oauth::setup_github_repository(&token, proxy)
        .await
        .map_err(|e| format!("Setup failed: {}", e))
}

// ── Scraper commands ──

#[command]
pub async fn galgame_search_metadata(
    keyword: String,
    source: String,
) -> CmdResult<Vec<super::scraper::MetadataResult>> {
    let cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    let proxy = if cfg.settings.http_proxy.is_empty() {
        None
    } else {
        Some(cfg.settings.http_proxy.as_str())
    };

    let src = if source.is_empty() { "all" } else { &source };
    super::scraper::search_metadata_multi(&keyword, src, proxy)
        .await
        .map_err(|e| format!("Search failed: {}", e))
}

#[command]
pub async fn galgame_apply_metadata(
    game_name: String,
    data: super::scraper::MetadataResult,
) -> CmdResult<()> {
    let mut cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;

    if let Some(idx) = find_game_index_by_name(&cfg.games, &game_name) {
        let game = &mut cfg.games[idx];

        if let Some(ref orig) = data.original_title {
            game.original_name = Some(orig.clone());
        }
        if let Some(ref desc) = data.description {
            game.description = Some(desc.clone());
        }
        if let Some(ref rd) = data.release_date {
            game.release_date = Some(rd.clone());
        }
        if let Some(ref dev) = data.developer {
            game.developer = Some(dev.clone());
        }
        if !data.developers.is_empty() {
            game.developers = data.developers.clone();
            if game.developer.is_none() {
                game.developer = game.developers.first().cloned();
            }
        }
        if !data.publishers.is_empty() {
            game.publishers = data.publishers.clone();
        }
        if !data.tags.is_empty() {
            game.tags = data.tags.clone();
        }
        if !data.genres.is_empty() {
            game.genres = data.genres.clone();
        }
        if !data.platforms.is_empty() {
            game.platforms = data.platforms.clone();
        }
        if let Some(rating) = data.rating {
            game.rating = Some(rating);
        }

        game.nsfw = data.nsfw;

        let source_name = data.source.as_deref().unwrap_or("");
        match source_name {
            "VNDB" => {
                game.vndb_id = Some(data.id.clone());
            }
            "Steam" => {
                game.steam_id = Some(
                    data.id
                        .strip_prefix("steam-")
                        .unwrap_or(&data.id)
                        .to_string(),
                );
            }
            "Bangumi" => {
                game.bangumi_id =
                    Some(data.id.strip_prefix("bgm-").unwrap_or(&data.id).to_string());
            }
            "YMGal" => {
                game.ymgal_id = Some(
                    data.id
                        .strip_prefix("ymgal-")
                        .unwrap_or(&data.id)
                        .to_string(),
                );
            }
            _ => {}
        }

        if let Some(ref cover_url) = data.cover_url {
            if !cover_url.is_empty() {
                let cover_dir = archive::get_backup_dir().join("covers");
                let ext = if cover_url.contains(".png") {
                    "png"
                } else {
                    "jpg"
                };
                let safe_name = game
                    .name
                    .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
                let cover_path = cover_dir.join(format!("{}.{}", safe_name, ext));

                let proxy = if cfg.settings.http_proxy.is_empty() {
                    None
                } else {
                    Some(cfg.settings.http_proxy.as_str())
                };
                match super::scraper::download_cover(cover_url, &cover_path, proxy).await {
                    Ok(_) => {
                        game.cover_image = Some(cover_path.to_string_lossy().to_string());
                        log::info!("Metadata cover successfully downloaded to {:?}", cover_path);
                    }
                    Err(e) => {
                        log::error!("Metadata cover download FAILED for {}: {}", game.name, e);
                    }
                }
            }
        }

        config::save_config(&cfg).map_err(|e| format!("Failed to save config: {}", e))?;

        // Trigger background cloud sync
        spawn_config_sync();

        Ok(())
    } else {
        Err(format!("Game not found: {}", game_name))
    }
}

// --- Process Monitoring Helpers ---

#[tauri::command]
pub fn galgame_get_running_game() -> CmdResult<Option<String>> {
    let cfg = load_config().map_err(|e| e.to_string())?;
    for game in &cfg.games {
        if let Some(exe_path) = &game.exe_path {
            if !exe_path.is_empty() {
                let exe_dir = std::path::Path::new(exe_path)
                    .parent()
                    .unwrap_or(std::path::Path::new("."));
                if is_any_process_in_directory(exe_dir).unwrap_or(false) {
                    return Ok(Some(game.name.clone()));
                }
            }
        }
    }
    Ok(None)
}

#[tauri::command]
pub fn galgame_kill_game(game_name: String) -> CmdResult<()> {
    let cfg = load_config().map_err(|e| e.to_string())?;
    let game = cfg
        .games
        .iter()
        .find(|g| g.name == game_name)
        .ok_or_else(|| "Game not found".to_string())?;

    if let Some(exe_path) = &game.exe_path {
        let exe_dir = std::path::Path::new(exe_path)
            .parent()
            .unwrap_or(std::path::Path::new("."));
        kill_processes_in_directory(exe_dir)?;
    }
    Ok(())
}

fn is_any_process_in_directory(dir: &std::path::Path) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        use serde::Deserialize;
        use wmi::WMIConnection;

        #[derive(Deserialize, Debug)]
        #[serde(rename_all = "PascalCase")]
        struct Win32Process {
            executable_path: Option<String>,
        }

        let wmi_con = WMIConnection::new().map_err(|e| e.to_string())?;
        let mut dir_str = dir
            .to_string_lossy()
            .to_string()
            .to_lowercase()
            .replace("/", "\\");
        if !dir_str.ends_with("\\") {
            dir_str.push('\\');
        }

        // Safety: ignore extremely short paths
        if dir_str.len() < 3 {
            return Ok(false);
        }

        let results: Vec<Win32Process> = wmi_con
            .raw_query("SELECT ExecutablePath FROM Win32_Process")
            .map_err(|e| e.to_string())?;

        let ignore_list = [
            "unitycrashhandler",
            "crashpad_handler",
            "werfault.exe",
            "conhost.exe",
            "cmd.exe",
        ];

        for proc in results {
            if let Some(path) = proc.executable_path {
                let lower_path = path.to_lowercase();
                if lower_path.contains(&dir_str) {
                    // Check if it's in the ignore list
                    let file_name = std::path::Path::new(&lower_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");

                    if ignore_list.iter().any(|&ignore| file_name.contains(ignore)) {
                        continue;
                    }

                    log::info!(
                        "Monitoring: Process STILL RUNNING in {}: {} (Found: {})",
                        dir_str,
                        lower_path,
                        file_name
                    );
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

fn kill_processes_in_directory(dir: &std::path::Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use serde::Deserialize;
        use std::os::windows::process::CommandExt;
        use std::process::Command;
        use wmi::WMIConnection;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        #[derive(Deserialize, Debug)]
        #[serde(rename_all = "PascalCase")]
        struct Win32Process {
            executable_path: Option<String>,
            process_id: u32,
        }

        let wmi_con = WMIConnection::new().map_err(|e| e.to_string())?;
        let mut dir_str = dir
            .to_string_lossy()
            .to_string()
            .to_lowercase()
            .replace("/", "\\");
        if !dir_str.ends_with("\\") {
            dir_str.push('\\');
        }

        let results: Vec<Win32Process> = wmi_con
            .raw_query("SELECT ExecutablePath, ProcessId FROM Win32_Process")
            .map_err(|e| e.to_string())?;

        for proc in results {
            if let Some(path) = proc.executable_path {
                let lower_path = path.to_lowercase();
                if lower_path.contains(&dir_str) {
                    log::info!("Killing process: {} (PID: {})", path, proc.process_id);
                    let _ = Command::new("taskkill")
                        .args(&["/F", "/PID", &proc.process_id.to_string()])
                        .creation_flags(CREATE_NO_WINDOW)
                        .spawn();
                }
            }
        }
    }
    Ok(())
}
/// Helper to get the cloud config timestamp for pre-launch sync check
pub async fn galgame_get_cloud_config_timestamp(
    cfg: &GalgameConfig,
    proxy: Option<&str>,
) -> Result<Option<i64>, String> {
    let backend = &cfg.cloud_settings.backend;
    if let CloudBackend::Disabled = backend {
        return Ok(None);
    }

    let op = backend
        .get_operator(&cfg.cloud_settings.root_path, proxy)
        .map_err(|e| e.to_string())?;

    match op.stat("galgame_config.json").await {
        Ok(meta) => {
            // opendal might not provide LastModified depending on backend
            // For GitHub it should.
            if let Some(lm) = meta.last_modified() {
                Ok(Some(lm.timestamp()))
            } else {
                // If no last_modified, we can't do the check
                Ok(None)
            }
        }
        Err(e) => {
            if e.kind() == opendal::ErrorKind::NotFound {
                Ok(None)
            } else {
                Err(format!("Failed to stat cloud config: {}", e))
            }
        }
    }
}

#[tauri::command]
pub async fn galgame_batch_add_games(games: Vec<Game>) -> CmdResult<usize> {
    let mut cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    let mut added_count = 0;

    for mut new_game in games {
        if find_game_index_by_name(&cfg.games, &new_game.name).is_none() {
            // Apply standard defaults if not set
            if new_game.game_paths.is_empty() {
                if let Some(ref exe) = new_game.exe_path {
                    if let Some(parent) = std::path::Path::new(exe).parent() {
                        new_game
                            .game_paths
                            .insert("default".to_string(), parent.to_string_lossy().to_string());
                    }
                }
            }
            cfg.games.push(new_game);
            added_count += 1;
        }
    }

    if added_count > 0 {
        save_config(&cfg).map_err(|e| format!("Failed to save config: {}", e))?;
        spawn_config_sync();
    }

    Ok(added_count)
}

// --- Collection Commands ---

#[command]
pub fn galgame_list_collections() -> CmdResult<Vec<super::collection::GameCollection>> {
    let cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    Ok(cfg.collections.collections.clone())
}

#[command]
pub fn galgame_add_collection(name: String) -> CmdResult<String> {
    let mut cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    let id = uuid::Uuid::new_v4().to_string();
    let collection = super::collection::GameCollection::new(id.clone(), name);
    cfg.collections.add_collection(collection);
    save_config(&cfg).map_err(|e| format!("Failed to save config: {}", e))?;
    spawn_config_sync();
    Ok(id)
}

#[command]
pub fn galgame_delete_collection(id: String) -> CmdResult<bool> {
    let mut cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    let removed = cfg.collections.remove_collection(&id);
    if removed {
        save_config(&cfg).map_err(|e| format!("Failed to save config: {}", e))?;
        spawn_config_sync();
    }
    Ok(removed)
}

#[command]
pub fn galgame_update_collection(collection: super::collection::GameCollection) -> CmdResult<()> {
    let mut cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    if let Some(target) = cfg.collections.get_collection_mut(&collection.id) {
        *target = collection;
        save_config(&cfg).map_err(|e| format!("Failed to save config: {}", e))?;
        spawn_config_sync();
        Ok(())
    } else {
        Err("Collection not found".to_string())
    }
}

#[command]
pub fn galgame_add_to_collection(collection_id: String, game_name: String) -> CmdResult<bool> {
    let mut cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    let added = cfg
        .collections
        .add_game_to_collection(&collection_id, &game_name);
    if added {
        save_config(&cfg).map_err(|e| format!("Failed to save config: {}", e))?;
        spawn_config_sync();
    }
    Ok(added)
}

#[command]
pub fn galgame_remove_from_collection(collection_id: String, game_name: String) -> CmdResult<bool> {
    let mut cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    let removed = cfg
        .collections
        .remove_game_from_collection(&collection_id, &game_name);
    if removed {
        save_config(&cfg).map_err(|e| format!("Failed to save config: {}", e))?;
        spawn_config_sync();
    }
    Ok(removed)
}

#[command]
pub fn galgame_prune_config() -> CmdResult<usize> {
    let mut cfg = load_config().map_err(|e| format!("Failed to load config: {}", e))?;
    let initial_count = cfg.games.len();

    // Remove games where the path no longer exists
    cfg.games.retain(|game| {
        if let Some(p) = &game.exe_path {
            std::path::Path::new(p).exists()
        } else {
            // If no exe_path, we keep it as it might be manually managed or metadata-only
            true
        }
    });

    // Remove duplicates based on normalized name
    let mut seen = std::collections::HashSet::new();
    let mut unique_games = Vec::new();
    for game in &cfg.games {
        let normalized = normalize_game_name(&game.name);
        if !seen.contains(&normalized) {
            seen.insert(normalized);
            unique_games.push(game.clone());
        }
    }
    cfg.games = unique_games;

    let pruned_count = initial_count - cfg.games.len();
    if pruned_count > 0 {
        save_config(&cfg).map_err(|e| format!("Failed to save pruned config: {}", e))?;
        spawn_config_sync();
    }
    Ok(pruned_count)
}
