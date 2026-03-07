// 云同步功能 - WebDAV/S3/OneDrive/Google Drive
use opendal::{Operator, services};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;
use filetime::FileTime;
use sha2::{Digest, Sha256};

#[derive(Error, Debug)]
pub enum CloudError {
    #[error("Cloud backend is disabled")]
    Disabled,
    #[error("OpenDAL error: {0}")]
    OpenDal(#[from] opendal::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Connection check failed: {0}")]
    CheckFailed(String),
}

pub type CloudResult<T> = Result<T, CloudError>;

/// 云存储后端配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(tag = "type")]
pub enum CloudBackend {
    #[default]
    Disabled,
    
    /// WebDAV 后端 (坚果云、阿里云盘等)
    WebDAV {
        endpoint: String,
        username: String,
        password: String,
    },
    
    /// Amazon S3 / MinIO / 阿里云 OSS (兼容 S3)
    S3 {
        endpoint: String,
        bucket: String,
        region: String,
        access_key_id: String,
        secret_access_key: String,
    },

    /// 阿里云 OSS (原生)
    AliyunOSS {
        endpoint: String,
        bucket: String,
        access_key_id: String,
        access_key_secret: String,
    },
    
    /// OneDrive
    OneDrive {
        client_id: String,
        client_secret: String,
        refresh_token: String,
    },
    
    /// Google Drive
    GoogleDrive {
        client_id: String,
        client_secret: String,
        refresh_token: String,
    },

    /// GitHub
    GitHub {
        owner: String,
        repo: String,
        branch: String,
        token: String,
    },
}

/// 云同步设置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CloudSettings {
    /// 是否启用自动同步
    #[serde(default)]
    pub always_sync: bool,
    
    /// 自动同步间隔（分钟），0表示禁用
    #[serde(default)]
    pub auto_sync_interval: u32,
    
    /// 云端根目录
    #[serde(default = "default_root_path")]
    pub root_path: String,
    
    /// 云存储后端
    #[serde(default)]
    pub backend: CloudBackend,
}

fn default_root_path() -> String {
    "/galgame-saves".to_string()
}

/// 同步状态 (对应三向合并)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SyncStatus {
    UpToDate,
    LocalNewer,
    CloudNewer,
    Conflict,
}

/// 单个文件的元数据记录
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileMetadata {
    pub hash: String,     // SHA-256
    pub size: u64,
    pub mtime: i64,       // Modified time Unix timestamp
}

/// 云端与本地对齐的同步存根 (类似 Git Tree)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncManifest {
    pub version: u32,
    pub last_sync_time: i64,
    pub base_device_id: String,
    /// relative path -> file metadata
    pub files: HashMap<String, FileMetadata>,
}

impl SyncManifest {
    pub fn new(device_id: &str) -> Self {
        Self {
            version: 1,
            last_sync_time: chrono::Utc::now().timestamp(),
            base_device_id: device_id.to_string(),
            files: HashMap::new(),
        }
    }

