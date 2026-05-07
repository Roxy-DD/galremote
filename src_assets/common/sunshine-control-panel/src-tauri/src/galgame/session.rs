use super::config::{get_config_path, load_config, save_config};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSession {
    pub game_name: String,
    pub start_time: i64,
    pub last_heartbeat: i64,
    pub device_id: String,
}

pub fn get_session_dir() -> PathBuf {
    get_config_path().parent().unwrap().join("sessions")
}

pub fn write_heartbeat(game_name: &str, start_time: i64, device_id: &str) -> Result<(), String> {
    let session_dir = get_session_dir();
    if !session_dir.exists() {
        fs::create_dir_all(&session_dir).map_err(|e| e.to_string())?;
    }

    let session = GameSession {
        game_name: game_name.to_string(),
        start_time,
        last_heartbeat: Utc::now().timestamp(),
        device_id: device_id.to_string(),
    };

    let path = session_dir.join(format!("{}.json", game_name));
    let content = serde_json::to_string(&session).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn clear_session(game_name: &str) {
    let path = get_session_dir().join(format!("{}.json", game_name));
    if path.exists() {
        let _ = fs::remove_file(path);
    }
}

pub fn recover_sessions() -> Result<usize, String> {
    let session_dir = get_session_dir();
    if !session_dir.exists() {
        return Ok(0);
    }

    let mut recovered_count = 0;
    let mut cfg = load_config().map_err(|e| e.to_string())?;
    let mut modified = false;

    if let Ok(entries) = fs::read_dir(session_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(session) = serde_json::from_str::<GameSession>(&content) {
                        // Only recover if it's from THIS device (to avoid multi-device conflicts)
                        if session.device_id == cfg.device_id {
                            let duration = session.last_heartbeat - session.start_time;
                            if duration > 0 {
                                if let Some(game) =
                                    cfg.games.iter_mut().find(|g| g.name == session.game_name)
                                {
                                    log::info!(
                                        "Recovering session for {}: +{}s",
                                        session.game_name,
                                        duration
                                    );
                                    game.total_play_time += duration as u64;
                                    game.last_played = Some(session.last_heartbeat);

                                    // Add to history if not too small
                                    if duration > 60 {
                                        game.play_history.push(super::game::PlaySession {
                                            start_time: session.start_time,
                                            duration_seconds: duration as u64,
                                            device_id: session.device_id.clone(),
                                        });
                                    }
                                    modified = true;
                                    recovered_count += 1;
                                }
                            }
                        }
                    }
                }
                // Always clear the session file after trying to recover
                let _ = fs::remove_file(path);
            }
        }
    }

    if modified {
        save_config(&cfg).map_err(|e| e.to_string())?;
    }

    Ok(recovered_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_serialization() {
        let session = GameSession {
            game_name: "Test Game".to_string(),
            start_time: 1000,
            last_heartbeat: 2000,
            device_id: "test-device".to_string(),
        };
        let json = serde_json::to_string(&session).unwrap();
        let decoded: GameSession = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.game_name, "Test Game");
        assert_eq!(decoded.last_heartbeat, 2000);
    }
}
