use std::sync::{Arc, Mutex};
use tauri::State;
use serde::{Deserialize, Serialize};

use crate::{AppState, ServiceConfig};
use crate::ssh::{SshSession, execute_command};
use crate::logger::add_log_internal;

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceStatus {
    name: String,
    status: String,
    enabled: bool,
    active: bool,
}

#[tauri::command]
pub async fn import_service(
    config: ServiceConfig,
    service_content: String,
    ssh_state: State<'_, Arc<Mutex<SshSession>>>,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    let log_message = format!("导入服务 {}", config.name);
    add_log_internal(&app_state, "INFO", &log_message);
    
    // 创建临时文件
    let temp_file = format!("/tmp/{}.service", config.name);
    let write_cmd = format!("cat > {} << 'EOL'\n{}\nEOL", temp_file, service_content);
    execute_command(&ssh_state, &write_cmd)?;
    
    // 复制到systemd目录
    let copy_cmd = format!("sudo cp {} {}", temp_file, config.service_file);
    execute_command(&ssh_state, &copy_cmd)?;
    
    // 重新加载systemd
    execute_command(&ssh_state, "sudo systemctl daemon-reload")?;
    
    // 如果需要，启用自启动
    if config.enable_autostart {
        let enable_cmd = format!("sudo systemctl enable {}", config.name);
        execute_command(&ssh_state, &enable_cmd)?;
        add_log_internal(&app_state, "INFO", &format!("已启用服务 {} 的自启动", config.name));
    }
    
    let success_message = format!("成功导入服务 {}", config.name);
    add_log_internal(&app_state, "INFO", &success_message);
    
    Ok(success_message)
}

#[tauri::command]
pub async fn enable_service(
    name: String,
    enable: bool,
    ssh_state: State<'_, Arc<Mutex<SshSession>>>,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    let action = if enable { "启用" } else { "禁用" };
    let log_message = format!("{} 服务 {} 的自启动", action, name);
    add_log_internal(&app_state, "INFO", &log_message);
    
    let cmd = if enable {
        format!("sudo systemctl enable {}", name)
    } else {
        format!("sudo systemctl disable {}", name)
    };
    
    execute_command(&ssh_state, &cmd)?;
    
    let success_message = format!("成功{} {} 服务的自启动", action, name);
    add_log_internal(&app_state, "INFO", &success_message);
    
    Ok(success_message)
}

#[tauri::command]
pub async fn start_service(
    name: String,
    ssh_state: State<'_, Arc<Mutex<SshSession>>>,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    let log_message = format!("启动服务 {}", name);
    add_log_internal(&app_state, "INFO", &log_message);
    
    let cmd = format!("sudo systemctl start {}", name);
    execute_command(&ssh_state, &cmd)?;
    
    // 检查服务状态
    let status_cmd = format!("sudo systemctl is-active {}", name);
    let status = execute_command(&ssh_state, &status_cmd)?;
    
    if status.trim() == "active" {
        let success_message = format!("成功启动服务 {}", name);
        add_log_internal(&app_state, "INFO", &success_message);
        Ok(success_message)
    } else {
        let error_message = format!("服务 {} 启动失败，状态: {}", name, status.trim());
        add_log_internal(&app_state, "ERROR", &error_message);
        Err(error_message)
    }
}

#[tauri::command]
pub async fn stop_service(
    name: String,
    ssh_state: State<'_, Arc<Mutex<SshSession>>>,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    let log_message = format!("停止服务 {}", name);
    add_log_internal(&app_state, "INFO", &log_message);
    
    let cmd = format!("sudo systemctl stop {}", name);
    execute_command(&ssh_state, &cmd)?;
    
    // 检查服务状态
    let status_cmd = format!("sudo systemctl is-active {} || echo 'inactive'", name);
    let status = execute_command(&ssh_state, &status_cmd)?;
    
    if status.trim() == "inactive" || status.trim() == "unknown" {
        let success_message = format!("成功停止服务 {}", name);
        add_log_internal(&app_state, "INFO", &success_message);
        Ok(success_message)
    } else {
        let error_message = format!("服务 {} 停止失败，状态: {}", name, status.trim());
        add_log_internal(&app_state, "ERROR", &error_message);
        Err(error_message)
    }
}

#[tauri::command]
pub async fn get_service_status(
    name: String,
    ssh_state: State<'_, Arc<Mutex<SshSession>>>,
    app_state: State<'_, AppState>,
) -> Result<ServiceStatus, String> {
    let log_message = format!("获取服务 {} 的状态", name);
    add_log_internal(&app_state, "INFO", &log_message);
    
    // 检查服务是否启用
    let enabled_cmd = format!("sudo systemctl is-enabled {} || echo 'disabled'", name);
    let enabled_output = execute_command(&ssh_state, &enabled_cmd)?;
    let enabled = enabled_output.trim() == "enabled";
    
    // 检查服务是否活动
    let active_cmd = format!("sudo systemctl is-active {} || echo 'inactive'", name);
    let active_output = execute_command(&ssh_state, &active_cmd)?;
    let active = active_output.trim() == "active";
    
    // 获取详细状态
    let status_cmd = format!("sudo systemctl status {} | head -3 | tail -1 | awk '{{$1=\"\"; print $0}}'", name);
    let status_output = execute_command(&ssh_state, &status_cmd).unwrap_or_else(|_| "未知".to_string());
    
    let status = ServiceStatus {
        name: name.clone(),
        status: status_output.trim().to_string(),
        enabled,
        active,
    };
    
    add_log_internal(&app_state, "INFO", &format!("成功获取服务 {} 的状态", name));
    
    Ok(status)
} 