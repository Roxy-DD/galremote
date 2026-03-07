// 存档压缩/解压功能
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

use super::game::{Game, SaveUnitType, Snapshot};

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum ArchiveError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Path error: {0}")]
    PathError(String),
    #[error("Game not found: {0}")]
    GameNotFound(String),
    #[error("Metadata error: {0}")]
    Metadata(#[from] serde_json::Error),
}

pub type ArchiveResult<T> = Result<T, ArchiveError>;

#[derive(Debug, Serialize, Deserialize)]
struct SnapshotMetadata {
    version: String,
    date: String,
    describe: String,
    device_id: String,
    game_name: String,
}

/// 创建存档快照
pub fn create_snapshot(
    game: &Game,
    device_id: &str,
    backup_dir: &Path,
    describe: &str,
) -> ArchiveResult<Snapshot> {
    let timestamp = chrono::Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let game_backup_dir = backup_dir.join(&game.name);
    fs::create_dir_all(&game_backup_dir)?;

    let zip_path = game_backup_dir.join(format!("{}.zip", timestamp));
    let file = File::create(&zip_path)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(6));

    let mut total_size: u64 = 0;

    // 1. 写入元数据
    let metadata = SnapshotMetadata {
        version: "1.0".to_string(),
        date: timestamp.clone(),
        describe: describe.to_string(),
        device_id: device_id.to_string(),
        game_name: game.name.clone(),
    };
    let metadata_json = serde_json::to_string_pretty(&metadata)?;
    zip.start_file("sunshine_metadata.json", options)?;
    zip.write_all(metadata_json.as_bytes())?;

    // 诊断日志
    let mut debug_log = String::new();
    debug_log.push_str(&format!("Snapshot Creation Log\n"));
    debug_log.push_str(&format!("Timestamp: {}\n", timestamp));
    debug_log.push_str(&format!("Device ID: {}\n", device_id));
    debug_log.push_str(&format!("Game Name: {}\n", game.name));
    debug_log.push_str(&format!("Save Paths Count: {}\n\n", game.save_paths.len()));

    // 2. 写入存档文件
    let mut file_count = 0;
    for (index, save_unit) in game.save_paths.iter().enumerate() {
        debug_log.push_str(&format!("Processing Unit #{}:\n", index + 1));
        debug_log.push_str(&format!("  Type: {:?}\n", save_unit.unit_type));
        debug_log.push_str(&format!(
            "  Available Keys: {:?}\n",
            save_unit.paths.keys().collect::<Vec<_>>()
        ));

        if let Some(path_str) = save_unit
            .paths
            .get(device_id)
            .or_else(|| save_unit.paths.get("default"))
        {
            let resolved_path_str = resolve_path_variables(path_str);
            let path = PathBuf::from(&resolved_path_str);

            debug_log.push_str(&format!("  Configured Path: {}\n", path_str));
            debug_log.push_str(&format!("  Resolved Path: {}\n", resolved_path_str));
            debug_log.push_str(&format!("  Path Exists: {}\n", path.exists()));

            if !path.exists() {
                log::warn!("Save path does not exist: {:?}", path);
                debug_log.push_str("  [WARNING] Path does not exist on disk!\n");
                continue;
            }

            match save_unit.unit_type {
                SaveUnitType::File => {
                    if path.is_file() {
                        let file_name = path
                            .file_name()
                            .ok_or_else(|| ArchiveError::PathError("Invalid file name".into()))?
                            .to_string_lossy();

                        debug_log.push_str(&format!("  Adding File: {}\n", file_name));

                        let mut file = File::open(&path)?;
                        let mut buffer = Vec::new();
                        file.read_to_end(&mut buffer)?;
                        total_size += buffer.len() as u64;

                        zip.start_file(file_name.to_string(), options)?;
                        zip.write_all(&buffer)?;
                        file_count += 1;
                    } else {
                        debug_log
                            .push_str("  [ERROR] Expected a file but found directory or other.\n");
                    }
                }
                SaveUnitType::Folder => {
                    if path.is_dir() {
                        let base_name = path
                            .file_name()
                            .ok_or_else(|| ArchiveError::PathError("Invalid folder name".into()))?
                            .to_string_lossy();

                        debug_log.push_str(&format!("  Scanning Directory: {}\n", base_name));
                        let mut dir_file_count = 0;

                        for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
                            let entry_path = entry.path();
                            let relative_path = entry_path.strip_prefix(&path).map_err(|_| {
                                ArchiveError::PathError("Failed to get relative path".into())
                            })?;

                            let archive_path = if relative_path.as_os_str().is_empty() {
                                continue;
                            } else {
                                format!(
                                    "{}/{}",
                                    base_name,
                                    relative_path.to_string_lossy().replace('\\', "/")
                                )
                            };

                            if entry_path.is_file() {
                                let mut file = File::open(entry_path)?;
                                let mut buffer = Vec::new();
                                file.read_to_end(&mut buffer)?;
                                total_size += buffer.len() as u64;

                                zip.start_file(archive_path.clone(), options)?;
                                zip.write_all(&buffer)?;
                                file_count += 1;
                                dir_file_count += 1;
                            } else if entry_path.is_dir() {
                                zip.add_directory(format!("{}/", archive_path), options)?;
                            }
                        }
                        debug_log.push_str(&format!(
                            "  Added {} files from directory.\n",
                            dir_file_count
                        ));
                    } else {
                        debug_log
                            .push_str("  [ERROR] Expected a directory but found file or other.\n");
                    }
                }
            }
        } else {
            log::warn!("No path configured for device: {}", device_id);
            debug_log.push_str("  [ERROR] No path configured for current Device ID!\n");
        }
        debug_log.push_str("\n");
    }

    // Write debug log to zip
    zip.start_file("creation_log.txt", options)?;
    zip.write_all(debug_log.as_bytes())?;

    zip.finish()?;

    log::info!(
        "Snapshot created: {} files, size: {}",
        file_count,
        total_size
    );

    Ok(Snapshot {
        date: timestamp,
        describe: describe.to_string(),
        path: zip_path.to_string_lossy().to_string(),
        size: Some(total_size),
        parent: None,
    })
}

