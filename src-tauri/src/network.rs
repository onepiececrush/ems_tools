use std::sync::{Arc, Mutex};
use tauri::State;
use serde::{Deserialize, Serialize};

use crate::{AppState, NetworkConfig};
use crate::ssh::{SshSession, execute_command};
use crate::logger::add_log_internal;

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub status: String,
    pub mac_address: Option<String>,
    pub ip_address: Option<String>,
    pub netmask: Option<String>,
}

#[tauri::command]
pub async fn get_network_interfaces(
    ssh_state: State<'_, Arc<Mutex<SshSession>>>,
    app_state: State<'_, AppState>,
) -> Result<Vec<NetworkInterface>, String> {
    add_log_internal(&app_state, "INFO", "获取网络接口列表");
    
    // 执行命令获取网络接口信息
    let output = execute_command(&ssh_state, "ip -o addr show")?;
    
    // 解析输出
    let interfaces = parse_ip_addr_output(&output);
    
    add_log_internal(&app_state, "INFO", &format!("找到 {} 个网络接口", interfaces.len()));
    
    Ok(interfaces)
}

#[tauri::command]
pub async fn get_interface_config(
    interface: String,
    ssh_state: State<'_, Arc<Mutex<SshSession>>>,
    app_state: State<'_, AppState>,
) -> Result<NetworkConfig, String> {
    let log_message = format!("获取网络接口 {} 的配置", interface);
    add_log_internal(&app_state, "INFO", &log_message);
    
    // 获取IP地址和子网掩码
    let ip_cmd = format!("ip -o addr show {} | grep -v 'inet6' | awk '{{print $4}}'", interface);
    let ip_output = execute_command(&ssh_state, &ip_cmd)?;
    
    // 获取默认网关
    let gateway_cmd = format!("ip route | grep default | grep {} | awk '{{print $3}}'", interface);
    let gateway_output = execute_command(&ssh_state, &gateway_cmd).unwrap_or_else(|_| "".to_string());
    
    // 解析IP地址和子网掩码
    let ip_parts: Vec<&str> = ip_output.trim().split('/').collect();
    if ip_parts.len() != 2 {
        return Err(format!("无法解析接口 {} 的IP配置", interface));
    }
    
    let ip_address = ip_parts[0].to_string();
    let prefix_len = ip_parts[1].parse::<u8>().map_err(|_| "无法解析子网前缀长度".to_string())?;
    let netmask = prefix_to_netmask(prefix_len)?;
    
    let gateway = if gateway_output.trim().is_empty() {
        None
    } else {
        Some(gateway_output.trim().to_string())
    };
    
    let config = NetworkConfig {
        interface: interface.clone(),
        ip_address,
        netmask,
        gateway,
    };
    
    add_log_internal(&app_state, "INFO", &format!("成功获取接口 {} 的配置", interface));
    
    Ok(config)
}

#[tauri::command]
pub async fn set_interface_config(
    config: NetworkConfig,
    ssh_state: State<'_, Arc<Mutex<SshSession>>>,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    let log_message = format!("设置网络接口 {} 的配置", config.interface);
    add_log_internal(&app_state, "INFO", &log_message);
    
    // 先关闭接口
    let down_cmd = format!("ip link set {} down", config.interface);
    execute_command(&ssh_state, &down_cmd)?;
    
    // 设置IP地址和子网掩码
    let netmask_prefix = netmask_to_prefix(&config.netmask)?;
    let ip_cmd = format!("ip addr add {}/{} dev {}", 
        config.ip_address, netmask_prefix, config.interface);
    
    // 先清除旧的IP地址
    let flush_cmd = format!("ip addr flush dev {}", config.interface);
    execute_command(&ssh_state, &flush_cmd)?;
    
    // 设置新的IP地址
    execute_command(&ssh_state, &ip_cmd)?;
    
    // 如果提供了网关，则设置默认路由
    if let Some(gateway) = &config.gateway {
        // 删除旧的默认路由
        let del_route_cmd = format!("ip route del default dev {} 2>/dev/null || true", config.interface);
        execute_command(&ssh_state, &del_route_cmd).ok();
        
        // 添加新的默认路由
        let route_cmd = format!("ip route add default via {} dev {}", gateway, config.interface);
        execute_command(&ssh_state, &route_cmd)?;
    }
    
    // 启用接口
    let up_cmd = format!("ip link set {} up", config.interface);
    execute_command(&ssh_state, &up_cmd)?;
    
    let success_message = format!("成功配置网络接口 {}", config.interface);
    add_log_internal(&app_state, "INFO", &success_message);
    
    Ok(success_message)
}

