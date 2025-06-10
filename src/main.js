console.log("EMS Tools Application Starting...");

// Tauri API 通过 window.__TAURI__ 对象暴露
const { invoke, dialog, window: appWindow } = window.__TAURI__;

// 全局状态
let isConnected = false;
let currentService = null;
let selectedInterface = null;

// 页面加载完成后初始化
document.addEventListener("DOMContentLoaded", () => {
  // 初始化标签页切换
  initTabs();
  
  // 初始化SSH连接表单
  initSshForm();
  
  // 初始化文件传输功能
  initFileTransfer();
  
  // 初始化网络配置功能
  initNetworkConfig();
  
  // 初始化服务管理功能
  initServiceManagement();
  
  // 初始化日志功能
  initLogs();
  
  // 定期刷新日志
  setInterval(refreshLogs, 10000);
});

// 初始化标签页切换
function initTabs() {
  const tabLinks = document.querySelectorAll(".sidebar nav ul li a");
  const tabPanels = document.querySelectorAll(".tab-panel");
  
  tabLinks.forEach(link => {
    link.addEventListener("click", (e) => {
      e.preventDefault();
      
      // 移除所有活动状态
      tabLinks.forEach(l => l.classList.remove("active"));
      tabPanels.forEach(p => p.classList.remove("active"));
      
      // 设置当前标签为活动状态
      const tab = link.getAttribute("data-tab");
      link.classList.add("active");
      document.getElementById(`${tab}-panel`).classList.add("active");
    });
  });
  
  // 服务管理子标签页
  const serviceTabs = document.querySelectorAll(".service-tab");
  const servicePanels = document.querySelectorAll(".service-panel");
  
  serviceTabs.forEach(tab => {
    tab.addEventListener("click", () => {
      serviceTabs.forEach(t => t.classList.remove("active"));
      servicePanels.forEach(p => p.classList.remove("active"));
      
      const serviceTab = tab.getAttribute("data-service-tab");
      tab.classList.add("active");
      document.getElementById(`${serviceTab}-service-panel`).classList.add("active");
    });
  });
}

// 初始化SSH连接表单
function initSshForm() {
  const sshForm = document.getElementById("ssh-form");
  const connectBtn = document.getElementById("connect-btn");
  const disconnectBtn = document.getElementById("disconnect-btn");
  const statusIndicator = document.getElementById("status-indicator");
  const statusText = document.getElementById("status-text");
  const connectionInfo = document.getElementById("connection-info");
  
  sshForm.addEventListener("submit", async (e) => {
    e.preventDefault();
    
    if (isConnected) return;
    
    const host = document.getElementById("host").value;
    const port = parseInt(document.getElementById("port").value);
    const username = document.getElementById("username").value;
    const password = document.getElementById("password").value;
    
    connectBtn.disabled = true;
    statusText.textContent = "连接中...";
    
    try {
      const result = await invoke("connect", {
        config: { host, port, username, password }
      });
      
      isConnected = true;
      statusIndicator.className = "status-indicator status-connected";
      statusText.textContent = "已连接";
      connectionInfo.textContent = `已连接到 ${host}:${port}`;
      
      connectBtn.disabled = true;
      disconnectBtn.disabled = false;
      
      // 启用其他功能
      document.getElementById("upload-btn").disabled = false;
      document.getElementById("refresh-interfaces-btn").disabled = false;
      
      // 刷新网络接口列表
      refreshNetworkInterfaces();
      
      // 添加日志
      await invoke("add_log", {
        level: "INFO",
        message: `用户 ${username} 连接到 ${host}:${port}`
      });
      
      // 刷新日志
      refreshLogs();
    } catch (error) {
      statusText.textContent = `连接失败: ${error}`;
      connectBtn.disabled = false;
      
      await invoke("add_log", {
        level: "ERROR",
        message: `连接失败: ${error}`
      });
      
      refreshLogs();
    }
  });
  
  disconnectBtn.addEventListener("click", async () => {
    if (!isConnected) return;
    
    try {
      await invoke("disconnect");
      
      isConnected = false;
      statusIndicator.className = "status-indicator status-disconnected";
      statusText.textContent = "未连接";
      connectionInfo.textContent = "未连接";
      
      connectBtn.disabled = false;
      disconnectBtn.disabled = true;
      
      // 禁用其他功能
      document.getElementById("upload-btn").disabled = true;
      document.getElementById("refresh-interfaces-btn").disabled = true;
      
      // 添加日志
      await invoke("add_log", {
        level: "INFO",
        message: "已断开SSH连接"
      });
      
      // 刷新日志
      refreshLogs();
    } catch (error) {
      console.error("断开连接失败:", error);
    }
  });
}

