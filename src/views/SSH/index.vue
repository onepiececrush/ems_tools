<template>
  <div class="ssh-container">
    <!-- 主连接区域 -->
    <div class="main-connection-area">
      <el-card class="connection-card">
        <div class="connection-header">
          <div class="header-info">
            <el-icon size="24" class="header-icon">
              <Connection />
            </el-icon>
            <div class="header-text">
              <h2 class="connection-title">SSH 远程连接</h2>
              <p class="connection-subtitle">连接到远程服务器进行管理操作</p>
            </div>
          </div>

          <!-- 连接状态指示器 -->
          <div class="connection-status">
            <el-tag
              :type="connectionTagType"
              size="large"
              :effect="sshStore.isConnected ? 'dark' : 'plain'"
              class="status-tag"
            >
              <span
                :class="[
                  'status-indicator',
                  sshStore.isConnected ? 'status-connected' :
                  sshStore.isConnecting ? 'status-connecting' : 'status-disconnected'
                ]"
              ></span>
              {{ sshStore.connectionStatus }}
            </el-tag>
          </div>
        </div>

        <!-- 连接表单 -->
        <div class="connection-form-area">
          <el-form
            ref="sshFormRef"
            :model="sshForm"
            :rules="sshRules"
            label-position="top"
            size="large"
            @submit.prevent="handleConnect"
          >
            <el-row :gutter="16">
              <el-col :span="12">
                <el-form-item label="主机地址" prop="host">
                  <el-input
                    v-model="sshForm.host"
                    placeholder="192.168.1.100"
                    :disabled="sshStore.isConnected"
                    class="form-input"
                  >
                    <template #prefix>
                      <el-icon><Monitor /></el-icon>
                    </template>
                  </el-input>
                </el-form-item>
              </el-col>

              <el-col :span="6">
                <el-form-item label="端口" prop="port">
                  <el-input-number
                    v-model="sshForm.port"
                    :min="1"
                    :max="65535"
                    :disabled="sshStore.isConnected"
                    controls-position="right"
                    class="form-input"
                    style="width: 100%"
                  />
                </el-form-item>
              </el-col>

              <el-col :span="6">
                <el-form-item label="用户名" prop="username">
                  <el-input
                    v-model="sshForm.username"
                    placeholder="root"
                    :disabled="sshStore.isConnected"
                    class="form-input"
                  >
                    <template #prefix>
                      <el-icon><User /></el-icon>
                    </template>
                  </el-input>
                </el-form-item>
              </el-col>
            </el-row>

            <el-row :gutter="16">
              <el-col :span="18">
                <el-form-item label="密码" prop="password">
                  <el-input
                    v-model="sshForm.password"
                    type="password"
                    placeholder="请输入密码"
                    show-password
                    :disabled="sshStore.isConnected"
                    class="form-input"
                    @keyup.enter="handleConnect"
                  >
                    <template #prefix>
                      <el-icon><Lock /></el-icon>
                    </template>
                  </el-input>
                </el-form-item>
              </el-col>

              <el-col :span="6">
                <el-form-item label=" " class="connect-button-item">
                  <el-button
                    v-if="!sshStore.isConnected"
                    type="primary"
                    size="large"
                    :loading="sshStore.isConnecting"
                    @click="handleConnect"
                    class="connect-button"
                  >
                    <el-icon v-if="!sshStore.isConnecting">
                      <Connection />
                    </el-icon>
                    {{ sshStore.isConnecting ? '连接中...' : '连接' }}
                  </el-button>

                  <el-button
                    v-else
                    type="danger"
                    size="large"
                    @click="handleDisconnect"
                    class="connect-button"
                  >
                    <el-icon>
                      <Close />
                    </el-icon>
                    断开连接
                  </el-button>
                </el-form-item>
              </el-col>
            </el-row>
          </el-form>
        </div>

        <!-- 高级选项（可折叠） -->
        <div class="advanced-options">
          <el-collapse v-model="activeAdvanced" accordion>
            <el-collapse-item title="高级选项" name="advanced">
              <div class="advanced-content">
                <el-row :gutter="16">
                  <el-col :span="8">
                    <el-form-item label="连接超时">
                      <el-input-number
                        v-model="advancedOptions.timeout"
                        :min="5"
                        :max="300"
                        controls-position="right"
                        style="width: 100%"
                      />
                      <span class="input-suffix">秒</span>
                    </el-form-item>
                  </el-col>

                  <el-col :span="8">
                    <el-form-item label="保持连接">
                      <el-switch
                        v-model="advancedOptions.keepAlive"
                        active-text="开启"
                        inactive-text="关闭"
                      />
                    </el-form-item>
                  </el-col>

                  <el-col :span="8">
                    <el-form-item label="压缩传输">
                      <el-switch
                        v-model="advancedOptions.compression"
                        active-text="开启"
                        inactive-text="关闭"
                      />
                    </el-form-item>
                  </el-col>
                </el-row>
              </div>
            </el-collapse-item>
          </el-collapse>
        </div>
      </el-card>
    </div>

    <!-- 快速连接区域 -->
    <div class="quick-connect-area">
      <div class="section-header">
        <h3 class="section-title">
          <el-icon><Clock /></el-icon>
          快速连接
        </h3>
        <div class="section-actions">
          <el-button
            v-if="sshStore.connectionHistory.length > 0"
            type="danger"
            size="small"
            text
            @click="handleClearHistory"
          >
            <el-icon><Delete /></el-icon>
            清空历史
          </el-button>
        </div>
      </div>

      <div class="history-container">
        <div v-if="sshStore.connectionHistory.length === 0" class="empty-history">
          <el-empty
            description="暂无连接历史"
            :image-size="120"
          >
            <template #description>
              <div class="empty-description">
                <p>暂无连接历史</p>
                <el-text type="info" size="small">
                  成功连接SSH后，历史记录将显示在这里，方便您快速重新连接
                </el-text>
              </div>
            </template>
          </el-empty>
        </div>

        <div v-else class="history-grid">
          <div
            v-for="item in sshStore.connectionHistory"
            :key="item.id"
            class="history-card"
            @click="handleConnectFromHistory(item)"
          >
            <div class="history-card-header">
              <div class="history-name">{{ item.name }}</div>
              <div class="history-actions" @click.stop>
                <el-button
                  type="danger"
                  size="small"
                  text
                  @click="handleRemoveHistory(item.id)"
                >
                  <el-icon><Close /></el-icon>
                </el-button>
              </div>
            </div>

            <div class="history-details">
              <div class="detail-item">
                <el-icon><Monitor /></el-icon>
                <span>{{ item.host }}:{{ item.port }}</span>
              </div>
              <div class="detail-item">
                <el-icon><User /></el-icon>
                <span>{{ item.username }}</span>
              </div>
            </div>

            <div class="history-footer">
              <el-text size="small" type="info">
                {{ formatTime(item.lastUsed) }}
              </el-text>
              <el-button
                type="primary"
                size="small"
                :loading="sshStore.isConnecting"
                @click.stop="handleConnectFromHistory(item)"
              >
                连接
              </el-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 连接提示 -->
    <div v-if="!sshStore.isConnected" class="connection-tips">
      <el-alert
        title="连接提示"
        type="info"
        :closable="false"
        show-icon
      >
        <template #default>
          <div class="tips-content">
            <div class="tip-item">
              <el-icon><Check /></el-icon>
              <span>确保目标主机已开启SSH服务（通常为22端口）</span>
            </div>
            <div class="tip-item">
              <el-icon><Check /></el-icon>
              <span>检查网络连接和防火墙设置</span>
            </div>
            <div class="tip-item">
              <el-icon><Check /></el-icon>
              <span>确认用户名和密码正确</span>
            </div>
          </div>
        </template>
      </el-alert>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from 'vue'
