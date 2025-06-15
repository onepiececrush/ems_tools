<template>
  <div class="services-container">
    <!-- 服务管理标签页 -->
    <el-tabs v-model="activeTab" class="services-tabs">
      <!-- 管理服务 -->
      <el-tab-pane label="管理服务" name="manage">
        <el-row :gutter="24">
          <!-- 服务查询 -->
          <el-col :span="12">
            <el-card class="service-query-card card-shadow">
              <template #header>
                <div class="card-header">
                  <el-icon size="20">
                    <Search />
                  </el-icon>
                  <span class="card-title">服务查询</span>
                </div>
              </template>

              <div class="query-form">
                <el-autocomplete
                  v-model="serviceName"
                  :fetch-suggestions="searchServices"
                  placeholder="请输入服务名称（支持模糊搜索）"
                  size="large"
                  :disabled="!sshStore.isConnected"
                  @keyup.enter="queryService"
                  @select="handleServiceSelect"
                  clearable
                  style="width: 100%"
                >
                  <template #prefix>
                    <el-icon><Tools /></el-icon>
                  </template>
                  <template #append>
                    <el-button
                      type="primary"
                      :loading="isQuerying"
                      :disabled="!sshStore.isConnected || !serviceName.trim()"
                      @click="queryService"
                    >
                      查询
                    </el-button>
                  </template>
                </el-autocomplete>
              </div>

              <div v-if="!sshStore.isConnected" class="no-connection">
                <el-alert
                  title="请先建立SSH连接"
                  type="warning"
                  :closable="false"
                  show-icon
                />
              </div>
            </el-card>
          </el-col>

          <!-- 服务状态 -->
          <el-col :span="12">
            <el-card class="service-status-card card-shadow">
              <template #header>
                <div class="card-header">
                  <el-icon size="20">
                    <InfoFilled />
                  </el-icon>
                  <span class="card-title">服务状态</span>
                </div>
              </template>

              <div v-if="!currentService" class="no-service">
                <el-empty description="请先查询服务状态" />
              </div>

              <div v-else class="service-info">
                <div class="service-name">
                  <el-icon size="18">
                    <Tools />
                  </el-icon>
                  <span>{{ currentService.name }}</span>
                </div>

                <div class="status-items">
                  <div class="status-item">
                    <span class="status-label">运行状态</span>
                    <el-tag
                      :type="currentService.active ? 'success' : 'danger'"
                      size="large"
                    >
                      {{ currentService.active ? '运行中' : '已停止' }}
                    </el-tag>
                  </div>

                  <div class="status-item">
                    <span class="status-label">自启动</span>
                    <el-tag
                      :type="currentService.enabled ? 'success' : 'info'"
                      size="large"
                    >
                      {{ currentService.enabled ? '已启用' : '已禁用' }}
                    </el-tag>
                  </div>
                </div>

                <div class="service-actions">
                  <el-button
                    type="success"
                    :loading="isOperating"
                    :disabled="currentService.active"
                    @click="startService"
                  >
                    <el-icon><VideoPlay /></el-icon>
                    启动
                  </el-button>

                  <el-button
                    type="danger"
                    :loading="isOperating"
                    :disabled="!currentService.active"
                    @click="stopService"
                  >
                    <el-icon><VideoPause /></el-icon>
                    停止
                  </el-button>

                  <el-button
                    type="primary"
                    :loading="isOperating"
                    :disabled="currentService.enabled"
                    @click="enableService"
                  >
                    <el-icon><Check /></el-icon>
                    启用自启动
                  </el-button>

                  <el-button
                    type="warning"
                    :loading="isOperating"
                    :disabled="!currentService.enabled"
                    @click="disableService"
                  >
                    <el-icon><Close /></el-icon>
                    禁用自启动
                  </el-button>

                  <el-button
                    type="info"
                    :loading="isDiagnosing"
                    @click="diagnoseService"
                  >
                    <el-icon><Tools /></el-icon>
                    诊断服务
                  </el-button>

                  <el-button
                    type="warning"
                    :loading="isOperating"
                    @click="forceStopService"
                  >
                    <el-icon><CircleClose /></el-icon>
                    强制停止
                  </el-button>

                  <el-button
                    type="primary"
                    :loading="isSearching"
                    @click="findExecutable"
                  >
                    <el-icon><Search /></el-icon>
                    查找文件
                  </el-button>
                </div>
              </div>
            </el-card>
          </el-col>
        </el-row>
      </el-tab-pane>

      <!-- 导入服务 -->
      <el-tab-pane label="导入服务" name="import">
        <el-row :gutter="24">
          <el-col :span="16">
            <el-card class="import-card card-shadow">
              <template #header>
                <div class="card-header">
                  <el-icon size="20">
                    <Upload />
                  </el-icon>
                  <span class="card-title">导入服务文件</span>
                </div>
              </template>

              <el-form
                ref="importFormRef"
                :model="importForm"
                :rules="importRules"
                label-width="120px"
                size="large"
              >
                <el-form-item label="服务名称" prop="name">
                  <el-input
                    v-model="importForm.name"
                    placeholder="请输入服务名称"
                    :disabled="!sshStore.isConnected"
                  >
                    <template #prefix>
                      <el-icon><Tools /></el-icon>
                    </template>
                  </el-input>
                </el-form-item>

                <el-form-item label="服务文件路径" prop="servicePath">
                  <el-input
                    v-model="importForm.servicePath"
                    placeholder="服务文件存放路径"
                    :disabled="!sshStore.isConnected"
                  >
                    <template #prefix>
                      <el-icon><Folder /></el-icon>
                    </template>
                  </el-input>
                </el-form-item>

                <el-form-item label="服务文件内容" prop="content">
                  <div class="content-input-section">
                    <div class="file-select-row">
                      <el-button
                        type="primary"
                        plain
                        :disabled="!sshStore.isConnected"
                        @click="selectServiceFile"
                        style="margin-bottom: 12px;"
                      >
                        <el-icon><FolderOpened /></el-icon>
                        选择本地.service文件
                      </el-button>
                      <span v-if="selectedFileName" class="selected-file-name">
                        已选择: {{ selectedFileName }}
                      </span>
                    </div>
                    <el-input
                      v-model="importForm.content"
                      type="textarea"
                      :rows="12"
                      placeholder="请输入systemd服务文件内容，或点击上方按钮选择本地文件"
                      :disabled="!sshStore.isConnected"
                    />
                  </div>
                </el-form-item>

                <el-form-item>
                  <el-checkbox
                    v-model="importForm.enableAutostart"
                    :disabled="!sshStore.isConnected"
                  >
                    导入后启用自启动
                  </el-checkbox>
                </el-form-item>

                <el-form-item>
                  <el-button
                    type="primary"
                    size="large"
                    :loading="isImporting"
                    :disabled="!sshStore.isConnected"
                    @click="importService"
                    style="width: 100%"
                  >
                    <el-icon v-if="!isImporting">
                      <Upload />
                    </el-icon>
                    {{ isImporting ? '导入中...' : '导入服务' }}
                  </el-button>
                </el-form-item>
              </el-form>
            </el-card>
          </el-col>

          <el-col :span="8">
            <el-card class="template-card card-shadow">
              <template #header>
                <div class="card-header">
                  <el-icon size="20">
                    <Document />
                  </el-icon>
                  <span class="card-title">服务模板</span>
                </div>
              </template>

              <div class="template-content">
                <el-alert
                  title="使用提示"
                  type="info"
                  :closable="false"
                  show-icon
                >
                  <template #default>
                    <ul class="tips-list">
                      <li>服务名称不需要包含.service后缀</li>
                      <li>服务文件会自动保存到指定路径</li>
                      <li>请确保服务文件内容格式正确</li>
                    </ul>
                  </template>
                </el-alert>

                <el-divider content-position="left">示例模板</el-divider>

                <div class="template-example">
                  <pre class="template-code">{{ serviceTemplate }}</pre>
                  <el-button
                    type="text"
                    size="small"
                    @click="useTemplate"
                  >
                    使用此模板
                  </el-button>
                </div>
              </div>
            </el-card>
          </el-col>
        </el-row>
      </el-tab-pane>
    </el-tabs>

    <!-- 服务诊断对话框 -->
    <el-dialog
      v-model="showDiagnosisDialog"
      title="服务诊断结果"
      width="800px"
      :before-close="closeDiagnosisDialog"
    >
      <div class="diagnosis-content">
        <pre class="diagnosis-text">{{ diagnosisResult }}</pre>
      </div>

      <template #footer>
        <el-button @click="closeDiagnosisDialog">关闭</el-button>
        <el-button type="primary" @click="copyDiagnosisResult">复制结果</el-button>
      </template>
    </el-dialog>

    <!-- 文件查找对话框 -->
    <el-dialog
      v-model="showSearchDialog"
      title="可执行文件查找结果"
      width="800px"
      :before-close="closeSearchDialog"
    >
      <div class="search-content">
        <pre class="search-text">{{ searchResult }}</pre>
      </div>

      <template #footer>
        <el-button @click="closeSearchDialog">关闭</el-button>
        <el-button type="primary" @click="copySearchResult">复制结果</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import type { FormInstance, FormRules } from 'element-plus'
