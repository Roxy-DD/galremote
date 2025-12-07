// 游戏数据模型
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// 存档单元类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SaveUnitType {
    File,
    Folder,
}

/// 存档单元 - 声明需要备份的文件/文件夹
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveUnit {
    pub unit_type: SaveUnitType,
    /// 设备ID到路径的映射，支持多设备
    #[serde(default)]
    pub paths: HashMap<String, String>,
    /// 恢复前是否删除原有文件
    #[serde(default)]
    pub delete_before_apply: bool,
}

/// 游戏状态
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum GameStatus {
    #[default]
    NotStarted,
    Playing,
    Finished,
    Shelved,
}

/// 游玩记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaySession {
    /// 开始时间（Unix时间戳）
    pub start_time: i64,
    /// 持续时长（秒）
    pub duration_seconds: u64,
    /// 设备ID
    pub device_id: String,
}

/// 游戏信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    /// 游戏名称
    pub name: String,
    /// 游戏封面图片路径
    #[serde(default)]
    pub cover_image: Option<String>,
    /// 存档路径列表
    pub save_paths: Vec<SaveUnit>,
    /// 游戏可执行文件路径（启动路径）
    #[serde(default)]
    pub exe_path: Option<String>,
    /// 游戏安装目录（用于检测游戏是否运行）
    #[serde(default)]
    pub game_paths: HashMap<String, String>,
    /// 备份模式
    #[serde(default)]
    pub backup_mode: BackupMode,
    /// 定时备份间隔（分钟），0表示禁用
    #[serde(default)]
    pub auto_backup_interval: u32,
    /// 总游玩时长（秒）
    #[serde(default)]
    pub total_play_time: u64,
    /// 最后一次游玩时间（Unix时间戳）
    #[serde(default)]
    pub last_played: Option<i64>,
    /// 游玩历史记录
    #[serde(default)]
    pub play_history: Vec<PlaySession>,

    /// 游戏简介 (VNDB)
    #[serde(default)]
    pub description: Option<String>,
    /// 开发商 (VNDB)
    #[serde(default)]
    pub developer: Option<String>,
    /// 发售日期 (VNDB)
    #[serde(default)]
    pub release_date: Option<String>,
    /// 游戏状态
    #[serde(default)]
    pub status: GameStatus,
}

/// 备份模式
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BackupMode {
    /// 仅手动备份
    #[default]
    Manual,
    /// 游戏关闭时自动备份
    OnGameExit,
    /// 定时备份
    Scheduled,
    /// 游戏关闭时 + 定时备份
    Both,
}

/// 快照信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// 快照唯一标识（时间戳）
    pub date: String,
    /// 描述
    pub describe: String,
    /// 备份文件路径
    pub path: String,
    /// 备份大小（字节）
    #[serde(default)]
    pub size: Option<u64>,
    /// 父快照（用于分支）
    #[serde(default)]
    pub parent: Option<String>,
}

/// 游戏的所有快照
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct GameSnapshots {
    pub name: String,
    pub backups: Vec<Snapshot>,
    /// HEAD 指向当前快照
    #[serde(default)]
    pub head: Option<String>,
}

impl Game {
    #[allow(dead_code)]
    pub fn new(name: String) -> Self {
        Self {
            name,
            cover_image: None,
            exe_path: None,
            save_paths: Vec::new(),
            game_paths: HashMap::new(),
            backup_mode: BackupMode::Manual,
            auto_backup_interval: 0,
            total_play_time: 0,
            last_played: None,
            play_history: Vec::new(),
            description: None,
            developer: None,
            release_date: None,
            status: GameStatus::default(),
        }
    }

    /// 添加存档路径
    #[allow(dead_code)]
    pub fn add_save_path(&mut self, path: String, unit_type: SaveUnitType, device_id: &str) {
        let mut paths = HashMap::new();
        paths.insert(device_id.to_string(), path);
        self.save_paths.push(SaveUnit {
            unit_type,
            paths,
            delete_before_apply: false,
        });
    }

    /// 获取当前设备的存档路径
    #[allow(dead_code)]
    pub fn get_save_paths(&self, device_id: &str) -> Vec<PathBuf> {
        self.save_paths
            .iter()
            .filter_map(|unit| unit.paths.get(device_id))
            .map(PathBuf::from)
            .collect()
    }
}

impl Snapshot {
    #[allow(dead_code)]
    pub fn new(describe: String, path: String) -> Self {
        Self {
            date: chrono::Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string(),
            describe,
            path,
            size: None,
            parent: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_creation() {
        let game = Game::new("Test Game".to_string());
        assert_eq!(game.name, "Test Game");
        assert!(game.save_paths.is_empty());
    }
}
