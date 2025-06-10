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
}

impl SshSession {
    pub fn new() -> Self {
        SshSession {
            session: None,
            config: None,
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
    let connect_result = connect_ssh(&config);
    
    match connect_result {
        Ok(session) => {
            let mut ssh_state = ssh_state.lock().map_err(|_| "Failed to lock SSH state".to_string())?;
            ssh_state.session = Some(session);
            ssh_state.config = Some(config.clone());
            
            let log_message = format!("成功连接到 {}:{}", config.host, config.port);
            add_log_internal(&app_state, "INFO", &log_message);
            
            Ok("连接成功".to_string())
        },
        Err(e) => {
            let error_msg = format!("连接失败: {}", e);
            add_log_internal(&app_state, "ERROR", &error_msg);
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
        ssh_state.session = None;
        ssh_state.config = None;
        
        add_log_internal(&app_state, "INFO", "SSH连接已断开");
        Ok("已断开连接".to_string())
    } else {
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
            
        channel.wait_close()
            .map_err(|e| format!("等待通道关闭失败: {}", e))?;
            
        let exit_status = channel.exit_status()
            .map_err(|e| format!("获取退出状态失败: {}", e))?;
            
        if exit_status != 0 {
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