import { ElMessage } from 'element-plus'
import { invoke } from '@tauri-apps/api/tauri'
import { open } from '@tauri-apps/api/dialog'
import { readTextFile } from '@tauri-apps/api/fs'
import { useSshStore } from '@/stores/ssh'

interface ServiceStatus {
  name: string
  status: string
  enabled: boolean
  active: boolean
}

interface ServiceInfo {
  name: string
  status: string
  enabled: boolean
  active: boolean
}

const sshStore = useSshStore()
const importFormRef = ref<FormInstance>()

const activeTab = ref('manage')
const serviceName = ref('')
const currentService = ref<ServiceInfo | null>(null)
const isQuerying = ref(false)
const isOperating = ref(false)
const isImporting = ref(false)
const isDiagnosing = ref(false)
const isSearching = ref(false)
const selectedFileName = ref('')
const showDiagnosisDialog = ref(false)
const diagnosisResult = ref('')
const showSearchDialog = ref(false)
const searchResult = ref('')

// 导入表单
const importForm = reactive({
  name: '',
  servicePath: '/etc/systemd/system/',
  content: '',
  enableAutostart: false
})

// 表单验证规则
const importRules: FormRules = {
  name: [
    { required: true, message: '请输入服务名称', trigger: 'blur' }
  ],
  servicePath: [
    { required: true, message: '请输入服务文件路径', trigger: 'blur' }
  ],
  content: [
    { required: true, message: '请输入服务文件内容', trigger: 'blur' }
  ]
}