/// 恢复存档快照
pub fn restore_snapshot(
    game: &Game,
    device_id: &str,
    snapshot: &Snapshot,
    force_delete_before: bool,
) -> ArchiveResult<()> {
    let zip_path = PathBuf::from(&snapshot.path);
    if !zip_path.exists() {
        return Err(ArchiveError::PathError(format!(
            "Snapshot file not found: {}",
            snapshot.path
        )));
    }

    // 准备日志
    let mut log_content = String::new();
    log_content.push_str(&format!("Restore Log for {}\n", snapshot.date));
    log_content.push_str(&format!("Zip Path: {:?}\n", zip_path));
    log_content.push_str(&format!("Force Delete: {}\n", force_delete_before));

    let file = File::open(&zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    // 策略：遍历配置的存档路径，如果 ZIP 中包含对应内容，则根据策略清理并恢复
    for save_unit in &game.save_paths {
        // 使用 fallback 逻辑获取路径
        let path_option = save_unit
            .paths
            .get(device_id)
            .or_else(|| save_unit.paths.get("default"));

        if let Some(path_str) = path_option {
            let target_path = PathBuf::from(resolve_path_variables(path_str));
            log_content.push_str(&format!("Target Path: {:?}\n", target_path));

            let mut should_restore = false;

            match save_unit.unit_type {
                SaveUnitType::File => {
                    let file_name = target_path
                        .file_name()
                        .ok_or_else(|| ArchiveError::PathError("Invalid file name".into()))?
                        .to_string_lossy();

                    if archive.by_name(&file_name).is_ok() {
                        should_restore = true;
                        log_content.push_str("  Matched File in Zip.\n");
                    }
                }
                SaveUnitType::Folder => {
                    let folder_name = target_path
                        .file_name()
                        .ok_or_else(|| ArchiveError::PathError("Invalid folder name".into()))?
                        .to_string_lossy();
                    let prefix = format!("{}/", folder_name);

                    log_content.push_str(&format!("  Checking prefix: {}\n", prefix));

                    for i in 0..archive.len() {
                        if let Ok(file) = archive.by_index(i) {
                            if file.name().starts_with(&prefix) {
                                should_restore = true;
                                log_content.push_str(&format!(
                                    "  Found matching entry: {}\n",
                                    file.name()
                                ));
                                break;
                            }
                        }
                    }
                }
            }

            if should_restore {
                log::info!("Restoring unit: {:?}", target_path);
                log_content.push_str("  Action: Restoring...\n");

                // 1. 清理
                // 1. 备份与清理
                if target_path.exists() {
                    let bak_root = super::config::get_config_path()
                        .parent()
                        .unwrap_or(Path::new("."))
                        .join(".bak_saves");

                    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
                    let safe_name: String = game
                        .name
                        .chars()
                        .map(|c| if c.is_alphanumeric() { c } else { '_' })
                        .collect();
                    let unit_hash = format!(
                        "{:x}",
                        md5::compute(target_path.to_string_lossy().as_bytes())
                    );
                    let backup_dir =
                        bak_root.join(format!("{}_{}_{}", safe_name, unit_hash, timestamp));

                    log::info!(
                        "Creating safety backup of local saves before restore: {:?}",
                        backup_dir
                    );
                    if let Err(e) = std::fs::create_dir_all(&backup_dir) {
                        log::error!("Failed to create backup dir: {}", e);
                    } else {
                        // Copy existing local files to .bak_saves
                        let options = fs_extra::dir::CopyOptions::new().content_only(true);
                        if target_path.is_dir() {
                            if let Err(e) = fs_extra::dir::copy(&target_path, &backup_dir, &options)
                            {
                                log::error!("Failed to backup local dir: {}", e);
                            }
                        } else if target_path.is_file() {
                            if let Some(file_name) = target_path.file_name() {
                                let _ = std::fs::copy(&target_path, backup_dir.join(file_name));
                            }
                        }
                    }

                    if force_delete_before || save_unit.delete_before_apply {
                        log_content.push_str("  Cleaning existing files...\n");
                        log::info!("Deleting existing path: {:?}", target_path);
                        if target_path.is_file() {
                            if let Err(e) = std::fs::remove_file(&target_path) {
                                log_content
                                    .push_str(&format!("  [ERROR] Failed to delete file: {}\n", e));
                            }
                        } else if target_path.is_dir() {
                            if let Err(e) = std::fs::remove_dir_all(&target_path) {
                                log_content
                                    .push_str(&format!("  [ERROR] Failed to delete dir: {}\n", e));
                            }
                        }
                    }
                } else {
                    log_content.push_str("  Target does not exist (clean).\n");
                }

                // 2. 覆盖/解压
                match save_unit.unit_type {
                    SaveUnitType::File => {
                        let file_name = target_path.file_name().unwrap().to_string_lossy();
                        let mut zip_file = archive.by_name(&file_name)?;

                        if let Some(parent) = target_path.parent() {
                            fs::create_dir_all(parent)?;
                        }

                        let mut out_file = File::create(&target_path)?;
                        std::io::copy(&mut zip_file, &mut out_file)?;
                        log_content.push_str("  File extracted.\n");
                    }
                    SaveUnitType::Folder => {
                        let folder_name = target_path.file_name().unwrap().to_string_lossy();
                        let prefix = format!("{}/", folder_name);
                        let parent_dir = target_path.parent().unwrap_or(Path::new("."));

                        let mut count = 0;
                        for i in 0..archive.len() {
                            let mut file = archive.by_index(i)?;
                            let name = file.name().to_string();

                            if name.starts_with(&prefix) {
                                let out_path = parent_dir.join(&name);

                                if file.is_dir() {
                                    fs::create_dir_all(&out_path)?;
                                } else {
                                    if let Some(p) = out_path.parent() {
                                        fs::create_dir_all(p)?;
                                    }
                                    let mut out_file = File::create(&out_path)?;
                                    std::io::copy(&mut file, &mut out_file)?;
                                    count += 1;
                                }
                            }
                        }
                        log_content.push_str(&format!("  Extracted {} files.\n", count));
                    }
                }
            } else {
                log_content.push_str("  [WARNING] No matching content in Zip to restore.\n");
            }
        } else {
            log_content.push_str("  [ERROR] No path configured (even default)!\n");
        }
    }

    // Save log
    let log_path = get_backup_dir().join("restore_debug.log");
    let _ = fs::write(log_path, log_content);

    Ok(())
}

/// 解析路径变量
pub fn resolve_path_variables(path: &str) -> String {
    let home = dirs::home_dir().unwrap_or_default();
    let app_data = dirs::data_dir().unwrap_or_default();
    let local_app_data = dirs::data_local_dir().unwrap_or_default();
    let documents = dirs::document_dir().unwrap_or_default();

    // 在 Windows 上替换反斜杠为正斜杠可能有助统一处理，但在 create Snapshot 时 PathBuf 会处理
    // 主要是变量名的替换
    path.replace("<home>", &home.to_string_lossy())
        .replace("<appdata>", &app_data.to_string_lossy())
        .replace("<localappdata>", &local_app_data.to_string_lossy())
        .replace("<documents>", &documents.to_string_lossy())
        .replace("%APPDATA%", &app_data.to_string_lossy())
        .replace("%LOCALAPPDATA%", &local_app_data.to_string_lossy())
        .replace("%USERPROFILE%", &home.to_string_lossy())
}

/// 获取备份目录
pub fn get_backup_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("sunshine-gui")
        .join("galgame-backups")
}