// 初始化文件传输功能
function initFileTransfer() {
  const fileTransferForm = document.getElementById("file-transfer-form");
  const selectFileBtn = document.getElementById("select-file-btn");
  const localFilePath = document.getElementById("local-file-path");
  const uploadBtn = document.getElementById("upload-btn");
  const progressBar = document.querySelector(".upload-progress");
  const progressFill = document.querySelector(".progress-fill");
  const progressText = document.querySelector(".progress-text");
  
  // 禁用上传按钮，直到连接建立
  uploadBtn.disabled = true;
  
  selectFileBtn.addEventListener("click", async () => {
    try {
      const selected = await dialog.open({
        multiple: false,
        filters: [{
          name: "All Files",
          extensions: ["*"]
        }]
      });
      
      if (selected) {
        localFilePath.value = selected;
      }
    } catch (error) {
      console.error("选择文件失败:", error);
    }
  });
  
  fileTransferForm.addEventListener("submit", async (e) => {
    e.preventDefault();
    
    if (!isConnected) return;
    
    const localPath = localFilePath.value;
    const remotePath = document.getElementById("remote-path").value;
    
    if (!localPath || !remotePath) {
      alert("请选择本地文件和指定远程路径");
      return;
    }
    
    uploadBtn.disabled = true;
    progressBar.style.display = "block";
    progressFill.style.width = "0%";
    progressText.textContent = "0%";
    
    try {
      // 模拟上传进度
      let progress = 0;
      const interval = setInterval(() => {
        progress += 5;
        if (progress > 90) clearInterval(interval);
        
        progressFill.style.width = `${progress}%`;
        progressText.textContent = `${progress}%`;
      }, 200);
      
      const result = await invoke("upload_file", {
        config: { localPath, remotePath }
      });
      
      clearInterval(interval);
      progressFill.style.width = "100%";
      progressText.textContent = "100%";
      
      setTimeout(() => {
        progressBar.style.display = "none";
        uploadBtn.disabled = false;
      }, 1000);
      
      // 添加日志
      await invoke("add_log", {
        level: "INFO",
        message: `文件上传成功: ${localPath} -> ${remotePath}`
      });
      
      // 刷新日志
      refreshLogs();
      
      alert("文件上传成功");
    } catch (error) {
      progressBar.style.display = "none";
      uploadBtn.disabled = false;
      
      // 添加日志
      await invoke("add_log", {
        level: "ERROR",
        message: `文件上传失败: ${error}`
      });
      
      // 刷新日志
      refreshLogs();
      
      alert(`文件上传失败: ${error}`);
    }
  });
}

// 初始化网络配置功能
function initNetworkConfig() {
  const refreshInterfacesBtn = document.getElementById("refresh-interfaces-btn");
  const interfaceSelect = document.getElementById("interface-select");
  const interfaceConfig = document.querySelector(".interface-config");
  const networkConfigForm = document.getElementById("network-config-form");
  
  // 禁用刷新按钮，直到连接建立
  refreshInterfacesBtn.disabled = true;
  
  refreshInterfacesBtn.addEventListener("click", () => {
    refreshNetworkInterfaces();
  });
  
  interfaceSelect.addEventListener("change", async () => {
    const interfaceName = interfaceSelect.value;
    
    if (!interfaceName) {
      interfaceConfig.style.display = "none";
      selectedInterface = null;
      return;
    }
    
    try {
      const config = await invoke("get_interface_config", { interface: interfaceName });
      
      document.getElementById("ip-address").value = config.ip_address;
      document.getElementById("netmask").value = config.netmask;
      document.getElementById("gateway").value = config.gateway || "";
      
      selectedInterface = interfaceName;
      interfaceConfig.style.display = "block";
    } catch (error) {
      console.error("获取接口配置失败:", error);
      interfaceConfig.style.display = "none";
      
      // 添加日志
      await invoke("add_log", {
        level: "ERROR",
        message: `获取网络接口 ${interfaceName} 配置失败: ${error}`
      });
      
      // 刷新日志
      refreshLogs();
      
      alert(`获取接口配置失败: ${error}`);
    }
  });
  
  networkConfigForm.addEventListener("submit", async (e) => {
    e.preventDefault();
    
    if (!selectedInterface) return;
    
    const ipAddress = document.getElementById("ip-address").value;
    const netmask = document.getElementById("netmask").value;
    const gateway = document.getElementById("gateway").value;
    
    try {
      const result = await invoke("set_interface_config", {
        config: {
          interface: selectedInterface,
          ip_address: ipAddress,
          netmask: netmask,
          gateway: gateway || null
        }
      });
      
      // 添加日志
      await invoke("add_log", {
        level: "INFO",
        message: `成功配置网络接口 ${selectedInterface}`
      });
      
      // 刷新日志
      refreshLogs();
      
      alert(`网络配置已保存: ${result}`);
    } catch (error) {
      console.error("保存网络配置失败:", error);
      
      // 添加日志
      await invoke("add_log", {
        level: "ERROR",
        message: `配置网络接口 ${selectedInterface} 失败: ${error}`
      });
      
      // 刷新日志
      refreshLogs();
      
      alert(`保存网络配置失败: ${error}`);
    }
  });
}

