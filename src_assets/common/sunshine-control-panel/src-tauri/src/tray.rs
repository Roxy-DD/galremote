use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, CheckMenuItem},
    tray::{TrayIconBuilder, MouseButton, TrayIconEvent},
    Manager, AppHandle, Runtime, Emitter
};
use std::time::Duration;
use std::sync::Mutex;
use log::{info, warn, error, debug};
use crate::utils;
use crate::toolbar;
use crate::update;
use crate::windows;

// 防止睡眠状态管理
static PREVENT_SLEEP_STATE: Mutex<bool> = Mutex::new(false);

/// 创建系统托盘
pub fn create_system_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    // 创建菜单项
    let vdd_settings = MenuItem::with_id(app, "vdd_settings", "📱 设置虚拟显示器（VDD）", true, None::<&str>)?;
    let show_toolbar = MenuItem::with_id(app, "show_toolbar", "🐾 显示工具栏", true, None::<&str>)?;
    let log_console = MenuItem::with_id(app, "log_console", "🔍 打开日志控制台", true, None::<&str>)?;
    let check_update = MenuItem::with_id(app, "check_update", "🔄 检查更新", true, None::<&str>)?;
    let about = MenuItem::with_id(app, "about", "ℹ️ 关于", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出程序", true, None::<&str>)?;
    let _separator1 = PredefinedMenuItem::separator(app)?;
    let separator2 = PredefinedMenuItem::separator(app)?;
    let separator3 = PredefinedMenuItem::separator(app)?;
    
    #[cfg(target_os = "windows")]
    let prevent_sleep = CheckMenuItem::with_id(app, "prevent_sleep", "💤 不许睡", true, false, None::<&str>)?;
    
    #[cfg(debug_assertions)]
    let open_desktop = MenuItem::with_id(app, "open_desktop", "🖥️ 打开桌面 UI", true, None::<&str>)?;
    #[cfg(debug_assertions)]
    let debug_page = MenuItem::with_id(app, "debug_page", "🐛 打开调试页面", true, None::<&str>)?;
    #[cfg(debug_assertions)]
    let separator_debug = PredefinedMenuItem::separator(app)?;
    
    // 构建菜单
    let mut items: Vec<&dyn tauri::menu::IsMenuItem<R>> = vec![
        &vdd_settings, &show_toolbar,
    ];
    
    #[cfg(target_os = "windows")]
    items.push(&prevent_sleep);
    
    items.push(&log_console);

    #[cfg(debug_assertions)]
    items.extend([&separator_debug as &dyn tauri::menu::IsMenuItem<R>, &debug_page]);

    #[cfg(debug_assertions)]
    items.push(&open_desktop);
    
    items.extend([&separator2 as &dyn tauri::menu::IsMenuItem<R>, &check_update, &about, &separator3, &quit]);
    
    let menu = Menu::with_items(app, &items)?;
    let is_admin = utils::is_running_as_admin().unwrap_or(false);
    let tooltip = if is_admin { "Sunshine GUI (管理员)" } else { "Sunshine GUI" };
    
    TrayIconBuilder::new()
        .menu(&menu)
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip(tooltip)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| handle_tray_menu_event(app, event.id().as_ref()))
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::Click { button: MouseButton::Left, .. } => handle_tray_click(tray.app_handle()),
            TrayIconEvent::DoubleClick { button: MouseButton::Left, .. } => handle_tray_double_click(tray.app_handle()),
            _ => {}
        })
        .build(app)?;
    
    Ok(())
}

/// 处理托盘单击事件
pub fn handle_tray_click<R: Runtime>(app: &AppHandle<R>) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        if let Some(window) = app.get_webview_window("main") {
            let is_visible = window.is_visible().unwrap_or(false);
            let is_minimized = window.is_minimized().unwrap_or(false);
            let is_focused = window.is_focused().unwrap_or(false);
            
            debug!("📊 窗口状态: visible={}, minimized={}, focused={}", is_visible, is_minimized, is_focused);
            
            if is_visible && !is_minimized && is_focused {
                debug!("🔽 单击：隐藏窗口");
                let _ = window.hide();
            } else {
                debug!("🔼 单击：显示窗口");
                windows::show_and_activate_window(&window);
            }
        }
    });
}

/// 处理托盘双击事件
pub fn handle_tray_double_click<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        debug!("🔼🔼 双击托盘：强制显示窗口");
        windows::show_and_activate_window(&window);
    }
}

/// 处理托盘菜单事件
pub fn handle_tray_menu_event<R: Runtime>(app: &AppHandle<R>, menu_id: &str) {
    match menu_id {
        "open_desktop" => {
            info!("🖥️ 托盘菜单：打开桌面 UI");
            if let Err(e) = windows::open_desktop_window(app) {
                error!("❌ 打开桌面 UI 失败: {}", e);
            }
        }
        "vdd_settings" => {
            open_vdd_settings(app);
        }
        "show_toolbar" => {
            toggle_toolbar(app);
        }
        "log_console" => {
            windows::open_log_console(app);
        }
        #[cfg(target_os = "windows")]
        "prevent_sleep" => {
            toggle_prevent_sleep(app);
        }
        #[cfg(debug_assertions)]
        "debug_page" => {
            info!("🐛 托盘菜单：打开调试页面");
            windows::open_debug_page(app);
        }
        "check_update" => {
            check_for_updates(app);
        }
        "about" => {
            info!("ℹ️ 托盘菜单：显示关于对话框");
            let _ = windows::open_about_window(app);
        }
        "quit" => {
            info!("🚪 托盘菜单：退出应用");
            #[cfg(target_os = "windows")]
            cleanup_prevent_sleep();
            std::process::exit(0);
        }
        _ => {
            warn!("⚠️ 未知的托盘菜单事件: {}", menu_id);
        }
    }
}