// 服务模板
const serviceTemplate = `[Unit]
Description=My Service
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/myapp
ExecStart=/opt/myapp/start.sh
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target`

// 搜索服务建议
const searchServices = async (queryString: string, callback: (suggestions: any[]) => void) => {
  if (!sshStore.isConnected) {
    callback([])
    return
  }

  try {
    const services = await invoke<string[]>('search_services', {
      query: queryString
    })

    const suggestions = services.map(service => ({
      value: service,
      label: service
    }))

    callback(suggestions)
  } catch (error) {
    console.error('搜索服务失败:', error)
    callback([])
  }
}

// 处理服务选择
const handleServiceSelect = (item: any) => {
  serviceName.value = item.value
}

// 查询服务
const queryService = async () => {
  if (!serviceName.value.trim()) {
    ElMessage.warning('请输入服务名称')
    return
  }

  try {
    isQuerying.value = true
    const result = await invoke<ServiceStatus>('get_service_status', {
      name: serviceName.value.trim()
    })

    currentService.value = {
      name: serviceName.value.trim(),
      status: result.status,
      enabled: result.enabled,
      active: result.active
    }

    ElMessage.success('查询服务状态成功')
  } catch (error) {
    ElMessage.error(`查询服务失败: ${error}`)
    currentService.value = null
  } finally {
    isQuerying.value = false
  }
}