import type { FormInstance, FormRules } from 'element-plus'
import { useSshStore } from '@/stores/ssh'

const sshStore = useSshStore()
const sshFormRef = ref<FormInstance>()

// 表单数据
const sshForm = reactive({
  host: '192.168.1.100',
  port: 22,
  username: 'root',
  password: ''
})

// 高级选项
const activeAdvanced = ref('')
const advancedOptions = reactive({
  timeout: 30,
  keepAlive: true,
  compression: false
})

// 表单验证规则
const sshRules: FormRules = {
  host: [
    { required: true, message: '请输入主机地址', trigger: 'blur' },
    {
      pattern: /^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/,
      message: '请输入有效的IP地址',
      trigger: 'blur'
    }
  ],
  port: [
    { required: true, message: '请输入端口号', trigger: 'blur' },
    { type: 'number', min: 1, max: 65535, message: '端口号范围为1-65535', trigger: 'blur' }
  ],
  username: [
    { required: true, message: '请输入用户名', trigger: 'blur' }
  ],
  password: [
    { required: true, message: '请输入密码', trigger: 'blur' }
  ]
}

// 连接状态标签类型
const connectionTagType = computed(() => {
  if (sshStore.isConnected) return 'success'
  if (sshStore.isConnecting) return 'warning'
  return 'danger'
})