/// 打开 VDD 设置
fn open_vdd_settings<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        info!("📱 托盘菜单：打开VDD设置");
        windows::show_and_activate_window(&window);
        let _ = window.emit("open-vdd-settings", ());
    }
}

/// 切换工具栏显示/隐藏
fn toggle_toolbar<R: Runtime>(app: &AppHandle<R>) {
    info!("🔧 托盘菜单：切换工具栏显示/隐藏");
    if let Some(toolbar_window) = app.get_webview_window("toolbar") {
        let _ = toolbar_window.close();
    } else if let Err(e) = toolbar::create_toolbar_window_internal(app) {
        error!("❌ 创建工具栏失败: {}", e);
    }
}

/// 检查更新
fn check_for_updates<R: Runtime>(app: &AppHandle<R>) {
    info!("🔄 托盘菜单：检查更新");
    let app_handle = app.clone();
    
    // 确保主窗口可见
    if let Some(window) = app.get_webview_window("main") {
        windows::show_and_activate_window(&window);
    }
    
    tauri::async_runtime::spawn(async move {
        match update::check_for_updates_internal(false).await {
            Ok(Some(update_info)) => {
                info!("🎉 发现新版本: {}", update_info.version);
                save_update_check_time(&app_handle);
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.emit("update-available", &update_info);
                }
            }
            Ok(None) => {
                info!("✅ 已是最新版本");
                save_update_check_time(&app_handle);
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.emit("update-check-result", serde_json::json!({
                        "is_latest": true,
                        "message": "已是最新版本"
                    }));
                }
            }
            Err(e) => {
                error!("❌ 检查更新失败: {}", e);
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.emit("update-check-result", serde_json::json!({
                        "is_latest": false,
                        "error": e
                    }));
                }
            }
        }
    });
}

/// 保存更新检查时间
fn save_update_check_time<R: Runtime>(app: &AppHandle<R>) {
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};
    
    if let Some(prefs) = app.try_state::<Arc<Mutex<update::UpdatePreferences>>>() {
        let mut prefs = prefs.lock().unwrap();
        prefs.last_check_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

/// 切换防止睡眠功能
#[cfg(target_os = "windows")]
fn toggle_prevent_sleep<R: Runtime>(_app: &AppHandle<R>) {
    let mut state = PREVENT_SLEEP_STATE.lock().unwrap();
    let new_state = !*state;
    
    if new_state {
        info!("🌙 托盘菜单：启用防止睡眠");
        match enable_prevent_sleep() {
            Ok(()) => {
                *state = true;
            }
            Err(e) => {
                error!("❌ 启用防止睡眠失败: {}", e);
                // 如果启用失败，保持原状态
            }
        }
    } else {
        info!("💤 托盘菜单：禁用防止睡眠");
        match disable_prevent_sleep() {
            Ok(()) => {
                *state = false;
            }
            Err(e) => {
                error!("❌ 禁用防止睡眠失败: {}", e);
            }
        }
    }
}

/// 启用防止睡眠（Windows）
#[cfg(target_os = "windows")]
fn enable_prevent_sleep() -> Result<(), String> {
    // 使用 FFI 直接调用 Windows API
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn SetThreadExecutionState(es_flags: u32) -> u32;
    }
    
    // ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_AWAYMODE_REQUIRED
    // ES_CONTINUOUS: 持续有效直到调用 SetThreadExecutionState(ES_CONTINUOUS) 来清除
    // ES_SYSTEM_REQUIRED: 防止系统进入睡眠状态
    // ES_AWAYMODE_REQUIRED: 允许系统进入离开模式（如果支持）
    const ES_CONTINUOUS: u32 = 0x80000000;
    const ES_SYSTEM_REQUIRED: u32 = 0x00000001;
    const ES_AWAYMODE_REQUIRED: u32 = 0x00000040;
    
    let flags = ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_AWAYMODE_REQUIRED;
    
    unsafe {
        let result = SetThreadExecutionState(flags);
        if result == 0 {
            return Err("SetThreadExecutionState 调用失败".to_string());
        }
    }
    
    Ok(())
}

/// 禁用防止睡眠（Windows）
#[cfg(target_os = "windows")]
fn disable_prevent_sleep() -> Result<(), String> {
    // 使用 FFI 直接调用 Windows API
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn SetThreadExecutionState(es_flags: u32) -> u32;
    }
    
    // ES_CONTINUOUS: 清除所有执行状态标志
    const ES_CONTINUOUS: u32 = 0x80000000;
    
    unsafe {
        let result = SetThreadExecutionState(ES_CONTINUOUS);
        if result == 0 {
            return Err("SetThreadExecutionState 调用失败".to_string());
        }
    }
    
    Ok(())
}

/// 清理防止睡眠状态（在应用退出时调用）
#[cfg(target_os = "windows")]
pub fn cleanup_prevent_sleep() {
    let state = PREVENT_SLEEP_STATE.lock().unwrap();
    if *state {
        if let Err(e) = disable_prevent_sleep() {
            error!("❌ 清理防止睡眠状态失败: {}", e);
        } else {
            info!("✅ 已清理防止睡眠状态");
        }
    }
}
