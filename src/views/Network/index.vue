<template>
  <div class="network-container">
    <el-row :gutter="24">
      <!-- 网络接口列表 -->
      <el-col :span="12">
        <el-card class="interfaces-card card-shadow">
          <template #header>
            <div class="card-header">
              <el-icon size="20">
                <Setting />
              </el-icon>
              <span class="card-title">网络接口</span>
              <el-button
                type="primary"
                size="small"
                :loading="isRefreshing"
                :disabled="!sshStore.isConnected"
                @click="refreshInterfaces"
              >
                <el-icon><Refresh /></el-icon>
                刷新
              </el-button>
            </div>
          </template>

          <div v-if="!sshStore.isConnected" class="no-connection">
            <el-empty description="请先建立SSH连接">
              <el-button type="primary" @click="$router.push('/ssh')">
                去连接
              </el-button>
            </el-empty>
          </div>

          <div v-else-if="interfaces.length === 0" class="no-interfaces">
            <el-empty description="暂无网络接口数据">
              <el-button type="primary" @click="refreshInterfaces">
                刷新接口
              </el-button>
            </el-empty>
          </div>

          <div v-else class="interfaces-list">
            <div
              v-for="iface in interfaces"
              :key="iface.name"
              class="interface-item"
              :class="{ active: selectedInterface?.name === iface.name }"
              @click="selectInterface(iface)"
            >
              <div class="interface-info">
                <div class="interface-name">
                  <el-icon size="16">
                    <Connection />
                  </el-icon>
                  {{ iface.name }}
                </div>
                <div class="interface-status">
                  <el-tag
                    :type="iface.status === 'UP' ? 'success' : 'info'"
                    size="small"
                  >
                    {{ iface.status }}
                  </el-tag>
                </div>
              </div>
              <div class="interface-details">
                <div v-if="iface.ip_address" class="detail-item">
                  <span class="detail-label">IP:</span>
                  <span class="detail-value">{{ iface.ip_address }}</span>
                </div>
                <div v-if="iface.mac_address" class="detail-item">
                  <span class="detail-label">MAC:</span>
                  <span class="detail-value">{{ iface.mac_address }}</span>
                </div>
              </div>
            </div>
          </div>
        </el-card>
      </el-col>

      <!-- 接口配置 -->
      <el-col :span="12">
        <el-card class="config-card card-shadow">
          <template #header>
            <div class="card-header">
              <el-icon size="20">
                <Tools />
              </el-icon>
              <span class="card-title">
                {{ selectedInterface ? `配置 ${selectedInterface.name}` : '接口配置' }}
              </span>
            </div>
          </template>

          <div v-if="!selectedInterface" class="no-selection">
            <el-empty description="请选择要配置的网络接口" />
          </div>

          <div v-else class="config-form">
            <el-form
              ref="configFormRef"
              :model="configForm"
              :rules="configRules"
              label-width="100px"
              size="large"
            >
              <el-form-item label="接口名称">
                <el-input
                  :value="selectedInterface.name"
                  disabled
                >
                  <template #prefix>
                    <el-icon><Connection /></el-icon>
                  </template>
                </el-input>
              </el-form-item>

              <el-form-item label="IP地址" prop="ip_address">
                <el-input
                  v-model="configForm.ip_address"
                  placeholder="请输入IP地址"
                >
                  <template #prefix>
                    <el-icon><Monitor /></el-icon>
                  </template>
                </el-input>
              </el-form-item>

              <el-form-item label="子网掩码" prop="netmask">
                <el-input
                  v-model="configForm.netmask"
                  placeholder="请输入子网掩码"
                >
                  <template #prefix>
                    <el-icon><Setting /></el-icon>
                  </template>
                </el-input>
              </el-form-item>

              <el-form-item label="默认网关" prop="gateway">
                <el-input
                  v-model="configForm.gateway"
                  placeholder="请输入默认网关（可选）"
                >
                  <template #prefix>
                    <el-icon><Share /></el-icon>
                  </template>
                </el-input>
              </el-form-item>

              <el-form-item>
                <el-button
                  type="primary"
                  size="large"
                  :loading="isSaving"
                  @click="saveConfig"
                  style="width: 100%"
                >
                  <el-icon v-if="!isSaving">
                    <Check />
                  </el-icon>
                  {{ isSaving ? '保存中...' : '保存配置' }}
                </el-button>
              </el-form-item>
            </el-form>

            <!-- 当前配置信息 -->
            <div class="current-config">
              <el-divider content-position="left">当前配置</el-divider>
              <div class="config-info">
                <div class="config-item">
                  <span class="config-label">状态:</span>
                  <el-tag
                    :type="selectedInterface.status === 'UP' ? 'success' : 'info'"
                    size="small"
                  >
                    {{ selectedInterface.status }}
                  </el-tag>
                </div>
                <div v-if="selectedInterface.ip_address" class="config-item">
                  <span class="config-label">当前IP:</span>
                  <span class="config-value">{{ selectedInterface.ip_address }}</span>
                </div>
                <div v-if="selectedInterface.netmask" class="config-item">
                  <span class="config-label">当前掩码:</span>
                  <span class="config-value">{{ selectedInterface.netmask }}</span>
                </div>
                <div v-if="selectedInterface.mac_address" class="config-item">
                  <span class="config-label">MAC地址:</span>
                  <span class="config-value">{{ selectedInterface.mac_address }}</span>
                </div>
              </div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import type { FormInstance, FormRules } from 'element-plus'