// 处理连接
const handleConnect = async () => {
  if (!sshFormRef.value) return

  const valid = await sshFormRef.value.validate()
  if (!valid) return

  await sshStore.connect(sshForm)
}

// 处理断开连接
const handleDisconnect = async () => {
  console.log('🔧 [SSH-VIEW] 用户点击断开连接')
  const success = await sshStore.disconnect()

  if (success) {
    console.log('✅ [SSH-VIEW] 断开连接成功')
    // 可以在这里添加额外的清理逻辑
  } else {
    console.log('❌ [SSH-VIEW] 断开连接失败，但前端状态已清理')
  }
}

// 从历史记录连接
const handleConnectFromHistory = async (historyItem: any) => {
  // 填充表单
  sshForm.host = historyItem.host
  sshForm.port = historyItem.port
  sshForm.username = historyItem.username
  sshForm.password = historyItem.password

  // 连接
  await sshStore.connectFromHistory(historyItem)
}

// 删除历史记录
const handleRemoveHistory = (id: string) => {
  sshStore.removeFromHistory(id)
}

// 清空历史记录
const handleClearHistory = () => {
  sshStore.clearHistory()
}

// 格式化时间
const formatTime = (timeString: string) => {
  const date = new Date(timeString)
  const now = new Date()
  const diff = now.getTime() - date.getTime()

  const minutes = Math.floor(diff / (1000 * 60))
  const hours = Math.floor(diff / (1000 * 60 * 60))
  const days = Math.floor(diff / (1000 * 60 * 60 * 24))

  if (minutes < 1) return '刚刚'
  if (minutes < 60) return `${minutes}分钟前`
  if (hours < 24) return `${hours}小时前`
  if (days < 7) return `${days}天前`

  return date.toLocaleDateString()
}


</script>

<style scoped>
.ssh-container {
  width: 100%;
  height: 100%;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 24px;
  overflow-y: auto;
}

/* 主连接区域 */
.main-connection-area {
  flex-shrink: 0;
}

.connection-card {
  border-radius: 16px;
  border: none;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.08);
  overflow: hidden;
}

.connection-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 32px 32px 24px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.header-info {
  display: flex;
  align-items: center;
  gap: 16px;
}

.header-icon {
  color: white;
}

.header-text {
  flex: 1;
}

.connection-title {
  margin: 0 0 4px 0;
  font-size: 24px;
  font-weight: 600;
  color: white;
}

.connection-subtitle {
  margin: 0;
  font-size: 14px;
  color: rgba(255, 255, 255, 0.8);
}

.connection-status {
  flex-shrink: 0;
}

.status-tag {
  font-size: 14px;
  padding: 8px 16px;
  border-radius: 20px;
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
}

/* 表单区域 */
.connection-form-area {
  padding: 32px;
}

.form-input {
  border-radius: 8px;
}

.form-input :deep(.el-input__wrapper) {
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
  border: 1px solid #e4e7ed;
  transition: all 0.3s ease;
}

.form-input :deep(.el-input__wrapper:hover) {
  border-color: #c0c4cc;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.form-input :deep(.el-input__wrapper.is-focus) {
  border-color: #409eff;
  box-shadow: 0 4px 12px rgba(64, 158, 255, 0.2);
}

.connect-button-item {
  margin-bottom: 0 !important;
}