// 辅助函数：解析ip addr命令的输出
fn parse_ip_addr_output(output: &str) -> Vec<NetworkInterface> {
    let mut interfaces = Vec::new();
    let mut current_interface: Option<NetworkInterface> = None;
    
    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }
        
        // 提取接口名称
        let interface_name = parts[1].trim_end_matches(':');
        
        // 检查是否是新接口
        if current_interface.is_none() || current_interface.as_ref().unwrap().name != interface_name {
            // 如果有之前的接口，添加到列表
            if let Some(interface) = current_interface {
                interfaces.push(interface);
            }
            
            // 创建新接口
            current_interface = Some(NetworkInterface {
                name: interface_name.to_string(),
                status: "unknown".to_string(),
                mac_address: None,
                ip_address: None,
                netmask: None,
            });
        }
        
        // 更新接口状态
        if let Some(ref mut interface) = current_interface {
            // 检查是否包含IP地址信息
            if line.contains("inet ") {
                for (i, part) in parts.iter().enumerate() {
                    if *part == "inet" && i + 1 < parts.len() {
                        let ip_info = parts[i + 1];
                        let ip_parts: Vec<&str> = ip_info.split('/').collect();
                        
                        if ip_parts.len() == 2 {
                            interface.ip_address = Some(ip_parts[0].to_string());
                            
                            if let Ok(prefix) = ip_parts[1].parse::<u8>() {
                                if let Ok(netmask) = prefix_to_netmask(prefix) {
                                    interface.netmask = Some(netmask);
                                }
                            }
                        }
                    }
                }
            }
            
            // 检查接口状态
            if line.contains("UP") {
                interface.status = "up".to_string();
            } else if line.contains("DOWN") {
                interface.status = "down".to_string();
            }
            
            // 尝试提取MAC地址
            for (i, part) in parts.iter().enumerate() {
                if (*part == "link/ether" || *part == "ether") && i + 1 < parts.len() {
                    interface.mac_address = Some(parts[i + 1].to_string());
                }
            }
        }
    }
    
    // 添加最后一个接口
    if let Some(interface) = current_interface {
        interfaces.push(interface);
    }
    
    interfaces
}

// 辅助函数：将CIDR前缀转换为子网掩码
fn prefix_to_netmask(prefix: u8) -> Result<String, String> {
    if prefix > 32 {
        return Err("无效的前缀长度".to_string());
    }
    
    let mask = !0u32 << (32 - prefix);
    let a = (mask >> 24) & 0xff;
    let b = (mask >> 16) & 0xff;
    let c = (mask >> 8) & 0xff;
    let d = mask & 0xff;
    
    Ok(format!("{}.{}.{}.{}", a, b, c, d))
}

// 辅助函数：将子网掩码转换为CIDR前缀
fn netmask_to_prefix(netmask: &str) -> Result<u8, String> {
    let parts: Vec<&str> = netmask.split('.').collect();
    if parts.len() != 4 {
        return Err("无效的子网掩码格式".to_string());
    }
    
    let mut prefix = 0;
    for part in parts {
        let octet = part.parse::<u8>().map_err(|_| "无效的子网掩码值".to_string())?;
        
        let mut bits = 0;
        let mut val = octet;
        while val > 0 {
            bits += val & 1;
            val >>= 1;
        }
        
        prefix += bits;
    }
    
    Ok(prefix)
} 