/// 列出游戏的所有快照
pub fn list_snapshots(game_name: &str) -> ArchiveResult<Vec<Snapshot>> {
    let backup_dir = get_backup_dir().join(game_name);
    if !backup_dir.exists() {
        return Ok(Vec::new());
    }

    let mut snapshots = Vec::new();
    for entry in fs::read_dir(&backup_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "zip") {
            if let Some(stem) = path.file_stem() {
                let date = stem.to_string_lossy().to_string();
                let metadata = fs::metadata(&path)?;
                let size = metadata.len();
                let mut describe = String::new();

                // 尝试读取 ZIP 中的 metadata.json
                if let Ok(file) = File::open(&path) {
                    if let Ok(mut archive) = ZipArchive::new(file) {
                        if let Ok(mut meta_file) = archive.by_name("sunshine_metadata.json") {
                            let mut content = String::new();
                            if meta_file.read_to_string(&mut content).is_ok() {
                                if let Ok(meta) = serde_json::from_str::<SnapshotMetadata>(&content)
                                {
                                    describe = meta.describe;
                                }
                            }
                        }
                    }
                }

                snapshots.push(Snapshot {
                    date,
                    describe,
                    path: path.to_string_lossy().to_string(),
                    size: Some(size),
                    parent: None,
                });
            }
        }
    }

    // 按日期排序
    snapshots.sort_by(|a, b| b.date.cmp(&a.date));
    Ok(snapshots)
}

/// 删除快照
pub fn delete_snapshot(snapshot: &Snapshot) -> ArchiveResult<()> {
    let path = PathBuf::from(&snapshot.path);
    if path.exists() {
        fs::remove_file(&path)?;
    }
    Ok(())
}