.connect-button {
  width: 100%;
  height: 48px;
  border-radius: 8px;
  font-weight: 600;
  font-size: 16px;
  box-shadow: 0 4px 12px rgba(64, 158, 255, 0.3);
  transition: all 0.3s ease;
}

.connect-button:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(64, 158, 255, 0.4);
}

/* 高级选项 */
.advanced-options {
  border-top: 1px solid #f0f2f5;
  padding: 0 32px 32px;
}

.advanced-content {
  padding-top: 16px;
}

.input-suffix {
  margin-left: 8px;
  color: #909399;
  font-size: 14px;
}

/* 快速连接区域 */
.quick-connect-area {
  flex: 1;
  min-height: 300px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.section-title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: #303133;
  display: flex;
  align-items: center;
  gap: 8px;
}

.section-actions {
  display: flex;
  gap: 8px;
}

.history-container {
  min-height: 200px;
}

.history-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
  max-height: 400px;
  overflow-y: auto;
}

.empty-history {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 250px;
  width: 100%;
  background: #fafbfc;
  border: 2px dashed #e4e7ed;
  border-radius: 12px;
  margin-top: 16px;
}

.empty-description {
  text-align: center;
}

.empty-description p {
  margin: 0 0 8px 0;
  font-size: 16px;
  color: #909399;
  font-weight: 500;
}

.history-card {
  background: white;
  border: 1px solid #e4e7ed;
  border-radius: 12px;
  padding: 20px;
  cursor: pointer;
  transition: all 0.3s ease;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
}

.history-card:hover {
  border-color: #409eff;
  box-shadow: 0 8px 24px rgba(64, 158, 255, 0.15);
  transform: translateY(-2px);
}

.history-card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.history-name {
  font-weight: 600;
  font-size: 16px;
  color: #303133;
}

.history-actions {
  opacity: 0;
  transition: opacity 0.3s ease;
}

.history-card:hover .history-actions {
  opacity: 1;
}

.history-details {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 16px;
}

.detail-item {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #606266;
  font-size: 14px;
}

.detail-item .el-icon {
  color: #909399;
}

.history-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 12px;
  border-top: 1px solid #f0f2f5;
}

/* 连接提示 */
.connection-tips {
  flex-shrink: 0;
}

.tips-content {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.tip-item {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #606266;
}

.tip-item .el-icon {
  color: #67c23a;
  font-size: 16px;
}

/* 状态指示器 */
.status-indicator {
  display: inline-block;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  margin-right: 8px;
}

.status-connected {
  background-color: #67c23a;
  animation: pulse 2s infinite;
}

.status-connecting {
  background-color: #e6a23c;
  animation: blink 1s infinite;
}

.status-disconnected {
  background-color: #f56c6c;
}

@keyframes pulse {
  0% { opacity: 1; }
  50% { opacity: 0.5; }
  100% { opacity: 1; }
}

@keyframes blink {
  0%, 50% { opacity: 1; }
  51%, 100% { opacity: 0; }
}

/* 响应式设计 */
@media (max-width: 1200px) {
  .history-grid {
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  }
}

@media (max-width: 768px) {
  .ssh-container {
    padding: 16px;
    gap: 16px;
  }

  .connection-header {
    padding: 24px 24px 20px;
    flex-direction: column;
    align-items: flex-start;
    gap: 16px;
  }

  .connection-form-area {
    padding: 24px;
  }

  .advanced-options {
    padding: 0 24px 24px;
  }

  .history-grid {
    grid-template-columns: 1fr;
  }

  .connection-title {
    font-size: 20px;
  }
}

@media (max-width: 480px) {
  .ssh-container {
    padding: 12px;
  }

  .connection-header {
    padding: 20px 20px 16px;
  }

  .connection-form-area {
    padding: 20px;
  }

  .advanced-options {
    padding: 0 20px 20px;
  }
}

/* 滚动条样式 */
.history-grid::-webkit-scrollbar {
  width: 6px;
}

.history-grid::-webkit-scrollbar-track {
  background: #f1f1f1;
  border-radius: 3px;
}

.history-grid::-webkit-scrollbar-thumb {
  background: #c1c1c1;
  border-radius: 3px;
}

.history-grid::-webkit-scrollbar-thumb:hover {
  background: #a8a8a8;
}
</style>