    /// 构建本地存档的资产树 (Manifest)
    pub fn build_local_manifest(
        game: &crate::galgame::game::Game,
        device_id: &str,
    ) -> CloudResult<Self> {
        use walkdir::WalkDir;

        let mut manifest = Self::new(device_id);

        for save_unit in &game.save_paths {
            if let Some(path_str) = save_unit
                .paths
                .get(device_id)
                .or_else(|| save_unit.paths.get("default"))
            {
                let resolved_path_str = crate::galgame::archive::resolve_path_variables(path_str);
                let path = PathBuf::from(&resolved_path_str);

                if !path.exists() {
                    continue;
                }

                match save_unit.unit_type {
                    crate::galgame::game::SaveUnitType::File => {
                        if path.is_file() {
                            let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                            if let Ok(hash) = calculate_file_hash(&path) {
                                let meta = std::fs::metadata(&path)?;
                                let mtime = filetime::FileTime::from_last_modification_time(&meta).unix_seconds();
                                manifest.files.insert(file_name, FileMetadata {
                                    hash,
                                    size: meta.len(),
                                    mtime,
                                });
                            }
                        }
                    }
                    crate::galgame::game::SaveUnitType::Folder => {
                        if path.is_dir() {
                            let base_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                            for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
                                let entry_path = entry.path();
                                if entry_path.is_file() {
                                    if let Ok(relative) = entry_path.strip_prefix(&path) {
                                        let relative_str = relative.to_string_lossy().replace('\\', "/");
                                        let archive_path = format!("{}/{}", base_name, relative_str);
                                        
                                        if let Ok(hash) = calculate_file_hash(entry_path) {
                                            let meta = std::fs::metadata(entry_path)?;
                                            let mtime = filetime::FileTime::from_last_modification_time(&meta).unix_seconds();
                                            manifest.files.insert(archive_path, FileMetadata {
                                                hash,
                                                size: meta.len(),
                                                mtime,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(manifest)
    }

    /// 三向合并比较：本地 vs 云端
    /// 返回 HashMap<文件路径, 同步状态>
    pub fn compare_manifests(
        local: &Self,
        cloud: &Option<Self>,
    ) -> HashMap<String, SyncStatus> {
        let mut changes = HashMap::new();

        if let Some(cloud_manifest) = cloud {
            // 找出本地有改变或者新增的文件
            for (path, local_meta) in &local.files {
                if let Some(cloud_meta) = cloud_manifest.files.get(path) {
                    if local_meta.hash == cloud_meta.hash {
                        changes.insert(path.clone(), SyncStatus::UpToDate);
                    } else if local_meta.mtime > cloud_meta.mtime {
                        // 本地改动且较新 (可能是继续游玩) 
                        // 在真正的 Git 3Way 中需要一个 Base，如果没有 Base，只能依据时间戳，或一律抛出 Conflict
                        // 这里我们使用简化版冲突检测：如果云端的 DeviceID 不是本机，且本地的时间戳更晚，我们警告冲突
                        // 如果最后修改 Device 是本机，说明是在本机续玩，LocalNewer
                        if cloud_manifest.base_device_id != local.base_device_id {
                            changes.insert(path.clone(), SyncStatus::Conflict);
                        } else {
                            changes.insert(path.clone(), SyncStatus::LocalNewer);
                        }
                    } else {
                        // 云端较新
                        changes.insert(path.clone(), SyncStatus::CloudNewer);
                    }
                } else {
                    // 云端没有，本地是新增的
                    changes.insert(path.clone(), SyncStatus::LocalNewer);
                }
            }

            // 找出云端存在但本地没有的文件 (通常代表云端有新游戏进度，或者本来被删除了)
            for (path, cloud_meta) in &cloud_manifest.files {
                if !local.files.contains_key(path) {
                    changes.insert(path.clone(), SyncStatus::CloudNewer);
                }
            }
        } else {
            // 没有云端版本，全部视为 LocalNewer (初次同步上传)
            for (path, _) in &local.files {
                changes.insert(path.clone(), SyncStatus::LocalNewer);
            }
        }

        changes
    }
}

/// 计算单个文件的 Hash
pub fn calculate_file_hash(path: &Path) -> CloudResult<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

impl CloudBackend {
    /// 获取 OpenDAL Operator
    pub fn get_operator(&self, root_path: &str) -> CloudResult<Operator> {
        match self {
            CloudBackend::Disabled => Err(CloudError::Disabled),
            
            CloudBackend::WebDAV { endpoint, username, password } => {
                let builder = services::Webdav::default()
                    .endpoint(endpoint)
                    .username(username)
                    .password(password)
                    .root(root_path);
                Ok(Operator::new(builder)?.finish())
            }
            
            CloudBackend::S3 { endpoint, bucket, region, access_key_id, secret_access_key } => {
                let builder = services::S3::default()
                    .endpoint(endpoint)
                    .bucket(bucket)
                    .region(region)
                    .access_key_id(access_key_id)
                    .secret_access_key(secret_access_key)
                    .root(root_path);
                Ok(Operator::new(builder)?.finish())
            }

            CloudBackend::AliyunOSS { endpoint, bucket, access_key_id, access_key_secret } => {
                let builder = services::Oss::default()
                    .endpoint(endpoint)
                    .bucket(bucket)
                    .access_key_id(access_key_id)
                    .access_key_secret(access_key_secret)
                    .root(root_path);
                Ok(Operator::new(builder)?.finish())
            }
            
            // OneDrive 和 Google Drive 需要额外的 OAuth 流程，暂不实现
            CloudBackend::OneDrive { .. } | CloudBackend::GoogleDrive { .. } => {
                Err(CloudError::CheckFailed("OneDrive/Google Drive not yet implemented".into()))
            }

            CloudBackend::GitHub { owner, repo, branch, token } => {
                let builder = services::Github::default()
                    .owner(owner)
                    .repo(repo)
                    .token(token)
                    .root(root_path);
                
                Ok(Operator::new(builder)?.finish())
            }
        }
    }



    /// 上传剪贴板文本到云端
    pub async fn upload_clipboard(&self, root_path: &str, text: &str) -> CloudResult<()> {
        let op = self.get_operator(root_path)?;
        // 固定使用 clipboard.txt 作为同步文件
        let path = "clipboard.txt";
        
        op.write(path, text.as_bytes().to_vec())
            .await
            .map_err(|e| CloudError::OpenDal(e))?;
            
        Ok(())
    }

    /// 从云端下载剪贴板文本
    pub async fn download_clipboard(&self, root_path: &str) -> CloudResult<String> {
        let op = self.get_operator(root_path)?;
        let path = "clipboard.txt";
        
        let content = op.read(path)
            .await
            .map_err(|e| CloudError::OpenDal(e))?;
            
        let content_bytes = content.to_vec();
        let text = String::from_utf8(content_bytes)
            .map_err(|e| CloudError::CheckFailed(format!("Invalid UTF-8: {}", e)))?;
            
        Ok(text)
    }

    /// 测试连接
    pub async fn check_connection(&self, root_path: &str) -> CloudResult<()> {
        const TEST_FILE: &str = ".connection_test";
        const TEST_CONTENT: &str = "sunshine-gui connection test";

        let op = self.get_operator(root_path)?;

        // 1. 创建测试文件
        op.write(TEST_FILE, TEST_CONTENT)
            .await
            .map_err(|e| CloudError::CheckFailed(format!("Failed to write: {}", e)))?;

        // 2. 读取测试文件
        let content = op.read(TEST_FILE)
            .await
            .map_err(|e| CloudError::CheckFailed(format!("Failed to read: {}", e)))?;
        
        let content_bytes = content.to_vec();
        let content_str = String::from_utf8_lossy(&content_bytes);
        if content_str != TEST_CONTENT {
            return Err(CloudError::CheckFailed("Content mismatch".into()));
        }

        // 3. 删除测试文件
        op.delete(TEST_FILE)
            .await
            .map_err(|e| CloudError::CheckFailed(format!("Failed to delete: {}", e)))?;

        Ok(())
    }

    /// 获取后端类型名称
    #[allow(dead_code)]
    pub fn type_name(&self) -> &'static str {
        match self {
            CloudBackend::Disabled => "disabled",
            CloudBackend::WebDAV { .. } => "WebDAV",
            CloudBackend::S3 { .. } => "S3",
            CloudBackend::AliyunOSS { .. } => "Aliyun OSS",
            CloudBackend::GitHub { .. } => "GitHub",
            CloudBackend::OneDrive { .. } => "OneDrive",
            CloudBackend::GoogleDrive { .. } => "Google Drive",
        }
    }
}

/// 上传文件到云端
#[allow(dead_code)]
pub async fn upload_file(
    backend: &CloudBackend,
    root_path: &str,
    local_path: &Path,
    remote_path: &str,
) -> CloudResult<()> {
    let op = backend.get_operator(root_path)?;
    let content = std::fs::read(local_path)?;
    op.write(remote_path, content).await?;
    log::info!("Uploaded {} to {}", local_path.display(), remote_path);
    Ok(())
}

/// 从云端下载文件
#[allow(dead_code)]
pub async fn download_file(
    backend: &CloudBackend,
    root_path: &str,
    remote_path: &str,
    local_path: &Path,
) -> CloudResult<()> {
    let op = backend.get_operator(root_path)?;
    let content = op.read(remote_path).await?;
    
    if let Some(parent) = local_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(local_path, content.to_vec())?;
    log::info!("Downloaded {} to {}", remote_path, local_path.display());
    Ok(())
}

/// 列出云端文件
#[allow(dead_code)]
pub async fn list_remote_files(
    backend: &CloudBackend,
    root_path: &str,
    path: &str,
) -> CloudResult<Vec<String>> {
    let op = backend.get_operator(root_path)?;
    let entries = op.list(path).await?;
    Ok(entries.into_iter().map(|e| e.name().to_string()).collect())
}

/// 递归删除云端目录
pub async fn delete_directory(
    backend: &CloudBackend,
    settings: &CloudSettings,
    dir_path: &str,
) -> CloudResult<()> {
    let op = backend.get_operator(&settings.root_path)?;
    let prefix = if dir_path.ends_with('/') {
        dir_path.to_string()
    } else {
        format!("{}/", dir_path)
    };

    // 递归列出所有文件
    async fn list_recursive(op: &Operator, path: &str) -> CloudResult<Vec<String>> {
        let mut files = Vec::new();
        // 尝试列出文件。如果后端不支持 list，可能需要其他方式，但标准 S3/WebDAV 都支持
        match op.list(path).await {
            Ok(entries) => {
                for entry in entries {
                    let full_path = if path == "." || path.is_empty() {
                        entry.name().to_string()
                    } else {
                        format!("{}/{}", path.trim_end_matches('/'), entry.name())
                    };
                    
                    if entry.metadata().is_dir() {
                        let sub_files = Box::pin(list_recursive(op, &full_path)).await?;
                        files.extend(sub_files);
                        // 记录目录本身（某些后端可能需要显示删除目录）
                        files.push(full_path); 
                    } else {
                        files.push(full_path);
                    }
                }
            },
            Err(e) => {
                log::warn!("Failed to list during delete: {}, error: {}", path, e);
            }
        }
        Ok(files)
    }

    // 列出所有文件并删除
    let files = list_recursive(&op, &prefix).await?;
    for file in files {
        if let Err(e) = op.delete(&file).await {
            log::warn!("Failed to delete file {}: {}", file, e);
        } else {
            log::info!("Deleted cloud file: {}", file);
        }
    }
    
    // 尝试删除根目录本身（如果不包含在列表中）
    let _ = op.delete(dir_path).await;
    
    Ok(())
}

/// 增量同步游戏的所有存档到云端 (Delta Sync via Manifest)
pub async fn sync_all_to_cloud(
    backend: &CloudBackend,
    settings: &CloudSettings,
    game: &crate::galgame::game::Game,
    device_id: &str,
    force: Option<&str>,
) -> CloudResult<u32> {
    let op = backend.get_operator(&settings.root_path)?;
    let mut count = 0;

    let game_cloud_dir = format!("{}/{}", settings.root_path.trim_end_matches('/'), game.name);
    let manifest_path = format!("{}/sync_manifest.json", game_cloud_dir);

    // 1. 下载云端 Manifest
    let mut cloud_manifest: Option<SyncManifest> = None;
    if op.exists(&manifest_path).await? {
        if let Ok(content) = op.read(&manifest_path).await {
            if let Ok(manifest) = serde_json::from_slice::<SyncManifest>(&content.to_bytes()) {
                cloud_manifest = Some(manifest);
            }
        }
    }

    // 2. 构建本地 Manifest
    let local_manifest = SyncManifest::build_local_manifest(game, device_id)?;

    // 3. 比较三向差异
    let changes = SyncManifest::compare_manifests(&local_manifest, &cloud_manifest);

    let mut has_conflicts = false;

    // 4. 执行同步上传
    for (relative_path, status) in &changes {
        match status {
            SyncStatus::UpToDate => {
                log::debug!("File up to date: {}", relative_path);
            }
            SyncStatus::CloudNewer => {
                log::info!("Cloud is newer, skip upload for now: {}", relative_path);
            }
            SyncStatus::Conflict => {
                log::warn!("Sync Conflict detected on {}", relative_path);
                if force == Some("local") {
                    log::info!("Force override (local to cloud) applied for {}", relative_path);
                } else {
                    has_conflicts = true;
                }
            }
            SyncStatus::LocalNewer => {}
        }
    }
    
    // We do a second pass.
    for (relative_path, status) in &changes {
        if *status == SyncStatus::LocalNewer || (*status == SyncStatus::Conflict && force == Some("local")) {
                // 执行上传
                // 还原出本地绝对路径
                let mut uploaded = false;
                for save_unit in &game.save_paths {
                    if let Some(path_str) = save_unit.paths.get(device_id).or_else(|| save_unit.paths.get("default")) {
                        let resolved_str = crate::galgame::archive::resolve_path_variables(path_str);
                        let base_path = PathBuf::from(&resolved_str);
                        
                        let target_path = if save_unit.unit_type == crate::galgame::game::SaveUnitType::File {
                            base_path.clone()
                        } else {
                            // 必须去除 base_name 才能在物理磁盘找到
                            let base_name = base_path.file_name().unwrap_or_default().to_string_lossy().to_string();
                            if relative_path.starts_with(&format!("{}/", base_name)) {
                                let stripped = relative_path.strip_prefix(&format!("{}/", base_name)).unwrap();
                                base_path.join(stripped)
                            } else {
                                continue;
                            }
                        };

                        if target_path.exists() && target_path.is_file() {
                            let cloud_file_path = format!("{}/{}", game_cloud_dir, relative_path);
                            if let Ok(content) = std::fs::read(&target_path) {
                                if let Ok(_) = op.write(&cloud_file_path, content).await {
                                    log::info!("Uploaded delta file: {}", relative_path);
                                    count += 1;
                                    uploaded = true;
                                    break;
                                }
                            }
                        }
                    }
                }
                if !uploaded {
                    log::warn!("Failed to map or upload local file for path: {}", relative_path);
                }
            }
        }

    // 5. 如果有硬冲突，中止 Manifest 上传。否则上传更新后的 Manifest
    if has_conflicts {
        // 在实际应用中，这里应该返回一个特定的 Conflict Error，通知 Vue 弹窗
        log::error!("Cannot update cloud manifest due to conflicts.");
        return Err(CloudError::CheckFailed("SYNC_CONFLICT".into()));
    }

    if count > 0 {
        let manifest_json = serde_json::to_string_pretty(&local_manifest).unwrap_or_default();
        op.write(&manifest_path, manifest_json).await?;
        log::info!("Updated cloud sync_manifest.json for Game: {}", game.name);
    }

    Ok(count)
}

/// 从云端增量同步 (Delta Download via Manifest)
pub async fn sync_all_from_cloud(
    backend: &CloudBackend,
    settings: &CloudSettings,
    game: &crate::galgame::game::Game,
    device_id: &str,
    force: Option<&str>,
) -> CloudResult<u32> {
    let op = backend.get_operator(&settings.root_path)?;
    let mut count = 0;

    let game_cloud_dir = format!("{}/{}", settings.root_path.trim_end_matches('/'), game.name);
    let manifest_path = format!("{}/sync_manifest.json", game_cloud_dir);

    // 1. 下载云端 Manifest
    let mut cloud_manifest: Option<SyncManifest> = None;
    if op.exists(&manifest_path).await? {
        if let Ok(content) = op.read(&manifest_path).await {
            if let Ok(manifest) = serde_json::from_slice::<SyncManifest>(&content.to_bytes()) {
                cloud_manifest = Some(manifest);
            }
        }
    }

    if cloud_manifest.is_none() {
        log::info!("No cloud manifest found for {}, nothing to download.", game.name);
        return Ok(0);
    }

    // 2. 构建本地 Manifest
    let local_manifest = SyncManifest::build_local_manifest(game, device_id)?;

    // 3. 比较差异
    let changes = SyncManifest::compare_manifests(&local_manifest, &cloud_manifest);

    let mut has_conflicts = false;

    // 4. 执行云端覆盖到本地
    for (relative_path, status) in changes {
        match status {
            SyncStatus::CloudNewer | SyncStatus::Conflict if force == Some("cloud") || status == SyncStatus::CloudNewer => {
                if status == SyncStatus::Conflict && force == Some("cloud") {
                     log::info!("Force override (cloud to local) applied for {}", relative_path);
                }
                
                let cloud_file_path = format!("{}/{}", game_cloud_dir, relative_path);
                
                // 还原本地写入路径
                let mut downloaded = false;
                for save_unit in &game.save_paths {
                    if let Some(path_str) = save_unit.paths.get(device_id).or_else(|| save_unit.paths.get("default")) {
                        let resolved_str = crate::galgame::archive::resolve_path_variables(path_str);
                        let base_path = PathBuf::from(&resolved_str);
                        
                        let target_path = if save_unit.unit_type == crate::galgame::game::SaveUnitType::File {
                            //如果是单文件映射，必须确保文件名一致
                            let base_name = base_path.file_name().unwrap_or_default().to_string_lossy().to_string();
                            if relative_path == base_name {
                                base_path.clone()
                            } else {
                                continue;
                            }
                        } else {
                            let base_name = base_path.file_name().unwrap_or_default().to_string_lossy().to_string();
                            if relative_path.starts_with(&format!("{}/", base_name)) {
                                let stripped = relative_path.strip_prefix(&format!("{}/", base_name)).unwrap();
                                base_path.join(stripped)
                            } else {
                                continue;
                            }
                        };

                        if let Ok(content) = op.read(&cloud_file_path).await {
                            if let Some(parent) = target_path.parent() {
                                let _ = std::fs::create_dir_all(parent);
                            }
                            if let Ok(_) = std::fs::write(&target_path, content.to_vec()) {
                                log::info!("Downloaded delta file: {}", relative_path);
                                count += 1;
                                downloaded = true;
                                break;
                            }
                        }
                    }
                }
                if !downloaded {
                    log::warn!("Could not map relative cloud path to local: {}", relative_path);
                }
            }
            SyncStatus::Conflict => {
                if force != Some("cloud") {
                    log::warn!("Sync Conflict detected on {}. Skipping download.", relative_path);
                    has_conflicts = true;
                }
            }
            _ => {}
        }
    }

    if has_conflicts {
        return Err(CloudError::CheckFailed("SYNC_CONFLICT".into()));
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_type_name() {
        assert_eq!(CloudBackend::Disabled.type_name(), "disabled");
        assert_eq!(
            CloudBackend::WebDAV {
                endpoint: String::new(),
                username: String::new(),
                password: String::new(),
            }.type_name(),
            "WebDAV"
        );
    }
}