// 刷新网络接口列表
async function refreshNetworkInterfaces() {
  const interfaceSelect = document.getElementById("interface-select");
  const interfaceConfig = document.querySelector(".interface-config");
  
  try {
    const interfaces = await invoke("get_network_interfaces");
    
    // 清空选择框
    interfaceSelect.innerHTML = '<option value="">-- 选择网络接口 --</option>';
    
    // 添加接口选项
    interfaces.forEach(iface => {
      const option = document.createElement("option");
      option.value = iface.name;
      option.textContent = `${iface.name} (${iface.status}) ${iface.ip_address || ""}`;
      interfaceSelect.appendChild(option);
    });
    
    // 隐藏配置面板
    interfaceConfig.style.display = "none";
    selectedInterface = null;
  } catch (error) {
    console.error("获取网络接口列表失败:", error);
    
    // 添加日志
    await invoke("add_log", {
      level: "ERROR",
      message: `获取网络接口列表失败: ${error}`
    });
    
    // 刷新日志
    refreshLogs();
  }
}

// 初始化服务管理功能
function initServiceManagement() {
  const checkServiceBtn = document.getElementById("check-service-btn");
  const serviceNameInput = document.getElementById("service-name");
  const serviceStatus = document.querySelector(".service-status");
  const startServiceBtn = document.getElementById("start-service-btn");
  const stopServiceBtn = document.getElementById("stop-service-btn");
  const enableServiceBtn = document.getElementById("enable-service-btn");
  const disableServiceBtn = document.getElementById("disable-service-btn");
  
  const importServiceForm = document.getElementById("import-service-form");
  
  checkServiceBtn.addEventListener("click", async () => {
    const serviceName = serviceNameInput.value.trim();
    
    if (!serviceName) {
      alert("请输入服务名称");
      return;
    }
    
    try {
      const status = await invoke("get_service_status", { name: serviceName });
      
      document.getElementById("service-status-text").textContent = status.status;
      document.getElementById("service-enabled-text").textContent = status.enabled ? "已启用" : "已禁用";
      
      serviceStatus.style.display = "block";
      currentService = serviceName;
      
      // 添加日志
      await invoke("add_log", {
        level: "INFO",
        message: `查询服务 ${serviceName} 状态: ${status.status}, 自启动: ${status.enabled ? "已启用" : "已禁用"}`
      });
      
      // 刷新日志
      refreshLogs();
    } catch (error) {
      console.error("获取服务状态失败:", error);
      serviceStatus.style.display = "none";
      
      // 添加日志
      await invoke("add_log", {
        level: "ERROR",
        message: `获取服务 ${serviceName} 状态失败: ${error}`
      });
      
      // 刷新日志
      refreshLogs();
      
      alert(`获取服务状态失败: ${error}`);
    }
  });
  
  startServiceBtn.addEventListener("click", async () => {
    if (!currentService) return;
    
    try {
      await invoke("start_service", { name: currentService });
      
      // 刷新服务状态
      checkServiceBtn.click();
      
      // 添加日志
      await invoke("add_log", {
        level: "INFO",
        message: `启动服务 ${currentService}`
      });
      
      // 刷新日志
      refreshLogs();
      
      alert(`服务 ${currentService} 已启动`);
    } catch (error) {
      console.error("启动服务失败:", error);
      
      // 添加日志
      await invoke("add_log", {
        level: "ERROR",
        message: `启动服务 ${currentService} 失败: ${error}`
      });
      
      // 刷新日志
      refreshLogs();
      
      alert(`启动服务失败: ${error}`);
    }
  });
  
  stopServiceBtn.addEventListener("click", async () => {
    if (!currentService) return;
    
    try {
      await invoke("stop_service", { name: currentService });
      
      // 刷新服务状态
      checkServiceBtn.click();
      
      // 添加日志
      await invoke("add_log", {
        level: "INFO",
        message: `停止服务 ${currentService}`
      });
      
      // 刷新日志
      refreshLogs();
      
      alert(`服务 ${currentService} 已停止`);
    } catch (error) {
      console.error("停止服务失败:", error);
      
      // 添加日志
      await invoke("add_log", {
        level: "ERROR",
        message: `停止服务 ${currentService} 失败: ${error}`
      });
      
      // 刷新日志
      refreshLogs();
      
      alert(`停止服务失败: ${error}`);
    }
  });
  
  enableServiceBtn.addEventListener("click", async () => {
    if (!currentService) return;
    
    try {
      await invoke("enable_service", { name: currentService, enable: true });
      
      // 刷新服务状态
      checkServiceBtn.click();
      
      // 添加日志
      await invoke("add_log", {
        level: "INFO",
        message: `启用服务 ${currentService} 的自启动`
      });
      
      // 刷新日志
      refreshLogs();
      
      alert(`服务 ${currentService} 已启用自启动`);
    } catch (error) {
      console.error("启用服务自启动失败:", error);
      
      // 添加日志
      await invoke("add_log", {
        level: "ERROR",
        message: `启用服务 ${currentService} 的自启动失败: ${error}`
      });
      
      // 刷新日志
      refreshLogs();
      
      alert(`启用服务自启动失败: ${error}`);
    }
  });
  
  disableServiceBtn.addEventListener("click", async () => {
    if (!currentService) return;
    
    try {
      await invoke("enable_service", { name: currentService, enable: false });
      
      // 刷新服务状态
      checkServiceBtn.click();
      
      // 添加日志
      await invoke("add_log", {
        level: "INFO",
        message: `禁用服务 ${currentService} 的自启动`
      });
      
      // 刷新日志
      refreshLogs();
      
      alert(`服务 ${currentService} 已禁用自启动`);
    } catch (error) {
      console.error("禁用服务自启动失败:", error);
      
      // 添加日志
      await invoke("add_log", {
        level: "ERROR",
        message: `禁用服务 ${currentService} 的自启动失败: ${error}`
      });
      
      // 刷新日志
      refreshLogs();
      
      alert(`禁用服务自启动失败: ${error}`);
    }
  });
  
  importServiceForm.addEventListener("submit", async (e) => {
    e.preventDefault();
    
    const serviceName = document.getElementById("service-file-name").value.trim();
    const servicePath = document.getElementById("service-file-path").value.trim();
    const serviceContent = document.getElementById("service-content").value.trim();
    const enableAutostart = document.getElementById("enable-autostart").checked;
    
    if (!serviceName || !servicePath || !serviceContent) {
      alert("请填写所有必填字段");
      return;
    }
    
    try {
      await invoke("import_service", {
        config: {
          name: serviceName,
          service_file: `${servicePath}${serviceName}.service`,
          enable_autostart: enableAutostart
        },
        service_content: serviceContent
      });
      
      // 添加日志
      await invoke("add_log", {
        level: "INFO",
        message: `导入服务 ${serviceName}`
      });
      
      // 刷新日志
      refreshLogs();
      
      alert(`服务 ${serviceName} 已成功导入`);
      
      // 清空表单
      document.getElementById("service-file-name").value = "";
      document.getElementById("service-content").value = "";
      document.getElementById("enable-autostart").checked = false;
    } catch (error) {
      console.error("导入服务失败:", error);
      
      // 添加日志
      await invoke("add_log", {
        level: "ERROR",
        message: `导入服务 ${serviceName} 失败: ${error}`
      });
      
      // 刷新日志
      refreshLogs();
      
      alert(`导入服务失败: ${error}`);
    }
  });
}

