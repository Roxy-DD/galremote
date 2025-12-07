// 存档路径自动扫描
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use walkdir::WalkDir;

/// 常见 Galgame 存档位置
pub struct SavePathScanner;

impl SavePathScanner {
    /// 获取常见存档目录
    pub fn get_common_save_locations() -> Vec<PathBuf> {
        let mut locations = Vec::new();

        // AppData/Roaming
        if let Some(app_data) = dirs::data_dir() {
            locations.push(app_data.clone());
            // 常见游戏厂商目录
            for vendor in &[
                "CIRCUS",
                "CUFFS",
                "Leaf",
                "Lump of Sugar",
                "minori",
                "NEKO WORKs",
                "Nitroplus",
                "Overflow",
                "Purple software",
                "SAGA PLANETS",
                "SkyFish",
                "TYPE-MOON",
                "Yuzusoft",
                "KEY",
                "VisualArt's",
                "August",
                "ALcot",
                "AKABEiSOFT2",
                "SMEE",
                "Hooksoft",
                "ASa Project",
                "Sphere",
                "feng",
            ] {
                let vendor_path = app_data.join(vendor);
                if vendor_path.exists() {
                    locations.push(vendor_path);
                }
            }
        }

        // AppData/LocalLow (Unity 游戏)
        if let Some(local_low) = dirs::data_local_dir()
            .map(|p| p.parent().map(|pp| pp.join("LocalLow")))
            .flatten()
        {
            if local_low.exists() {
                locations.push(local_low);
            }
        }

        // Documents
        if let Some(documents) = dirs::document_dir() {
            locations.push(documents.clone());
            // 常见子目录
            for subdir in &["My Games", "SaveData", "Saves", "ゲーム"] {
                let sub_path = documents.join(subdir);
                if sub_path.exists() {
                    locations.push(sub_path);
                }
            }
        }

        // Saved Games
        if let Some(home) = dirs::home_dir() {
            let saved_games = home.join("Saved Games");
            if saved_games.exists() {
                locations.push(saved_games);
            }
        }

        locations
    }

    /// 扫描目录中可能的存档文件夹
    pub fn scan_directory(path: &PathBuf, max_depth: usize) -> Vec<SaveCandidate> {
        let mut candidates = Vec::new();

        for entry in WalkDir::new(path)
            .max_depth(max_depth)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
        {
            let dir_path = entry.path();

            // 检查是否包含存档文件
            if Self::looks_like_save_folder(dir_path) {
                let name = Self::guess_game_name(dir_path);
                candidates.push(SaveCandidate {
                    path: dir_path.to_path_buf(),
                    game_name: name,
                    confidence: Self::calculate_confidence(dir_path),
                });
            }
        }

        // 按置信度排序
        candidates.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        candidates
    }

    /// 判断目录是否像存档文件夹
    fn looks_like_save_folder(path: &std::path::Path) -> bool {
        if !path.is_dir() {
            return false;
        }

        // 检查常见存档文件扩展名
        let save_extensions = ["sav", "save", "dat", "ini", "json", "ksdat", "ksd"];
        let save_patterns = ["save", "data", "config", "system", "global"];

        let has_save_file = std::fs::read_dir(path)
            .map(|entries| {
                entries.filter_map(|e| e.ok()).any(|entry| {
                    let file_name = entry.file_name().to_string_lossy().to_lowercase();
                    let path = entry.path();

                    // 检查扩展名
                    if let Some(ext) = path.extension() {
                        let ext_str = ext.to_string_lossy().to_lowercase();
                        if save_extensions.contains(&ext_str.as_str()) {
                            return true;
                        }
                    }

                    // 检查文件名模式
                    save_patterns.iter().any(|p| file_name.contains(p))
                })
            })
            .unwrap_or(false);

        has_save_file
    }

    /// 猜测游戏名称
    fn guess_game_name(path: &std::path::Path) -> String {
        // 尝试从路径中提取游戏名称
        let components: Vec<_> = path.components().collect();

        // 优先使用倒数第二个目录名（通常是游戏名）
        if components.len() >= 2 {
            if let Some(comp) = components.get(components.len() - 2) {
                let name = comp.as_os_str().to_string_lossy().to_string();
                // 过滤掉通用目录名
                if !["SaveData", "Saves", "save", "data", "AppData", "Roaming"]
                    .contains(&name.as_str())
                {
                    return name;
                }
            }
        }

        // 使用当前目录名
        path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown Game".to_string())
    }

    /// 计算置信度
    fn calculate_confidence(path: &std::path::Path) -> f32 {
        let mut confidence: f32 = 0.5;

        // 检查文件数量
        let file_count = std::fs::read_dir(path)
            .map(|entries| entries.count())
            .unwrap_or(0);

        if file_count > 0 && file_count < 50 {
            confidence += 0.2;
        }

        // 检查是否有 .sav 文件
        if std::fs::read_dir(path)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .any(|e| e.path().extension().map_or(false, |ext| ext == "sav"))
            })
            .unwrap_or(false)
        {
            confidence += 0.3;
        }

        confidence.min(1.0_f32)
    }

    /// 获取已知的 Galgame 存档路径模板
    #[allow(dead_code)]
    pub fn get_known_game_paths() -> HashMap<String, String> {
        let mut paths = HashMap::new();

        // 常见游戏存档路径模板
        paths.insert(
            "Fate/stay night".to_string(),
            "<appdata>/TYPE-MOON/Fate/savedata".to_string(),
        );
        paths.insert("CLANNAD".to_string(), "<documents>/KEY/CLANNAD".to_string());
        paths.insert(
            "Summer Pockets".to_string(),
            "<appdata>/KEY/SummerPockets".to_string(),
        );

        paths
    }
}

/// 存档候选
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveCandidate {
    pub path: PathBuf,
    pub game_name: String,
    pub confidence: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_common_locations() {
        let locations = SavePathScanner::get_common_save_locations();
        assert!(!locations.is_empty());
    }
}