import { ElMessage } from 'element-plus'
import { invoke } from '@tauri-apps/api/tauri'
import { useSshStore } from '@/stores/ssh'

interface NetworkInterface {
  name: string
  status: string
  mac_address?: string
  ip_address?: string
  netmask?: string
}

interface NetworkConfig {
  interface: string
  ip_address: string
  netmask: string
  gateway?: string
}

const sshStore = useSshStore()
const configFormRef = ref<FormInstance>()

const interfaces = ref<NetworkInterface[]>([])
const selectedInterface = ref<NetworkInterface | null>(null)
const isRefreshing = ref(false)
const isSaving = ref(false)

// 配置表单
const configForm = reactive({
  ip_address: '',
  netmask: '',
  gateway: ''
})

// 表单验证规则
const configRules: FormRules = {
  ip_address: [
    { required: true, message: '请输入IP地址', trigger: 'blur' },
    { 
      pattern: /^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/,
      message: '请输入有效的IP地址',
      trigger: 'blur'
    }
  ],
  netmask: [
    { required: true, message: '请输入子网掩码', trigger: 'blur' },
    { 
      pattern: /^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/,
      message: '请输入有效的子网掩码',
      trigger: 'blur'
    }
  ],
  gateway: [
    { 
      pattern: /^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/,
      message: '请输入有效的网关地址',
      trigger: 'blur'
    }
  ]
}

// 刷新网络接口
const refreshInterfaces = async () => {
  if (!sshStore.isConnected) {
    ElMessage.warning('请先建立SSH连接')
    return
  }

  try {
    isRefreshing.value = true
    const result = await invoke<NetworkInterface[]>('get_network_interfaces')
    interfaces.value = result
    
    // 如果当前选中的接口不在新列表中，清除选择
    if (selectedInterface.value && !result.find(i => i.name === selectedInterface.value!.name)) {
      selectedInterface.value = null
    }
  } catch (error) {
    ElMessage.error(`获取网络接口失败: ${error}`)
  } finally {
    isRefreshing.value = false
  }
}

// 选择接口
const selectInterface = async (iface: NetworkInterface) => {
  selectedInterface.value = iface
  
  try {
    const config = await invoke<NetworkConfig>('get_interface_config', {
      interface: iface.name
    })
    
    configForm.ip_address = config.ip_address
    configForm.netmask = config.netmask
    configForm.gateway = config.gateway || ''
  } catch (error) {
    ElMessage.error(`获取接口配置失败: ${error}`)
    // 使用默认值
    configForm.ip_address = iface.ip_address || ''
    configForm.netmask = iface.netmask || '255.255.255.0'
    configForm.gateway = ''
  }
}

// 保存配置
const saveConfig = async () => {
  if (!configFormRef.value || !selectedInterface.value) return
  
  const valid = await configFormRef.value.validate()
  if (!valid) return

  try {
    isSaving.value = true
    
    const config: NetworkConfig = {
      interface: selectedInterface.value.name,
      ip_address: configForm.ip_address,
      netmask: configForm.netmask,
      gateway: configForm.gateway || undefined
    }
    
    await invoke('set_interface_config', { config })
    
    ElMessage.success('网络配置保存成功')
    
    // 刷新接口列表
    await refreshInterfaces()
  } catch (error) {
    ElMessage.error(`保存配置失败: ${error}`)
  } finally {
    isSaving.value = false
  }
}

// 组件挂载时刷新接口列表
onMounted(() => {
  if (sshStore.isConnected) {
    refreshInterfaces()
  }
})
</script>

<style scoped>
.network-container {
  padding: 20px 0;
}

.interfaces-card,
.config-card {
  height: 100%;
  border-radius: 12px;
  border: none;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-weight: 600;
  color: #303133;
}

.card-header .el-button {
  margin-left: auto;
}

.card-title {
  font-size: 16px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.interfaces-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.interface-item {
  padding: 16px;
  border: 1px solid #e4e7ed;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.3s ease;
}

.interface-item:hover {
  border-color: #409eff;
  box-shadow: 0 2px 8px rgba(64, 158, 255, 0.1);
}

.interface-item.active {
  border-color: #409eff;
  background-color: #f0f8ff;
}

.interface-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.interface-name {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  color: #303133;
}

.interface-details {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.detail-item {
  display: flex;
  gap: 8px;
  font-size: 13px;
}

.detail-label {
  color: #909399;
  font-weight: 500;
  min-width: 40px;
}

.detail-value {
  color: #606266;
}

.no-connection,
.no-interfaces,
.no-selection {
  padding: 40px 20px;
}

.config-form {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.current-config {
  margin-top: 24px;
}

.config-info {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  background: #f8f9fa;
  border-radius: 8px;
}

.config-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.config-label {
  font-weight: 500;
  color: #606266;
}

.config-value {
  color: #303133;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
}
</style>
