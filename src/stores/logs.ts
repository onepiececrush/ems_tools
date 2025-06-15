import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { ElMessage } from 'element-plus'

export interface LogEntry {
  timestamp: string
  level: string
  message: string
}

export const useLogsStore = defineStore('logs', () => {
  const logs = ref<LogEntry[]>([])
  const isLoading = ref(false)

  const fetchLogs = async () => {
    try {
      isLoading.value = true
      const result = await invoke<LogEntry[]>('get_logs')
      logs.value = result
    } catch (error) {
      console.error('获取日志失败:', error)
      ElMessage.error(`获取日志失败: ${error}`)
    } finally {
      isLoading.value = false
    }
  }

  const addLog = async (level: string, message: string) => {
    try {
      await invoke('add_log', { level, message })
      await fetchLogs()
    } catch (error) {
      console.error('添加日志失败:', error)
    }
  }

  const clearLogs = async () => {
    try {
      await invoke('clear_logs')
      logs.value = []
      ElMessage.success('日志已清空')
    } catch (error) {
      ElMessage.error(`清空日志失败: ${error}`)
    }
  }

  return {
    logs,
    isLoading,
    fetchLogs,
    addLog,
    clearLogs
  }
})
