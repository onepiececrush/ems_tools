import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { ElMessage } from 'element-plus'

export interface SshConfig {
  host: string
  port: number
  username: string
  password: string
}

export interface SshHistoryItem {
  id: string
  name: string
  host: string
  port: number
  username: string
  password: string
  lastUsed: string
  createdAt: string
}

const HISTORY_STORAGE_KEY = 'ssh_connection_history'

export const useSshStore = defineStore('ssh', () => {
  const isConnected = ref(false)
  const connectionConfig = ref<SshConfig | null>(null)
  const connectionStatus = ref('未连接')
  const isConnecting = ref(false)
  const connectionHistory = ref<SshHistoryItem[]>([])

  // 初始化时加载历史记录
  const loadHistory = () => {
    try {
      const stored = localStorage.getItem(HISTORY_STORAGE_KEY)
      if (stored) {
        connectionHistory.value = JSON.parse(stored)
      }
    } catch (error) {
      console.error('加载SSH连接历史失败:', error)
      connectionHistory.value = []
    }
  }

  // 保存历史记录到localStorage
  const saveHistory = () => {
    try {
      localStorage.setItem(HISTORY_STORAGE_KEY, JSON.stringify(connectionHistory.value))
    } catch (error) {
      console.error('保存SSH连接历史失败:', error)
    }
  }

  const connectionInfo = computed(() => {
    if (!isConnected.value || !connectionConfig.value) {
      return '未连接'
    }
    return `已连接到 ${connectionConfig.value.host}:${connectionConfig.value.port}`
  })

  const connect = async (config: SshConfig, saveToHistory = true) => {
    try {
      isConnecting.value = true
      connectionStatus.value = '连接中...'

      await invoke('connect', { config })

      isConnected.value = true
      connectionConfig.value = config
      connectionStatus.value = '已连接'

      // 连接成功后保存到历史记录
      if (saveToHistory) {
        addToHistory(config)
      }

      ElMessage.success('SSH连接成功')
      return true
    } catch (error) {
      connectionStatus.value = `连接失败: ${error}`
      ElMessage.error(`连接失败: ${error}`)
      return false
    } finally {
      isConnecting.value = false
    }
  }

  const disconnect = async () => {
    try {
      console.log('🔧 [SSH-STORE] 开始断开SSH连接')
      await invoke('disconnect')

      isConnected.value = false
      connectionConfig.value = null
      connectionStatus.value = '未连接'

      console.log('✅ [SSH-STORE] SSH连接已断开')
      ElMessage.success('已断开SSH连接')
      return true
    } catch (error) {
      console.error('❌ [SSH-STORE] 断开连接失败:', error)

      // 即使后端断开失败，也清理前端状态
      isConnected.value = false
      connectionConfig.value = null
      connectionStatus.value = '断开连接失败'

      ElMessage.error(`断开连接失败: ${error}`)
      return false
    }
  }

  // 检查连接状态
  const checkConnectionStatus = async () => {
    try {
      const isActive = await invoke('check_connection_status')
      if (!isActive && isConnected.value) {
        // 连接已断开但前端状态未更新
        console.log('🔧 [SSH-STORE] 检测到连接已断开，更新前端状态')
        isConnected.value = false
        connectionConfig.value = null
        connectionStatus.value = '连接已断开'
        ElMessage.warning('SSH连接已断开')
      }
      return isActive
    } catch (error) {
      console.error('❌ [SSH-STORE] 检查连接状态失败:', error)
      return false
    }
  }

  // 添加到历史记录
  const addToHistory = (config: SshConfig, name?: string) => {
    const now = new Date().toISOString()
    const id = `${config.host}_${config.port}_${config.username}_${Date.now()}`

    // 检查是否已存在相同的连接
    const existingIndex = connectionHistory.value.findIndex(
      item => item.host === config.host &&
              item.port === config.port &&
              item.username === config.username
    )

    if (existingIndex >= 0) {
      // 更新现有记录的最后使用时间
      connectionHistory.value[existingIndex].lastUsed = now
    } else {
      // 添加新记录
      const historyItem: SshHistoryItem = {
        id,
        name: name || `${config.username}@${config.host}:${config.port}`,
        host: config.host,
        port: config.port,
        username: config.username,
        password: config.password,
        lastUsed: now,
        createdAt: now
      }

      connectionHistory.value.unshift(historyItem)

      // 限制历史记录数量（最多保存20条）
      if (connectionHistory.value.length > 20) {
        connectionHistory.value = connectionHistory.value.slice(0, 20)
      }
    }

    saveHistory()
  }

  // 从历史记录删除
  const removeFromHistory = (id: string) => {
    connectionHistory.value = connectionHistory.value.filter(item => item.id !== id)
    saveHistory()
  }

  // 清空历史记录
  const clearHistory = () => {
    connectionHistory.value = []
    saveHistory()
  }

  // 从历史记录连接
  const connectFromHistory = async (historyItem: SshHistoryItem) => {
    const config: SshConfig = {
      host: historyItem.host,
      port: historyItem.port,
      username: historyItem.username,
      password: historyItem.password
    }

    // 更新最后使用时间
    historyItem.lastUsed = new Date().toISOString()
    saveHistory()

    return await connect(config, false) // 不重复保存到历史
  }

  // 初始化历史记录
  loadHistory()

  return {
    isConnected,
    connectionConfig,
    connectionStatus,
    connectionInfo,
    isConnecting,
    connectionHistory,
    connect,
    disconnect,
    checkConnectionStatus,
    addToHistory,
    removeFromHistory,
    clearHistory,
    connectFromHistory,
    loadHistory
  }
})
