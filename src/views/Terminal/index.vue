<template>
  <div class="terminal-container">
    <el-card class="terminal-card card-shadow">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <el-icon size="20">
              <Monitor />
            </el-icon>
            <span class="card-title">终端控制台</span>
            <el-tag v-if="sshStore.isConnected" type="success" size="small">
              已连接
            </el-tag>
          </div>

          <div class="header-actions">
            <el-button
              type="success"
              size="small"
              @click="handle4GConnection"
              :loading="terminalStore.isExecuting"
              :disabled="!sshStore.isConnected"
            >
              <el-icon><Connection /></el-icon>
              4G入网
            </el-button>

            <el-button
              type="info"
              size="small"
              @click="terminalStore.clearTerminal"
            >
              <el-icon><Delete /></el-icon>
              清屏
            </el-button>
          </div>
        </div>
      </template>

      <div class="terminal-content">
        <!-- 未连接SSH时的提示 -->
        <div v-if="!sshStore.isConnected" class="no-connection">
          <el-empty description="请先连接SSH服务器">
            <el-button type="primary" @click="$router.push('/ssh')">
              前往连接
            </el-button>
          </el-empty>
        </div>

        <!-- 终端界面 -->
        <div v-else class="terminal-interface">
          <div class="terminal-pane">
            <!-- 终端输出区域 -->
            <div class="terminal-output" ref="terminalOutputRef">
              <div class="output-content">
                <pre class="terminal-text" v-html="terminalStore.terminalOutput"></pre>
              </div>
            </div>

            <!-- 命令输入区域 -->
            <div class="terminal-input">
              <div class="input-prompt">
                <span class="prompt-symbol">$</span>
                <el-input
                  v-model="currentCommand"
                  placeholder="输入命令... (支持: ls, ll, la, cd, pwd, cat, etc.)"
                  @keyup.enter="executeCommand"
                  @keyup.up="navigateHistory(-1)"
                  @keyup.down="navigateHistory(1)"
                  ref="commandInputRef"
                  class="command-input"
                >
                  <template #append>
                    <el-button
                      type="primary"
                      @click="executeCommand"
                      :loading="terminalStore.isExecuting"
                      :disabled="!currentCommand.trim()"
                    >
                      执行
                    </el-button>
                  </template>
                </el-input>
              </div>

              <!-- 命令提示 -->
              <div v-if="showCommandHint" class="command-hint">
                <el-text size="small" type="info">
                  💡 提示：{{ commandHint }}
                </el-text>
              </div>
            </div>
          </div>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, onMounted, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { useSshStore } from '@/stores/ssh'
import { useTerminalStore } from '@/stores/terminal'

const sshStore = useSshStore()
const terminalStore = useTerminalStore()

// 本地状态（不需要持久化的）
const currentCommand = ref('')
const showCommandHint = ref(false)
const commandHint = ref('')

// DOM引用
const terminalOutputRef = ref<HTMLElement>()
const commandInputRef = ref()

// 命令提示映射
const commandHints: Record<string, string> = {
  'll': "'ll' 会自动转换为 'ls -l' 来显示详细文件列表",
  'la': "'la' 会自动转换为 'ls -la' 来显示所有文件（包括隐藏文件）",
  'l': "'l' 会自动转换为 'ls -CF' 来显示文件类型标识",
  'ls': "列出目录内容，可使用 -l (详细), -a (全部), -h (人类可读大小)",
  'cd': "切换目录，使用 'cd ..' 返回上级目录，'cd ~' 返回主目录",
  'pwd': "显示当前工作目录的完整路径",
  'cat': "显示文件内容，如 'cat filename.txt'",
  'mkdir': "创建目录，如 'mkdir dirname'",
  'rm': "删除文件，使用 'rm -rf' 删除目录（谨慎使用）",
  'cp': "复制文件或目录，如 'cp source dest'",
  'mv': "移动或重命名文件，如 'mv oldname newname'",
  'find': "查找文件，如 'find . -name \"*.txt\"'",
  'grep': "搜索文本，如 'grep \"pattern\" filename'",
  'ps': "显示运行中的进程，使用 'ps aux' 显示所有进程",
  'top': "实时显示系统进程信息",
  'df': "显示磁盘空间使用情况，使用 'df -h' 显示人类可读格式",
  'free': "显示内存使用情况，使用 'free -h' 显示人类可读格式",
  'chmod': "修改文件权限，如 'chmod 755 filename'",
  'chown': "修改文件所有者，如 'chown user:group filename'"
}

// 执行命令
const executeCommand = async () => {
  if (!currentCommand.value.trim()) return

  try {
    await terminalStore.executeCommand(currentCommand.value)
    currentCommand.value = ''

    // 滚动到底部
    await nextTick()
    scrollToBottom()
  } catch (error) {
    ElMessage.error(`命令执行失败: ${error}`)
  }
}

// 4G入网处理
const handle4GConnection = async () => {
  try {
    await terminalStore.execute4GConnection()

    // 滚动到底部
    await nextTick()
    scrollToBottom()
  } catch (error) {
    ElMessage.error(`4G入网配置失败: ${error}`)
  }
}

