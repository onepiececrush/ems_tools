use tauri::State;
use chrono::Local;

use crate::{AppState, LogEntry};

// 添加日志
#[tauri::command]
pub async fn add_log(
    level: String,
    message: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    add_log_internal(&app_state, &level, &message);
    Ok(())
}

// 内部日志添加函数，可以被其他模块调用
pub fn add_log_internal(
    app_state: &AppState,
    level: &str,
    message: &str,
) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    
    // 根据日志级别记录到系统日志
    match level {
        "INFO" => log::info!("{}", message),
        "WARN" => log::warn!("{}", message),
        "ERROR" => log::error!("{}", message),
        "DEBUG" => log::debug!("{}", message),
        _ => log::info!("{}", message),
    }
    
    // 添加到应用状态
    let log_entry = LogEntry {
        timestamp,
        level: level.to_string(),
        message: message.to_string(),
    };
    
    if let Ok(mut logs) = app_state.log_entries.lock() {
        // 限制日志条目数量，防止内存占用过多
        if logs.len() >= 1000 {
            logs.remove(0);
        }
        logs.push(log_entry);
    }
}

// 获取日志
#[tauri::command]
pub async fn get_logs(
    app_state: State<'_, AppState>,
) -> Result<Vec<LogEntry>, String> {
    match app_state.log_entries.lock() {
        Ok(logs) => Ok(logs.clone()),
        Err(_) => Err("无法访问日志数据".to_string()),
    }
}

// 清除日志
#[tauri::command]
pub async fn clear_logs(
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    match app_state.log_entries.lock() {
        Ok(mut logs) => {
            logs.clear();
            Ok(())
        },
        Err(_) => Err("无法访问日志数据".to_string()),
    }
} 