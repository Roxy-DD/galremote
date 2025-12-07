// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod commands;
mod fs_utils;
mod galgame;
mod logger;
mod proxy_server;
mod sunshine;
mod system;
mod toolbar;
mod tray;
mod update;
mod utils;
mod vdd;
mod windows;

use log::info;

fn main() {
    // 设置环境变量以忽略证书错误
    #[cfg(target_os = "windows")]
    unsafe {
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        std::env::set_var(
            "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
            "--ignore-certificate-errors",
        );
    }

    tauri::Builder::default()
        .manage(app::AppState {
            main_window: std::sync::Mutex::new(None),
        })
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            app::handle_single_instance(app, args);
        }))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // 初始化日志系统（需要在 setup 中获取 app handle）
            logger::init_logger(app.handle().clone());
            info!("🚀 Sunshine Control Panel 启动中...");

            app::setup_application(app)
        })
        .on_window_event(|window, event| {
            windows::handle_window_event(window, event);
        })
        .invoke_handler(tauri::generate_handler![
            commands::toggle_dark_mode,
            commands::show_toolbar_menu,
            toolbar::handle_toolbar_menu_action,
            toolbar::save_toolbar_position,
            system::get_current_dpi,
            system::set_desktop_dpi,
            commands::open_tool_window,
            toolbar::create_toolbar_window,
            commands::fetch_speech_phrases,
            vdd::get_vdd_settings_file_path,
            vdd::get_vdd_tools_dir_path,
            vdd::get_vdd_edid_file_path,
            vdd::load_vdd_settings,
            vdd::save_vdd_settings,
            vdd::exec_pipe_cmd,
            vdd::upload_edid_file,
            vdd::read_edid_file,
            vdd::delete_edid_file,
            system::get_gpus,
            system::get_system_info,
            sunshine::get_sunshine_install_path,
            sunshine::get_sunshine_version,
            sunshine::parse_sunshine_config,
            sunshine::get_sunshine_url,
            sunshine::get_command_line_url,
            sunshine::get_active_sessions,
            sunshine::change_bitrate,
            proxy_server::get_proxy_url_command,
            utils::open_external_url,
            utils::restart_graphics_driver,
            utils::restart_sunshine_service,
            utils::restart_as_admin,
            utils::is_running_as_admin,
            vdd::uninstall_vdd_driver,
            fs_utils::get_icc_file_list,
            fs_utils::read_directory,
            fs_utils::read_image_as_data_url,
            fs_utils::copy_image_to_assets,
            fs_utils::cleanup_unused_covers,
            fs_utils::resolve_lnk_target,
            fs_utils::scan_directory_for_apps,
            update::check_for_updates,
            update::download_update,
            update::install_update,
            logger::get_all_logs,
            logger::clear_logs,
            logger::export_logs,
            // Galgame 存档管理命令
            galgame::commands::galgame_get_config,
            galgame::commands::galgame_save_config,
            galgame::commands::galgame_add_game,
            galgame::commands::galgame_delete_game,
            galgame::commands::galgame_list_games,
            galgame::commands::galgame_create_snapshot,
            galgame::commands::galgame_restore_snapshot,
            galgame::commands::galgame_list_snapshots,
            galgame::commands::galgame_delete_snapshot,
            galgame::commands::galgame_check_cloud_connection,
            galgame::commands::galgame_sync_to_cloud,
            galgame::commands::galgame_sync_from_cloud,
            galgame::commands::galgame_scan_save_paths,
            galgame::commands::galgame_get_common_locations,
            galgame::commands::galgame_open_backup_folder,
            galgame::commands::galgame_open_save_folder,
            galgame::commands::galgame_sync_clipboard_to_cloud,
            galgame::commands::galgame_sync_clipboard_from_cloud,
            galgame::commands::galgame_delete_cloud_game,
            galgame::commands::galgame_launch_game,
            galgame::commands::galgame_search_metadata,
            galgame::commands::galgame_apply_metadata,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
