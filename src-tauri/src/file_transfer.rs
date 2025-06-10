use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tauri::State;
use serde::{Deserialize, Serialize};

use crate::{AppState, FileTransferConfig};
use crate::ssh::SshSession;
use crate::logger::add_log_internal;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileUploadResult {
    success: bool,
    message: String,
    bytes_transferred: usize,
}

#[tauri::command]
pub async fn upload_file(
    config: FileTransferConfig,
    ssh_state: State<'_, Arc<Mutex<SshSession>>>,
    app_state: State<'_, AppState>,
) -> Result<FileUploadResult, String> {
    // 记录开始上传的日志
    let log_message = format!("开始上传文件: {} -> {}", config.local_path, config.remote_path);
    add_log_internal(&app_state, "INFO", &log_message);
    
    let ssh_state = ssh_state.lock().map_err(|_| "Failed to lock SSH state".to_string())?;
    
    if let Some(ref session) = ssh_state.session {
        // 打开本地文件
        let mut local_file = File::open(&config.local_path)
            .map_err(|e| format!("无法打开本地文件: {}", e))?;
            
        // 获取文件大小
        let _file_size = local_file.metadata()
            .map_err(|e| format!("无法获取文件元数据: {}", e))?
            .len() as usize;
            
        // 读取文件内容
        let mut buffer = Vec::new();
        local_file.read_to_end(&mut buffer)
            .map_err(|e| format!("读取文件失败: {}", e))?;
            
        // 确保远程目录存在
        let remote_path = Path::new(&config.remote_path);
        if let Some(parent) = remote_path.parent() {
            let mkdir_cmd = format!("mkdir -p '{}'", parent.display());
            let mut channel = session.channel_session()
                .map_err(|e| format!("无法创建会话通道: {}", e))?;
                
            channel.exec(&mkdir_cmd)
                .map_err(|e| format!("创建远程目录失败: {}", e))?;
                
            channel.wait_close()
                .map_err(|e| format!("等待通道关闭失败: {}", e))?;
        }
        
        // 创建SFTP会话
        let sftp = session.sftp()
            .map_err(|e| format!("无法创建SFTP会话: {}", e))?;
            
        // 创建远程文件
        let mut remote_file = sftp.create(remote_path)
            .map_err(|e| format!("无法创建远程文件: {}", e))?;
            
        // 写入文件内容
        remote_file.write_all(&buffer)
            .map_err(|e| format!("写入远程文件失败: {}", e))?;
            
        // 记录成功上传的日志
        let success_message = format!("文件上传成功: {} -> {}, 大小: {} 字节", 
            config.local_path, config.remote_path, buffer.len());
        add_log_internal(&app_state, "INFO", &success_message);
        
        Ok(FileUploadResult {
            success: true,
            message: "文件上传成功".to_string(),
            bytes_transferred: buffer.len(),
        })
    } else {
        let error_msg = "没有活动的SSH连接";
        add_log_internal(&app_state, "ERROR", error_msg);
        Err(error_msg.to_string())
    }
} 