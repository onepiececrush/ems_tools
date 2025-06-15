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
        let remote_path_str = config.remote_path.clone();
        let log_message = format!("准备上传到远程路径: {}", remote_path_str);
        add_log_internal(&app_state, "INFO", &log_message);

        // 确定目标目录路径
        let target_dir = if remote_path_str.ends_with('/') {
            // 如果以/结尾，说明是目录路径
            remote_path_str.clone()
        } else {
            // 否则获取父目录
            let remote_path = Path::new(&remote_path_str);
            if let Some(parent) = remote_path.parent() {
                format!("{}/", parent.to_string_lossy())
            } else {
                "/tmp/".to_string() // 默认目录
            }
        };

        // 创建目标目录
        let mkdir_cmd = format!("mkdir -p '{}'", target_dir.trim_end_matches('/'));
        let log_message = format!("创建远程目录: {}", target_dir);
        add_log_internal(&app_state, "INFO", &log_message);

        let mut channel = session.channel_session()
            .map_err(|e| format!("无法创建会话通道: {}", e))?;

        channel.exec(&mkdir_cmd)
            .map_err(|e| format!("创建远程目录失败: {}", e))?;

        // 读取命令输出以检查是否成功
        let mut output = String::new();
        if let Err(e) = channel.read_to_string(&mut output) {
            log::warn!("读取mkdir命令输出失败: {}", e);
        }

        // 发送EOF信号
        channel.send_eof()
            .map_err(|e| format!("发送EOF失败: {}", e))?;

        // 等待通道关闭，但添加错误处理
        if let Err(e) = channel.wait_close() {
            log::warn!("文件传输中等待通道关闭时出错: {}", e);
            // 不返回错误，继续执行文件传输
        }

        let exit_status = channel.exit_status().unwrap_or(-1);
        if exit_status != 0 && exit_status != -1 {
            let error_msg = format!("创建远程目录失败，退出代码: {}，输出: {}", exit_status, output);
            add_log_internal(&app_state, "ERROR", &error_msg);
            return Err(error_msg);
        }

        let success_msg = format!("远程目录创建成功: {}", target_dir);
        add_log_internal(&app_state, "INFO", &success_msg);
        
        // 创建SFTP会话
        add_log_internal(&app_state, "INFO", "创建SFTP会话...");
        let sftp = session.sftp()
            .map_err(|e| {
                let error_msg = format!("无法创建SFTP会话: {}", e);
                add_log_internal(&app_state, "ERROR", &error_msg);
                error_msg
            })?;
        add_log_internal(&app_state, "INFO", "SFTP会话创建成功");

        // 检查远程路径是否为目录
        let remote_path_str = config.remote_path.clone();
        if remote_path_str.ends_with('/') {
            // 如果路径以/结尾，说明是目录，需要添加文件名
            let file_name = Path::new(&config.local_path)
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| "无法获取本地文件名".to_string())?;
            let full_remote_path = format!("{}{}", remote_path_str, file_name);
            let remote_path = Path::new(&full_remote_path);

            let log_message = format!("创建远程文件: {}", full_remote_path);
            add_log_internal(&app_state, "INFO", &log_message);

            // 创建远程文件
            let mut remote_file = sftp.create(remote_path)
                .map_err(|e| {
                    let error_msg = format!("无法创建远程文件 '{}': {} (错误代码: {:?})", full_remote_path, e, e);
                    add_log_internal(&app_state, "ERROR", &error_msg);
                    error_msg
                })?;

            // 写入文件内容
            remote_file.write_all(&buffer)
                .map_err(|e| {
                    let error_msg = format!("写入远程文件失败: {}", e);
                    add_log_internal(&app_state, "ERROR", &error_msg);
                    error_msg
                })?;
        } else {
            // 直接使用指定的路径
            let log_message = format!("创建远程文件: {}", remote_path_str);
            add_log_internal(&app_state, "INFO", &log_message);

            let remote_path = Path::new(&remote_path_str);

            // 创建远程文件
            let mut remote_file = sftp.create(remote_path)
                .map_err(|e| {
                    let error_msg = format!("无法创建远程文件 '{}': {} (错误代码: {:?})", remote_path_str, e, e);
                    add_log_internal(&app_state, "ERROR", &error_msg);
                    error_msg
                })?;

            // 写入文件内容
            remote_file.write_all(&buffer)
                .map_err(|e| {
                    let error_msg = format!("写入远程文件失败: {}", e);
                    add_log_internal(&app_state, "ERROR", &error_msg);
                    error_msg
                })?;
        }

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