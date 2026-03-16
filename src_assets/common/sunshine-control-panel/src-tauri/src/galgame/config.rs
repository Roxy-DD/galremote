// 配置管理
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

use super::cloud::CloudSettings;
use super::game::Game;
use super::collection::CollectionStore;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Config not found")]
    NotFound,
}

pub type ConfigResult<T> = Result<T, ConfigError>;

/// Galgame 管理器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalgameConfig {
    /// 配置版本
    pub version: String,

    /// 游戏列表
    #[serde(default)]
    pub games: Vec<Game>,

    /// 云同步设置
    #[serde(default)]
    pub cloud_settings: CloudSettings,

    /// 设备 ID
    #[serde(default = "get_device_id")]
    pub device_id: String,

    /// 设备名称
    #[serde(default)]
    pub device_name: String,

    /// 通用设置
    #[serde(default)]
    pub settings: GalgameSettings,

    /// 收藏夹
    #[serde(default)]
    pub collections: CollectionStore,
}

/// 通用设置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GalgameSettings {
    /// 恢复前是否提示
    #[serde(default)]
    pub prompt_before_restore: bool,

    /// 恢复前自动备份
    #[serde(default = "default_true")]
    pub backup_before_restore: bool,

    /// 最大自动备份数量（0表示无限制）
    #[serde(default)]
    pub max_auto_backup_count: u32,

    /// 启用日志记录
    #[serde(default = "default_true")]
    pub log_to_file: bool,

    /// 退出时最小化到托盘
    #[serde(default = "default_true")]
    pub exit_to_tray: bool,
    
    /// 启用 NSFW 封面模糊
    #[serde(default = "default_true")]
    pub nsfw_blur: bool,

    /// NSFW 封面模糊强度
    #[serde(default = "default_blur_intensity")]
    pub nsfw_blur_intensity: u32,

    /// 界面主题 (light/dark)
    #[serde(default = "default_theme")]
    pub theme: String,

    /// HTTP 代理地址
    #[serde(default)]
    pub http_proxy: String,
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_blur_intensity() -> u32 {
    20
}

fn default_true() -> bool {
    true
}

fn get_device_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

impl Default for GalgameConfig {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            games: Vec::new(),
            cloud_settings: CloudSettings::default(),
            device_id: get_device_id(),
            device_name: get_device_name(),
            settings: GalgameSettings::default(),
            collections: CollectionStore::default(),
        }
    }
}

/// 获取设备名称
fn get_device_name() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "Unknown Device".to_string())
}

/// 获取配置文件路径
pub fn get_config_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("sunshine-gui")
        .join("galgame-config.json")
}

/// 加载配置
pub fn load_config() -> ConfigResult<GalgameConfig> {
    let path = get_config_path();
    if !path.exists() {
        return Ok(GalgameConfig::default());
    }

    let content = fs::read_to_string(&path)?;
    let config: GalgameConfig = serde_json::from_str(&content)?;
    Ok(config)
}

/// 保存配置
pub fn save_config(config: &GalgameConfig) -> ConfigResult<()> {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(config)?;
    fs::write(&path, content)?;
    log::info!("Config saved to {:?}", path);
    Ok(())
}

/// 删除游戏
pub fn remove_game(config: &mut GalgameConfig, game_name: &str) -> ConfigResult<bool> {
    let len_before = config.games.len();
    config.games.retain(|g| g.name != game_name);
    let removed = config.games.len() < len_before;
    if removed {
        save_config(config)?;
    }
    Ok(removed)
}
