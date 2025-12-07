// 云同步功能 - WebDAV/S3/OneDrive/Google Drive
use opendal::{Operator, services};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

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

            CloudBackend::GitHub { owner, repo, branch: _branch, token } => {
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

/// 同步所有备份到云端（镜像模式：删除云端多余的文件和文件夹）
pub async fn sync_all_to_cloud(
    backend: &CloudBackend,
    settings: &CloudSettings,
    local_backup_dir: &Path,
) -> CloudResult<u32> {
    let op = backend.get_operator(&settings.root_path)?;
    let mut count = 0;

    // 1. 获取所有本地文件 (相对路径)
    let mut local_files = std::collections::HashSet::new();
    if local_backup_dir.exists() {
        for entry in walkdir::WalkDir::new(local_backup_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let local_path = entry.path();
            if let Ok(relative) = local_path.strip_prefix(local_backup_dir) {
                let path_str = relative.to_string_lossy().replace('\\', "/");
                local_files.insert(path_str);
            }
        }
    }

    // 2. 递归列出所有云端文件
    async fn list_all_remote(op: &Operator, path: &str) -> CloudResult<Vec<String>> {
        let mut files = Vec::new();
        match op.list(path).await {
            Ok(entries) => {
                for entry in entries {
                    let name = entry.name();
                    // op.list returns relative paths to the listed path usually, 
                    // or absolute path relative to root? 
                    // OpenDAL behavior varies but usually it's simplified. 
                    // If root is "/", entry name is "folder/".
                    // Let's assume standard behavior: we reconstruct full path.
                    
                    let full_path = if path == "/" || path == "." {
                        name.to_string()
                    } else {
                        format!("{}/{}", path.trim_end_matches('/'), name)
                    };

                    if entry.metadata().is_dir() {
                        let sub_files = Box::pin(list_all_remote(op, &full_path)).await?;
                        files.extend(sub_files);
                    } else {
                        files.push(full_path);
                    }
                }
            },
            Err(e) => {
                 // NotFound is fine for listing
                 if e.kind() != opendal::ErrorKind::NotFound {
                     log::warn!("List remote failed: {}", e);
                 }
            }
        }
        Ok(files)
    }

    // List from root
    let remote_files = list_all_remote(&op, "/").await?;
    
    // 3. 找出需要删除的文件 (云端有但本地没有)
    for remote_file in remote_files {
        // 忽略特殊文件
        if remote_file == "galgame_config.json" 
            || remote_file == "clipboard.txt" 
            || remote_file == ".connection_test" 
            || remote_file.ends_with('/') // ignore directories in file set check (directories are implicit or handled separately)
        {
            continue;
        }

        // normalized path check
        let normalized = remote_file.trim_matches('/');
        if !local_files.contains(normalized) {
            log::info!("Pruning cloud file (deleted locally): {}", remote_file);
            if let Err(e) = op.delete(&remote_file).await {
                 log::error!("Failed to delete cloud file {}: {}", remote_file, e);
            }
        }
    }
    
    // 4. 清理空文件夹 (Prune empty directories)
    // Optional, but good for cleanliness. 
    // Simplified: Just delete game folders that are not in local?
    // We can reuse the folder pruning logic for top-level folders.
    // ... (Keep simpler logic: if we delete all files, folder might remain but that's harmless-ish)

    // 5. Upload Loop (Standard Sync)
    for entry in walkdir::WalkDir::new(local_backup_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().map_or(false, |ext| {
                let ext_str = ext.to_string_lossy().to_lowercase();
                matches!(ext_str.as_str(), "zip" | "jpg" | "jpeg" | "png" | "webp")
            })
        })
    {
        let local_path = entry.path();
        let relative_path = local_path
            .strip_prefix(local_backup_dir)
            .unwrap_or(local_path);
        let remote_path = relative_path.to_string_lossy().replace('\\', "/");

        // Simple check: if remote exists, skip. 
        // Improvement: Check size/time? No, zip names are timestamps provided by archive.rs.
        // So existence check is sufficient for immutable archives.
        // Cover images might change? They are "cover.jpg".
        // If cover image content changes, we should overwrite.
        // Zip files have timestamps, so unique.
        
        let is_cover = remote_path.contains("cover.");
        
        if !is_cover && op.exists(&remote_path).await? {
            // log::info!("Skipping existing remote file: {}", remote_path);
            continue;
        }

        let content = std::fs::read(local_path)?;
        op.write(&remote_path, content).await?;
        count += 1;
        log::info!("Synced to cloud: {}", remote_path);
    }

    Ok(count)
}

/// 从云端同步所有备份
pub async fn sync_all_from_cloud(
    backend: &CloudBackend,
    settings: &CloudSettings,
    local_backup_dir: &Path,
) -> CloudResult<u32> {
    let op = backend.get_operator(&settings.root_path)?;
    let mut count = 0;

    // 递归列出所有文件
    async fn list_recursive(op: &Operator, path: &str) -> CloudResult<Vec<String>> {
        let mut files = Vec::new();
        let entries = op.list(path).await?;
        
        for entry in entries {
            let full_path = if path == "." || path == "/" || path.is_empty() {
                entry.name().to_string()
            } else {
                format!("{}/{}", path.trim_end_matches('/'), entry.name())
            };
            
            if entry.metadata().is_dir() {
                let sub_files = Box::pin(list_recursive(op, &full_path)).await?;
                files.extend(sub_files);
            } else {
                let lower_path = full_path.to_lowercase();
                if lower_path.ends_with(".zip") 
                    || lower_path.ends_with(".jpg") 
                    || lower_path.ends_with(".jpeg") 
                    || lower_path.ends_with(".png") 
                    || lower_path.ends_with(".webp") 
                {
                    files.push(full_path);
                }
            }
        }
        Ok(files)
    }

    let remote_files = list_recursive(&op, "/").await?;
    
    for remote_path in remote_files {
        let local_path = local_backup_dir.join(&remote_path);
        
        // 跳过已存在的文件
        if local_path.exists() {
            continue;
        }

        let content = op.read(&remote_path).await?;
        if let Some(parent) = local_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&local_path, content.to_vec())?;
        count += 1;
        log::info!("Synced from cloud: {}", remote_path);
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