// 导航命令历史
const navigateHistory = (direction: number) => {
  const command = terminalStore.navigateHistory(direction)
  currentCommand.value = command
}

// 滚动到底部
const scrollToBottom = () => {
  if (terminalOutputRef.value) {
    terminalOutputRef.value.scrollTop = terminalOutputRef.value.scrollHeight
  }
}

// 聚焦命令输入框
const focusCommandInput = () => {
  if (commandInputRef.value) {
    commandInputRef.value.focus()
  }
}

// 监听终端输出变化，自动滚动到底部
watch(() => terminalStore.terminalOutput, () => {
  nextTick(() => {
    scrollToBottom()
  })
})

// 监听命令输入变化，显示提示
watch(currentCommand, (newCommand) => {
  const trimmed = newCommand.trim().toLowerCase()
  const baseCommand = trimmed.split(' ')[0]

  if (baseCommand && commandHints[baseCommand]) {
    commandHint.value = commandHints[baseCommand]
    showCommandHint.value = true
  } else {
    showCommandHint.value = false
  }
})

// 监听SSH连接状态变化
watch(() => sshStore.isConnected, async (newConnected, oldConnected) => {
  if (newConnected && !oldConnected) {
    // 新连接建立时，重置终端状态
    console.log('🔧 [TERMINAL-UI] SSH连接建立，重置终端状态')
    await terminalStore.resetTerminal()
  }
})

onMounted(async () => {
  // 初始化终端状态
  if (sshStore.isConnected) {
    await terminalStore.initializeTerminal()
  }

  // 聚焦到命令输入框
  nextTick(() => {
    focusCommandInput()
  })
})
</script>

<style scoped>
.terminal-container {
  padding: 20px 0;
  height: 100%;
}

.terminal-card {
  border-radius: 12px;
  border: none;
  height: calc(100vh - 200px);
  display: flex;
  flex-direction: column;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.card-title {
  font-size: 16px;
  font-weight: 600;
  color: #303133;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.terminal-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.no-connection {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.terminal-interface {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.terminal-pane {
  height: calc(100vh - 280px);
  display: flex;
  flex-direction: column;
  background: #1e1e1e;
  border-radius: 8px;
  overflow: hidden;
}

.terminal-output {
  flex: 1;
  overflow-y: auto;
  background: #1e1e1e;
  padding: 16px;
}

.output-content {
  height: 100%;
}

.terminal-text {
  color: #00ff00;          /* 默认绿色 */
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 14px;
  line-height: 1.4;
  margin: 0;
  white-space: pre-wrap;
  word-break: break-all;
  background: transparent;
}

/* 终端输出类型样式 - 经典终端配色方案 */
.terminal-text :deep(.terminal-command) {
  color: #ffffff !important;          /* 用户命令：亮白色 */
  font-weight: bold;
}

.terminal-text :deep(.terminal-output) {
  color: #00ff00 !important;          /* 系统输出：浅绿色 */
}

.terminal-text :deep(.terminal-error) {
  color: #ff6b6b !important;          /* 错误信息：红色 */
  font-weight: bold;
}

.terminal-text :deep(.terminal-prompt) {
  color: #ffd93d !important;          /* 提示符：黄色 */
  font-weight: bold;
}

.terminal-text :deep(.terminal-comment) {
  color: #888888 !important;          /* 注释/系统消息：灰色 */
  font-style: italic;
}

.terminal-input {
  background: #2d2d2d;
  border-top: 1px solid #404040;
  padding: 12px 16px;
}

.input-prompt {
  display: flex;
  align-items: center;
  gap: 8px;
}

.prompt-symbol {
  color: #ffd93d;          /* 提示符：黄色 */
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 14px;
  font-weight: bold;
  min-width: 20px;
}

.command-input {
  flex: 1;
}

.command-input :deep(.el-input__wrapper) {
  background: #1e1e1e;
  border: 1px solid #404040;
  color: #ffffff;          /* 用户输入：亮白色 */
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
}

.command-input :deep(.el-input__inner) {
  color: #ffffff;          /* 用户输入：亮白色 */
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
}

.command-input :deep(.el-input__wrapper:hover) {
  border-color: #606060;
}

.command-input :deep(.el-input__wrapper.is-focus) {
  border-color: #409eff;
  box-shadow: 0 0 0 2px rgba(64, 158, 255, 0.2);
}

.command-hint {
  margin-top: 8px;
  padding: 8px 12px;
  background: rgba(64, 158, 255, 0.1);
  border: 1px solid rgba(64, 158, 255, 0.2);
  border-radius: 6px;
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(-5px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 终端滚动条样式 */
.terminal-output::-webkit-scrollbar {
  width: 8px;
}

.terminal-output::-webkit-scrollbar-track {
  background: #2d2d2d;
}

.terminal-output::-webkit-scrollbar-thumb {
  background: #404040;
  border-radius: 4px;
}

.terminal-output::-webkit-scrollbar-thumb:hover {
  background: #606060;
}


</style>
