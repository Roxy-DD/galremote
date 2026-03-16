// 收藏夹系统 (对齐 Vnite gameCollectionDoc)
use serde::{Deserialize, Serialize};

/// 收藏夹
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameCollection {
    /// 收藏夹唯一ID
    pub id: String,
    /// 收藏夹名称
    pub name: String,
    /// 排序权重
    #[serde(default)]
    pub sort: i32,
    /// 排序依据
    #[serde(default = "default_sort_by")]
    pub sort_by: String,
    /// 排序方向
    #[serde(default = "default_sort_order")]
    pub sort_order: String,
    /// 游戏名称列表
    #[serde(default)]
    pub games: Vec<String>,
}

fn default_sort_by() -> String {
    "custom".to_string()
}

fn default_sort_order() -> String {
    "asc".to_string()
}

impl GameCollection {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            sort: 0,
            sort_by: default_sort_by(),
            sort_order: default_sort_order(),
            games: Vec::new(),
        }
    }
}

/// 收藏夹列表（持久化到 config 中）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CollectionStore {
    #[serde(default)]
    pub collections: Vec<GameCollection>,
}

impl CollectionStore {
    pub fn add_collection(&mut self, collection: GameCollection) {
        self.collections.push(collection);
    }

    pub fn remove_collection(&mut self, id: &str) -> bool {
        let len_before = self.collections.len();
        self.collections.retain(|c| c.id != id);
        self.collections.len() < len_before
    }

    pub fn get_collection(&self, id: &str) -> Option<&GameCollection> {
        self.collections.iter().find(|c| c.id == id)
    }

    pub fn get_collection_mut(&mut self, id: &str) -> Option<&mut GameCollection> {
        self.collections.iter_mut().find(|c| c.id == id)
    }

    pub fn add_game_to_collection(&mut self, collection_id: &str, game_name: &str) -> bool {
        if let Some(col) = self.get_collection_mut(collection_id) {
            if !col.games.contains(&game_name.to_string()) {
                col.games.push(game_name.to_string());
                return true;
            }
        }
        false
    }

    pub fn remove_game_from_collection(&mut self, collection_id: &str, game_name: &str) -> bool {
        if let Some(col) = self.get_collection_mut(collection_id) {
            let len_before = col.games.len();
            col.games.retain(|g| g != game_name);
            return col.games.len() < len_before;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_crud() {
        let mut store = CollectionStore::default();
        
        let col = GameCollection::new("col-1".to_string(), "我的收藏".to_string());
        store.add_collection(col);
        assert_eq!(store.collections.len(), 1);
        
        assert!(store.add_game_to_collection("col-1", "Amairo Chocolate"));
        assert!(!store.add_game_to_collection("col-1", "Amairo Chocolate")); // duplicate
        
        let col = store.get_collection("col-1").unwrap();
        assert_eq!(col.games.len(), 1);
        assert_eq!(col.games[0], "Amairo Chocolate");
        
        assert!(store.remove_game_from_collection("col-1", "Amairo Chocolate"));
        assert!(!store.remove_game_from_collection("col-1", "Nonexistent"));
        
        assert!(store.remove_collection("col-1"));
        assert_eq!(store.collections.len(), 0);
    }
}
