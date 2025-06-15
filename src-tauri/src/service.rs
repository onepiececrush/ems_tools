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
    let start_result = execute_command(&ssh_state, &cmd);

    // 等待一下让服务有时间启动
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // 检查服务状态
    let status_cmd = format!("sudo systemctl is-active {}", name);
    let status = execute_command(&ssh_state, &status_cmd)?;

    if status.trim() == "active" {
        let success_message = format!("成功启动服务 {}", name);
        add_log_internal(&app_state, "INFO", &success_message);
        Ok(success_message)
    } else {
        // 获取详细的错误信息
        let mut error_details = Vec::new();

        // 检查启动命令是否成功
        if let Err(start_error) = start_result {
            error_details.push(format!("启动命令执行失败: {}", start_error));
        }

        // 获取服务状态详情
        let status_detail_cmd = format!("sudo systemctl status {} --no-pager -l", name);
        if let Ok(status_detail) = execute_command(&ssh_state, &status_detail_cmd) {
            error_details.push(format!("服务状态详情: {}", status_detail));
        }

        // 获取最近的日志
        let journal_cmd = format!("sudo journalctl -u {} --no-pager -n 10", name);
        if let Ok(journal_output) = execute_command(&ssh_state, &journal_cmd) {
            error_details.push(format!("最近日志: {}", journal_output));
        }

        let error_message = format!(
            "服务 {} 启动失败，当前状态: {}\n详细信息:\n{}",
            name,
            status.trim(),
            error_details.join("\n")
        );
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
    
    // 获取详细状态描述
    let status_description = if active {
        "运行中".to_string()
    } else {
        // 尝试获取更详细的状态信息
        let status_cmd = format!("sudo systemctl is-failed {} 2>/dev/null || echo 'stopped'", name);
        let status_output = execute_command(&ssh_state, &status_cmd).unwrap_or_else(|_| "未知".to_string());
        let status_trim = status_output.trim();

        match status_trim {
            "failed" => "启动失败".to_string(),
            "stopped" => "已停止".to_string(),
            "inactive" => "未激活".to_string(),
            _ => "已停止".to_string(),
        }
    };

    let status = ServiceStatus {
        name: name.clone(),
        status: status_description,
        enabled,
        active,
    };
    
    add_log_internal(&app_state, "INFO", &format!("成功获取服务 {} 的状态", name));

    Ok(status)
}

#[tauri::command]
pub async fn search_services(
    query: String,
    ssh_state: State<'_, Arc<Mutex<SshSession>>>,
    app_state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let log_message = format!("搜索服务: {}", query);
    add_log_internal(&app_state, "INFO", &log_message);

    // 搜索包含查询字符串的服务
    let search_cmd = if query.trim().is_empty() {
        // 如果查询为空，列出所有服务
        "sudo systemctl list-unit-files --type=service | grep -E '\\.service' | awk '{print $1}' | sed 's/\\.service$//' | head -20".to_string()
    } else {
        // 模糊搜索服务名称
        format!("sudo systemctl list-unit-files --type=service | grep -i '{}' | awk '{{print $1}}' | sed 's/\\.service$//' | head -10", query)
    };

    let output = execute_command(&ssh_state, &search_cmd)?;
    let services: Vec<String> = output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_string())
        .collect();

    let success_message = format!("找到 {} 个匹配的服务", services.len());
    add_log_internal(&app_state, "INFO", &success_message);

    Ok(services)
}