// 启动服务
const startService = async () => {
  if (!currentService.value) return

  try {
    isOperating.value = true
    await invoke('start_service', { name: currentService.value.name })
    ElMessage.success('服务启动成功')
    await queryService()
  } catch (error) {
    ElMessage.error(`启动服务失败: ${error}`)
  } finally {
    isOperating.value = false
  }
}

// 停止服务
const stopService = async () => {
  if (!currentService.value) return

  try {
    isOperating.value = true
    await invoke('stop_service', { name: currentService.value.name })
    ElMessage.success('服务停止成功')
    await queryService()
  } catch (error) {
    ElMessage.error(`停止服务失败: ${error}`)
  } finally {
    isOperating.value = false
  }
}

// 启用自启动
const enableService = async () => {
  if (!currentService.value) return

  try {
    isOperating.value = true
    await invoke('enable_service', { 
      name: currentService.value.name,
      enable: true
    })
    ElMessage.success('启用自启动成功')
    await queryService()
  } catch (error) {
    ElMessage.error(`启用自启动失败: ${error}`)
  } finally {
    isOperating.value = false
  }
}

// 禁用自启动
const disableService = async () => {
  if (!currentService.value) return

  try {
    isOperating.value = true
    await invoke('enable_service', { 
      name: currentService.value.name,
      enable: false
    })
    ElMessage.success('禁用自启动成功')
    await queryService()
  } catch (error) {
    ElMessage.error(`禁用自启动失败: ${error}`)
  } finally {
    isOperating.value = false
  }
}

// 导入服务
const importService = async () => {
  if (!importFormRef.value) return

  const valid = await importFormRef.value.validate()
  if (!valid) return

  try {
    isImporting.value = true
    
    await invoke('import_service', {
      config: {
        name: importForm.name,
        service_file: `${importForm.servicePath}${importForm.name}.service`,
        enable_autostart: importForm.enableAutostart
      },
      serviceContent: importForm.content
    })

    ElMessage.success('服务导入成功')
    
    // 清空表单
    importForm.name = ''
    importForm.content = ''
    importForm.enableAutostart = false
    selectedFileName.value = ''
  } catch (error) {
    ElMessage.error(`导入服务失败: ${error}`)
  } finally {
    isImporting.value = false
  }
}

// 选择服务文件
const selectServiceFile = async () => {
  try {
    const selected = await open({
      title: '选择systemd服务文件',
      multiple: false,
      filters: [
        {
          name: 'Service Files',
          extensions: ['service']
        },
        {
          name: 'All Files',
          extensions: ['*']
        }
      ]
    })

    console.log('文件选择结果:', selected)

    if (selected && typeof selected === 'string') {
      // 读取文件内容
      const content = await readTextFile(selected)
      importForm.content = content

      // 提取文件名
      const fileName = selected.split(/[/\\]/).pop() || ''
      selectedFileName.value = fileName

      // 如果服务名称为空，尝试从文件名提取
      if (!importForm.name && fileName.endsWith('.service')) {
        importForm.name = fileName.replace('.service', '')
      }

      ElMessage.success('服务文件读取成功')
    } else if (selected === null) {
      console.log('用户取消了文件选择')
    } else {
      console.log('未知的文件选择结果:', selected)
    }
  } catch (error) {
    console.error('文件选择错误:', error)
    ElMessage.error(`选择文件失败: ${error}`)
  }
}

// 使用模板
const useTemplate = () => {
  importForm.content = serviceTemplate
  selectedFileName.value = ''
  ElMessage.success('已应用服务模板')
}

// 诊断服务
const diagnoseService = async () => {
  if (!currentService.value) return

  try {
    isDiagnosing.value = true
    const result = await invoke<string>('diagnose_service', {
      name: currentService.value.name
    })

    diagnosisResult.value = result
    showDiagnosisDialog.value = true
    ElMessage.success('服务诊断完成')
  } catch (error) {
    ElMessage.error(`服务诊断失败: ${error}`)
  } finally {
    isDiagnosing.value = false
  }
}

