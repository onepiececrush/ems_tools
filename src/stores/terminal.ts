import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { ElMessage } from 'element-plus'

// 终端输出类型枚举
enum OutputType {
  COMMAND = 'command',      // 用户输入的命令
  OUTPUT = 'output',        // 系统输出结果
  ERROR = 'error',          // 错误信息
  PROMPT = 'prompt',        // 提示符
  COMMENT = 'comment'       // 注释/系统消息
}

export const useTerminalStore = defineStore('terminal', () => {
  // 终端状态
  const terminalOutput = ref('<span class="terminal-comment">欢迎使用EMS终端控制台</span>\n<span class="terminal-prompt">$ </span>')
  const commandHistory = ref<string[]>([])
  const historyIndex = ref(-1)
  const currentDirectory = ref('/root')
  const isExecuting = ref(false)
  const isInitialized = ref(false) // 标记是否已初始化

  // 添加输出到终端
  const addOutput = (output: string) => {
    terminalOutput.value += output
  }

  // 添加带类型标记的输出到终端
  const addTypedOutput = (output: string, type: OutputType) => {
    const escapedOutput = output.replace(/</g, '&lt;').replace(/>/g, '&gt;')
    terminalOutput.value += `<span class="terminal-${type}">${escapedOutput}</span>`
  }

  // 添加命令到历史记录
  const addToHistory = (command: string) => {
    if (commandHistory.value[commandHistory.value.length - 1] !== command) {
      commandHistory.value.push(command)
    }
    historyIndex.value = commandHistory.value.length
  }

  // 清屏
  const clearTerminal = () => {
    terminalOutput.value = ''
    addTypedOutput(`${currentDirectory.value}$ `, OutputType.PROMPT)
  }

  // 更新当前工作目录
  const updateCurrentDirectory = async () => {
    try {
      const dir = await invoke<string>('get_current_directory')
      currentDirectory.value = dir
      return dir
    } catch (error) {
      console.warn('⚠️ [TERMINAL-STORE] 获取工作目录失败:', error)
      return currentDirectory.value
    }
  }

  // 执行单个命令
  const executeCommand = async (command: string) => {
    if (!command.trim()) return

    try {
      isExecuting.value = true
      console.log('🔧 [TERMINAL-STORE] 执行命令:', command)

      // 添加到历史记录
      addToHistory(command)

      // 显示命令在输出中
      addTypedOutput(command, OutputType.COMMAND)
      addOutput('\n')

      // 执行命令
      const result = await invoke<string>('execute_terminal_command', {
        command: command
      })

      console.log('✅ [TERMINAL-STORE] 命令执行结果:', result)

      // 添加输出
      if (result.trim()) {
        addTypedOutput(result, OutputType.OUTPUT)
      }

      // 更新当前工作目录
      await updateCurrentDirectory()

      // 显示新的提示符
      addTypedOutput(`${currentDirectory.value}$ `, OutputType.PROMPT)

      return result
    } catch (error) {
      console.error('❌ [TERMINAL-STORE] 命令执行失败:', error)
      addTypedOutput(`Error: ${error}`, OutputType.ERROR)
      addOutput('\n')
      addTypedOutput(`${currentDirectory.value}$ `, OutputType.PROMPT)
      throw error
    } finally {
      isExecuting.value = false
    }
  }

  // 批量执行命令（支持延时）
  const executeBatchCommands = async (commands: Array<{command: string, delay?: number}>) => {
    try {
      isExecuting.value = true
      
      for (let i = 0; i < commands.length; i++) {
        const { command, delay = 0 } = commands[i]
        
        console.log(`🔧 [TERMINAL-STORE] 执行批量命令 ${i + 1}/${commands.length}: ${command}`)
        
        // 如果有延时且不是第一个命令，先等待
        if (delay > 0 && i > 0) {
          console.log(`⏱️ [TERMINAL-STORE] 等待 ${delay}ms`)
          addTypedOutput(`# 等待 ${delay}ms...`, OutputType.COMMENT)
          addOutput('\n')
          await new Promise(resolve => setTimeout(resolve, delay))
        }
        
        // 执行命令
        await executeCommand(command)
      }
      
      ElMessage.success('批量命令执行完成')
    } catch (error) {
      ElMessage.error(`批量命令执行失败: ${error}`)
      throw error
    } finally {
      isExecuting.value = false
    }
  }

  // 4G入网命令序列
  const execute4GConnection = async () => {
    const commands = [
      { command: 'echo -e "AT+QCFG=\\"usbnet\\",1\\r\\n" > /dev/ttyUSB0' },
      { command: 'echo out > /sys/class/gpio/gpio498/direction' },
      { command: 'echo 1 > /sys/class/gpio/gpio498/value' },
      { command: 'echo 0 > /sys/class/gpio/gpio498/value', delay: 1000 }, // 1秒延时
      { command: 'echo -e "AT+CGDCONT=1,\\"IP\\",\\"CMNET\\"\\r\\n" > /dev/ttyUSB0' },
      { command: 'echo -e "AT+QNETDEVCTL=1,1,1\\r\\n" > /dev/ttyUSB0' },
      { command: 'dhclient -v usb0' }
    ]

    addOutput('\n')
    addTypedOutput('# 开始执行4G入网配置...', OutputType.COMMENT)
    addOutput('\n')
    await executeBatchCommands(commands)
  }

  // 初始化终端状态（只在首次访问时）
  const initializeTerminal = async () => {
    if (isInitialized.value) {
      console.log('🔧 [TERMINAL-STORE] 终端已初始化，跳过重新初始化')
      return
    }

    try {
      const dir = await updateCurrentDirectory()
      terminalOutput.value = ''
      addTypedOutput('欢迎使用EMS终端控制台', OutputType.COMMENT)
      addOutput('\n')
      addTypedOutput(`${dir}$ `, OutputType.PROMPT)
      isInitialized.value = true
      console.log('🔧 [TERMINAL-STORE] 终端初始化完成')
    } catch (error) {
      console.warn('⚠️ [TERMINAL-STORE] 初始化终端失败:', error)
    }
  }

  // 重置终端状态（用于重新连接SSH时）
  const resetTerminal = async () => {
    isInitialized.value = false
    await initializeTerminal()
  }

  // 导航命令历史
  const navigateHistory = (direction: number) => {
    if (commandHistory.value.length === 0) return ''

    historyIndex.value += direction
    
    if (historyIndex.value < 0) {
      historyIndex.value = 0
    } else if (historyIndex.value >= commandHistory.value.length) {
      historyIndex.value = commandHistory.value.length
      return ''
    }

    return commandHistory.value[historyIndex.value] || ''
  }

  return {
    // 状态
    terminalOutput,
    commandHistory,
    historyIndex,
    currentDirectory,
    isExecuting,
    isInitialized,

    // 方法
    addOutput,
    addTypedOutput,
    addToHistory,
    clearTerminal,
    updateCurrentDirectory,
    executeCommand,
    executeBatchCommands,
    execute4GConnection,
    initializeTerminal,
    resetTerminal,
    navigateHistory
  }
}, {
  persist: {
    key: 'terminal-state',
    storage: localStorage,
    paths: ['terminalOutput', 'commandHistory', 'currentDirectory', 'isInitialized']
  }
})
