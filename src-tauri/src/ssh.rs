use std::io::Read;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use ssh2::Session;
use tauri::State;
use anyhow::{Result, anyhow};

use crate::{SshConnectionConfig, AppState};
use crate::logger::add_log_internal;

// 全局SSH会话
pub struct SshSession {
    pub session: Option<Session>,
    pub config: Option<SshConnectionConfig>,
    pub current_directory: String,
}

impl SshSession {
    pub fn new() -> Self {
        SshSession {
            session: None,
            config: None,
            current_directory: "/root".to_string(), // 默认工作目录
        }
    }
}

// 初始化SSH会话状态
pub fn init_ssh_state() -> Arc<Mutex<SshSession>> {
    Arc::new(Mutex::new(SshSession::new()))
}

#[tauri::command]
pub async fn connect(
    config: SshConnectionConfig,
    ssh_state: State<'_, Arc<Mutex<SshSession>>>,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    // 记录连接尝试
    let log_message = format!("尝试连接到 {}:{} (用户: {})", config.host, config.port, config.username);
    add_log_internal(&app_state, "INFO", &log_message);
    println!("🔧 [SSH] {}", log_message);

    let connect_result = connect_ssh(&config);

    match connect_result {
        Ok(session) => {
            let mut ssh_state = ssh_state.lock().map_err(|_| "Failed to lock SSH state".to_string())?;
            ssh_state.session = Some(session);
            ssh_state.config = Some(config.clone());
            ssh_state.current_directory = "/root".to_string(); // 重置工作目录

            let success_msg = format!("成功连接到 {}:{}", config.host, config.port);
            add_log_internal(&app_state, "INFO", &success_msg);
            println!("✅ [SSH] {}", success_msg);

            Ok("连接成功".to_string())
        },
        Err(e) => {
            let error_msg = format!("连接失败: {}", e);
            add_log_internal(&app_state, "ERROR", &error_msg);
            println!("❌ [SSH] {}", error_msg);
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub async fn disconnect(
    ssh_state: State<'_, Arc<Mutex<SshSession>>>,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    let mut ssh_state = ssh_state.lock().map_err(|_| "Failed to lock SSH state".to_string())?;

    if ssh_state.session.is_some() {
        // 记录断开连接开始
        let config_info = if let Some(ref config) = ssh_state.config {
            format!("{}:{}", config.host, config.port)
        } else {
            "未知主机".to_string()
        };

        add_log_internal(&app_state, "INFO", &format!("开始断开SSH连接: {}", config_info));
        println!("🔧 [SSH] 开始断开连接: {}", config_info);

        // 主动断开SSH会话
        if let Some(ref mut session) = ssh_state.session {
            match session.disconnect(None, "User requested disconnection", None) {
                Ok(_) => {
                    println!("✅ [SSH] SSH会话已主动断开");
                    add_log_internal(&app_state, "INFO", "SSH会话已主动断开");
                }
                Err(e) => {
                    println!("⚠️ [SSH] 主动断开SSH会话时出错: {}", e);
                    add_log_internal(&app_state, "WARN", &format!("主动断开SSH会话时出错: {}", e));
                    // 继续执行清理，不返回错误
                }
            }
        }

        // 清理会话状态
        ssh_state.session = None;
        ssh_state.config = None;
        ssh_state.current_directory = "/root".to_string(); // 重置工作目录

        println!("✅ [SSH] SSH连接状态已清理");
        add_log_internal(&app_state, "INFO", "SSH连接已完全断开");
        Ok("已断开连接".to_string())
    } else {
        println!("⚠️ [SSH] 尝试断开连接，但没有活动的SSH连接");
        add_log_internal(&app_state, "WARN", "尝试断开连接，但没有活动的SSH连接");
        Err("没有活动的SSH连接".to_string())
    }
}

// 执行远程命令
pub fn execute_command(
    ssh_state: &Arc<Mutex<SshSession>>,
    command: &str
) -> Result<String, String> {
    let ssh_state = ssh_state.lock().map_err(|_| "Failed to lock SSH state".to_string())?;

    if let Some(ref session) = ssh_state.session {
        let mut channel = session.channel_session()
            .map_err(|e| format!("无法创建会话通道: {}", e))?;

        channel.exec(command)
            .map_err(|e| format!("执行命令失败: {}", e))?;

        let mut output = String::new();
        channel.read_to_string(&mut output)
            .map_err(|e| format!("读取命令输出失败: {}", e))?;

        // 发送EOF信号
        channel.send_eof()
            .map_err(|e| format!("发送EOF失败: {}", e))?;

        // 等待通道关闭，但添加错误处理
        if let Err(e) = channel.wait_close() {
            log::warn!("等待通道关闭时出错: {}", e);
            // 不返回错误，继续执行
        }

        let exit_status = channel.exit_status()
            .unwrap_or(-1); // 如果无法获取退出状态，使用-1

        if exit_status != 0 && exit_status != -1 {
            return Err(format!("命令执行失败，退出代码: {}，输出: {}", exit_status, output));
        }

        Ok(output)
    } else {
        Err("没有活动的SSH连接".to_string())
    }
}

// 连接到SSH服务器
fn connect_ssh(config: &SshConnectionConfig) -> Result<Session, anyhow::Error> {
    let tcp = TcpStream::connect(format!("{}:{}", config.host, config.port))?;
    let mut session = Session::new()?;
    
    session.set_tcp_stream(tcp);
    session.handshake()?;
    
    session.userauth_password(&config.username, &config.password)?;
    
    if !session.authenticated() {
        return Err(anyhow!("认证失败"));
    }
    
    Ok(session)
}

// 转换常用别名为实际命令
fn convert_aliases(command: &str) -> String {
    let trimmed = command.trim();

    // 常用别名映射
    match trimmed {
        "ll" => "ls -l".to_string(),
        "la" => "ls -la".to_string(),
        "l" => "ls -CF".to_string(),
        cmd if cmd.starts_with("ll ") => {
            // 处理带参数的ll命令，如 "ll /home"
            format!("ls -l {}", &cmd[3..])
        },
        cmd if cmd.starts_with("la ") => {
            // 处理带参数的la命令，如 "la /home"
            format!("ls -la {}", &cmd[3..])
        },
        _ => command.to_string()
    }
}

// 简单的终端命令执行（用于终端界面）
#[tauri::command]
pub async fn execute_terminal_command(
    command: String,
    ssh_state: State<'_, Arc<Mutex<SshSession>>>,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    let log_message = format!("执行终端命令: {}", command);
    add_log_internal(&app_state, "INFO", &log_message);
    println!("🔧 [TERMINAL] 执行命令: '{}'", command);

    // 检查是否是cd命令
    if command.trim().starts_with("cd ") || command.trim() == "cd" {
        return handle_cd_command(command.trim(), &ssh_state, &app_state).await;
    }

    // 转换别名
    let converted_command = convert_aliases(&command);
    if converted_command != command {
        println!("🔄 [TERMINAL] 别名转换: '{}' -> '{}'", command, converted_command);
    }

    // 对于其他命令，先切换到当前工作目录再执行
    let current_dir = {
        let ssh_guard = ssh_state.lock().map_err(|_| "无法获取SSH状态锁".to_string())?;
        ssh_guard.current_directory.clone()
    };

    let full_command = if current_dir != "/root" {
        format!("cd '{}' && {}", current_dir, converted_command)
    } else {
        converted_command.clone()
    };

    println!("🔧 [TERMINAL] 实际执行命令: '{}'", full_command);
    let result = execute_command(&ssh_state, &full_command);

    match result {
        Ok(output) => {
            println!("✅ [TERMINAL] 命令执行成功，输出: '{}'", output.replace('\n', "\\n"));
            Ok(output)
        }
        Err(error) => {
            println!("❌ [TERMINAL] 命令执行失败: {}", error);

            // 提供更友好的错误提示
            if error.contains("退出代码: 127") {
                if command.trim() == "ll" || command.trim().starts_with("ll ") {
                    let suggestion = "提示：'ll' 命令不存在，已自动转换为 'ls -l'。请检查目录是否存在或权限是否足够。";
                    println!("💡 [TERMINAL] {}", suggestion);
                    Err(format!("bash: ll: command not found\n{}", suggestion))
                } else {
                    let suggestion = format!("提示：命令 '{}' 不存在，请检查拼写或使用 'which {}' 查看命令是否安装。", command.trim(), command.trim());
                    println!("💡 [TERMINAL] {}", suggestion);
                    Err(format!("bash: {}: command not found\n{}", command.trim(), suggestion))
                }
            } else {
                Err(error)
            }
        }
    }
}

// 处理cd命令
async fn handle_cd_command(
    command: &str,
    ssh_state: &State<'_, Arc<Mutex<SshSession>>>,
    app_state: &State<'_, AppState>,
) -> Result<String, String> {
    println!("🔧 [TERMINAL] 处理cd命令: '{}'", command);

    let target_dir = if command.trim() == "cd" {
        // 无参数的cd，切换到home目录
        "/root".to_string()
    } else {
        // 提取目标目录
        let parts: Vec<&str> = command.trim().splitn(2, ' ').collect();
        if parts.len() < 2 {
            return Err("cd命令格式错误".to_string());
        }
        parts[1].trim().to_string()
    };

    // 获取当前工作目录
    let current_dir = {
        let ssh_guard = ssh_state.lock().map_err(|_| "无法获取SSH状态锁".to_string())?;
        ssh_guard.current_directory.clone()
    };

    // 保存原始target_dir用于错误消息
    let original_target = target_dir.clone();

    // 计算新的绝对路径
    let new_dir = if target_dir.starts_with('/') {
        // 绝对路径
        target_dir
    } else if target_dir == ".." {
        // 上级目录
        if current_dir == "/" {
            "/".to_string()
        } else {
            let mut parts: Vec<&str> = current_dir.split('/').filter(|s| !s.is_empty()).collect();
            if !parts.is_empty() {
                parts.pop();
            }
            if parts.is_empty() {
                "/".to_string()
            } else {
                format!("/{}", parts.join("/"))
            }
        }
    } else if target_dir == "." {
        // 当前目录
        current_dir.clone()
    } else {
        // 相对路径
        if current_dir == "/" {
            format!("/{}", target_dir)
        } else {
            format!("{}/{}", current_dir, target_dir)
        }
    };

    println!("🔧 [TERMINAL] 计算新目录: '{}' -> '{}'", current_dir, new_dir);

    // 验证目录是否存在
    let test_command = format!("test -d '{}' && echo 'EXISTS' || echo 'NOT_EXISTS'", new_dir);
    let test_result = execute_command(ssh_state, &test_command)?;

    if test_result.trim() != "EXISTS" {
        return Err(format!("bash: cd: {}: No such file or directory", original_target));
    }

    // 更新当前工作目录
    {
        let mut ssh_guard = ssh_state.lock().map_err(|_| "无法获取SSH状态锁".to_string())?;
        ssh_guard.current_directory = new_dir.clone();
    }

    println!("✅ [TERMINAL] cd命令成功，新工作目录: '{}'", new_dir);
    add_log_internal(app_state, "INFO", &format!("工作目录已切换到: {}", new_dir));

    Ok("".to_string()) // cd命令通常没有输出
}

// 获取当前工作目录
#[tauri::command]
pub async fn get_current_directory(
    ssh_state: State<'_, Arc<Mutex<SshSession>>>,
) -> Result<String, String> {
    let ssh_guard = ssh_state.lock().map_err(|_| "无法获取SSH状态锁".to_string())?;
    Ok(ssh_guard.current_directory.clone())
}

// 检查SSH连接状态
#[tauri::command]
pub async fn check_connection_status(
    ssh_state: State<'_, Arc<Mutex<SshSession>>>,
    app_state: State<'_, AppState>,
) -> Result<bool, String> {
    let ssh_guard = ssh_state.lock().map_err(|_| "无法获取SSH状态锁".to_string())?;

    if let Some(ref _session) = ssh_guard.session {
        // 尝试执行一个简单的命令来验证连接状态
        drop(ssh_guard); // 释放锁以便execute_command使用

        match execute_command(&ssh_state, "echo 'connection_test'") {
            Ok(output) => {
                let is_connected = output.trim() == "connection_test";
                if is_connected {
                    add_log_internal(&app_state, "INFO", "SSH连接状态检查: 连接正常");
                } else {
                    add_log_internal(&app_state, "WARN", "SSH连接状态检查: 连接异常");
                }
                Ok(is_connected)
            }
            Err(_) => {
                add_log_internal(&app_state, "WARN", "SSH连接状态检查: 连接已断开");
                Ok(false)
            }
        }
    } else {
        add_log_internal(&app_state, "INFO", "SSH连接状态检查: 无活动连接");
        Ok(false)
    }
}