// 初始化日志功能
function initLogs() {
  const refreshLogsBtn = document.getElementById("refresh-logs-btn");
  const clearLogsBtn = document.getElementById("clear-logs-btn");
  
  refreshLogsBtn.addEventListener("click", () => {
    refreshLogs();
  });
  
  clearLogsBtn.addEventListener("click", async () => {
    try {
      await invoke("clear_logs");
      refreshLogs();
    } catch (error) {
      console.error("清空日志失败:", error);
    }
  });
  
  // 初始加载日志
  refreshLogs();
}

// 刷新日志列表
async function refreshLogs() {
  try {
    const logs = await invoke("get_logs");
    const logEntries = document.getElementById("log-entries");
    
    // 清空日志列表
    logEntries.innerHTML = "";
    
    // 添加日志条目
    logs.forEach(log => {
      const row = document.createElement("tr");
      
      const timestampCell = document.createElement("td");
      timestampCell.textContent = log.timestamp;
      row.appendChild(timestampCell);
      
      const levelCell = document.createElement("td");
      levelCell.textContent = log.level;
      levelCell.className = `log-level-${log.level}`;
      row.appendChild(levelCell);
      
      const messageCell = document.createElement("td");
      messageCell.textContent = log.message;
      row.appendChild(messageCell);
      
      logEntries.appendChild(row);
    });
    
    // 滚动到底部
    const logContainer = document.querySelector(".log-container");
    logContainer.scrollTop = logContainer.scrollHeight;
  } catch (error) {
    console.error("获取日志失败:", error);
  }
} 