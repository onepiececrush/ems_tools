#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod ssh;
mod file_transfer;
mod network;
mod service;
mod logger;

use std::sync::Mutex;
use serde::{Deserialize, Serialize};

// 全局状态
struct AppState {
    log_entries: Mutex<Vec<LogEntry>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SshConnectionConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileTransferConfig {
    local_path: String,
    remote_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct NetworkConfig {
    interface: String,
    ip_address: String,
    netmask: String,
    gateway: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServiceConfig {
    name: String,
    service_file: String,
    enable_autostart: bool,
}

// 初始化日志
fn main() {
    env_logger::init();
    
    // 初始化SSH会话状态
    let ssh_state = ssh::init_ssh_state();

    tauri::Builder::default()
        .manage(AppState {
            log_entries: Mutex::new(Vec::new()),
        })
        .manage(ssh_state)
        .invoke_handler(tauri::generate_handler![
            ssh::connect,
            ssh::disconnect,
            ssh::execute_terminal_command,
            ssh::get_current_directory,
            ssh::check_connection_status,
            file_transfer::upload_file,
            network::get_network_interfaces,
            network::get_interface_config,
            network::set_interface_config,
            service::import_service,
            service::enable_service,
            service::start_service,
            service::stop_service,
            service::get_service_status,
            service::search_services,
            service::diagnose_service,
            service::find_service_executable,
            service::force_stop_service,
            logger::get_logs,
            logger::add_log,
            logger::clear_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
} 