#[tauri::command]
pub async fn diagnose_service(
    name: String,
    ssh_state: State<'_, Arc<Mutex<SshSession>>>,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    let log_message = format!("诊断服务 {}", name);
    add_log_internal(&app_state, "INFO", &log_message);

    let mut diagnosis = Vec::new();

    // 1. 检查服务文件是否存在
    let service_file_cmd = format!("sudo systemctl cat {} 2>/dev/null || echo 'SERVICE_FILE_NOT_FOUND'", name);
    match execute_command(&ssh_state, &service_file_cmd) {
        Ok(output) => {
            if output.trim() == "SERVICE_FILE_NOT_FOUND" {
                diagnosis.push("❌ 服务文件不存在".to_string());
            } else {
                diagnosis.push("✅ 服务文件存在".to_string());
            }
        }
        Err(e) => diagnosis.push(format!("⚠️ 无法检查服务文件: {}", e)),
    }

    // 2. 检查服务状态
    let status_cmd = format!("sudo systemctl is-active {}", name);
    match execute_command(&ssh_state, &status_cmd) {
        Ok(status) => {
            diagnosis.push(format!("📊 当前状态: {}", status.trim()));
        }
        Err(e) => diagnosis.push(format!("⚠️ 无法获取状态: {}", e)),
    }

    // 3. 检查是否启用
    let enabled_cmd = format!("sudo systemctl is-enabled {}", name);
    match execute_command(&ssh_state, &enabled_cmd) {
        Ok(enabled) => {
            diagnosis.push(format!("🔧 启用状态: {}", enabled.trim()));
        }
        Err(e) => diagnosis.push(format!("⚠️ 无法获取启用状态: {}", e)),
    }

    // 4. 获取详细状态
    let detail_cmd = format!("sudo systemctl status {} --no-pager -l", name);
    match execute_command(&ssh_state, &detail_cmd) {
        Ok(detail) => {
            diagnosis.push(format!("📋 详细状态:\n{}", detail));
        }
        Err(e) => diagnosis.push(format!("⚠️ 无法获取详细状态: {}", e)),
    }

    // 5. 获取最近日志
    let log_cmd = format!("sudo journalctl -u {} --no-pager -n 20", name);
    match execute_command(&ssh_state, &log_cmd) {
        Ok(logs) => {
            diagnosis.push(format!("📝 最近日志:\n{}", logs));
        }
        Err(e) => diagnosis.push(format!("⚠️ 无法获取日志: {}", e)),
    }

    let result = diagnosis.join("\n\n");
    add_log_internal(&app_state, "INFO", &format!("完成服务 {} 的诊断", name));

    Ok(result)
}

#[tauri::command]
pub async fn find_service_executable(
    service_name: String,
    ssh_state: State<'_, Arc<Mutex<SshSession>>>,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    let log_message = format!("查找服务 {} 的可执行文件", service_name);
    add_log_internal(&app_state, "INFO", &log_message);

    let mut search_results = Vec::new();

    // 1. 查看服务配置文件内容
    let service_config_cmd = format!("sudo systemctl cat {}", service_name);
    match execute_command(&ssh_state, &service_config_cmd) {
        Ok(config) => {
            search_results.push(format!("📋 当前服务配置:\n{}", config));
        }
        Err(e) => search_results.push(format!("⚠️ 无法读取服务配置: {}", e)),
    }

    // 2. 搜索可能的可执行文件位置
    let search_patterns = vec![
        format!("find /usr/local/bin -name '*{}*' 2>/dev/null", service_name),
        format!("find /usr/bin -name '*{}*' 2>/dev/null", service_name),
        format!("find /opt -name '*{}*' 2>/dev/null", service_name),
        format!("find /home -name '*{}*' 2>/dev/null", service_name),
        "find /ems -type f 2>/dev/null".to_string(),
    ];

    for pattern in search_patterns {
        match execute_command(&ssh_state, &pattern) {
            Ok(output) => {
                if !output.trim().is_empty() {
                    search_results.push(format!("🔍 找到文件:\n{}", output));
                }
            }
            Err(_) => {} // 忽略搜索错误
        }
    }

    // 3. 检查 /ems 目录状态
    let ems_dir_cmd = "ls -la /ems/ 2>/dev/null || echo 'Directory /ems does not exist'";
    match execute_command(&ssh_state, ems_dir_cmd) {
        Ok(output) => {
            search_results.push(format!("📁 /ems 目录内容:\n{}", output));
        }
        Err(e) => search_results.push(format!("⚠️ 无法检查 /ems 目录: {}", e)),
    }

    let result = search_results.join("\n\n");
    add_log_internal(&app_state, "INFO", &format!("完成服务 {} 可执行文件查找", service_name));

    Ok(result)
}

#[tauri::command]
pub async fn force_stop_service(
    name: String,
    ssh_state: State<'_, Arc<Mutex<SshSession>>>,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    let log_message = format!("强制停止服务 {}", name);
    add_log_internal(&app_state, "INFO", &log_message);

    // 1. 停止服务
    let stop_cmd = format!("sudo systemctl stop {}", name);
    let _ = execute_command(&ssh_state, &stop_cmd); // 忽略错误，继续执行

    // 2. 禁用服务自动重启
    let disable_cmd = format!("sudo systemctl disable {}", name);
    let _ = execute_command(&ssh_state, &disable_cmd); // 忽略错误，继续执行

    // 3. 重置失败状态
    let reset_cmd = format!("sudo systemctl reset-failed {}", name);
    let _ = execute_command(&ssh_state, &reset_cmd); // 忽略错误，继续执行

    // 4. 检查最终状态
    let status_cmd = format!("sudo systemctl is-active {} || echo 'inactive'", name);
    let final_status = execute_command(&ssh_state, &status_cmd)?;

    let success_message = format!("服务 {} 已强制停止，当前状态: {}", name, final_status.trim());
    add_log_internal(&app_state, "INFO", &success_message);

    Ok(success_message)
}