// 关闭诊断对话框
const closeDiagnosisDialog = () => {
  showDiagnosisDialog.value = false
  diagnosisResult.value = ''
}

// 复制诊断结果
const copyDiagnosisResult = async () => {
  try {
    await navigator.clipboard.writeText(diagnosisResult.value)
    ElMessage.success('诊断结果已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败，请手动复制')
  }
}

// 强制停止服务
const forceStopService = async () => {
  if (!currentService.value) return

  try {
    isOperating.value = true
    const result = await invoke<string>('force_stop_service', {
      name: currentService.value.name
    })

    ElMessage.success(result)
    await queryService() // 刷新状态
  } catch (error) {
    ElMessage.error(`强制停止服务失败: ${error}`)
  } finally {
    isOperating.value = false
  }
}

// 查找可执行文件
const findExecutable = async () => {
  if (!currentService.value) return

  try {
    isSearching.value = true
    const result = await invoke<string>('find_service_executable', {
      serviceName: currentService.value.name
    })

    searchResult.value = result
    showSearchDialog.value = true
    ElMessage.success('文件查找完成')
  } catch (error) {
    ElMessage.error(`查找文件失败: ${error}`)
  } finally {
    isSearching.value = false
  }
}

// 关闭查找对话框
const closeSearchDialog = () => {
  showSearchDialog.value = false
  searchResult.value = ''
}

// 复制查找结果
const copySearchResult = async () => {
  try {
    await navigator.clipboard.writeText(searchResult.value)
    ElMessage.success('查找结果已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败，请手动复制')
  }
}
</script>

<style scoped>
.services-container {
  padding: 20px 0;
}

.services-tabs {
  background: white;
  border-radius: 12px;
  padding: 24px;
  box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.1);
}

.service-query-card,
.service-status-card,
.import-card,
.template-card {
  height: 100%;
  border-radius: 12px;
  border: none;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  color: #303133;
}

.card-title {
  font-size: 16px;
}

.query-form {
  margin-bottom: 20px;
}

.no-connection,
.no-service {
  padding: 40px 20px;
}

.service-info {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.service-name {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 18px;
  font-weight: 600;
  color: #303133;
  padding: 16px;
  background: #f8f9fa;
  border-radius: 8px;
}

.status-items {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.status-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 0;
  border-bottom: 1px solid #f0f2f5;
}

.status-item:last-child {
  border-bottom: none;
}

.status-label {
  font-weight: 500;
  color: #606266;
}

.service-actions {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  margin-top: 20px;
}

.template-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.tips-list {
  margin: 0;
  padding-left: 20px;
  color: #606266;
}

.tips-list li {
  margin: 4px 0;
}

.template-example {
  position: relative;
}

.template-code {
  background: #f8f9fa;
  border: 1px solid #e4e7ed;
  border-radius: 6px;
  padding: 12px;
  font-size: 12px;
  line-height: 1.5;
  color: #606266;
  white-space: pre-wrap;
  word-wrap: break-word;
  margin: 0;
}

.template-example .el-button {
  position: absolute;
  top: 8px;
  right: 8px;
}

.content-input-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.file-select-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.selected-file-name {
  color: #67c23a;
  font-size: 14px;
  font-weight: 500;
}

/* 诊断对话框样式 */
.diagnosis-content {
  max-height: 500px;
  overflow-y: auto;
}

.diagnosis-text {
  background: #f8f9fa;
  border: 1px solid #e4e7ed;
  border-radius: 6px;
  padding: 16px;
  margin: 0;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 13px;
  line-height: 1.6;
  color: #303133;
  white-space: pre-wrap;
  word-break: break-all;
}

/* 文件查找对话框样式 */
.search-content {
  max-height: 500px;
  overflow-y: auto;
}

.search-text {
  background: #f8f9fa;
  border: 1px solid #e4e7ed;
  border-radius: 6px;
  padding: 16px;
  margin: 0;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 13px;
  line-height: 1.6;
  color: #303133;
  white-space: pre-wrap;
  word-break: break-all;
}
</style>
