<template>
  <div class="file-transfer-container">
    <el-row :gutter="24">
      <!-- 文件上传表单 -->
      <el-col :span="16">
        <el-card class="upload-card card-shadow">
          <template #header>
            <div class="card-header">
              <el-icon size="20">
                <Upload />
              </el-icon>
              <span class="card-title">文件传输</span>
            </div>
          </template>

          <el-form
            ref="uploadFormRef"
            :model="uploadForm"
            :rules="uploadRules"
            label-width="100px"
            size="large"
          >
            <el-form-item label="本地文件" prop="localPath">
              <div class="file-input-container">
                <el-input
                  v-model="uploadForm.localPath"
                  placeholder="请选择要上传的文件"
                  readonly
                >
                  <template #prefix>
                    <el-icon><Document /></el-icon>
                  </template>
                </el-input>
                <el-button
                  type="primary"
                  @click="selectFile"
                >
                  <el-icon><FolderOpened /></el-icon>
                  选择文件
                </el-button>
              </div>
            </el-form-item>

            <el-form-item label="远程路径" prop="remotePath">
              <el-input
                v-model="uploadForm.remotePath"
                placeholder="请输入远程服务器路径"
                :disabled="!sshStore.isConnected"
              >
                <template #prefix>
                  <el-icon><Folder /></el-icon>
                </template>
              </el-input>
            </el-form-item>

            <el-form-item>
              <el-button
                type="primary"
                size="large"
                :loading="isUploading"
                :disabled="!sshStore.isConnected || !uploadForm.localPath"
                @click="handleUpload"
                style="width: 100%"
              >
                <el-icon v-if="!isUploading">
                  <Upload />
                </el-icon>
                {{ isUploading ? '上传中...' : '开始上传' }}
              </el-button>
            </el-form-item>
          </el-form>

          <!-- 上传进度 -->
          <div v-if="isUploading || uploadProgress > 0" class="upload-progress">
            <div class="progress-header">
              <span class="progress-label">上传进度</span>
              <span class="progress-percentage">{{ uploadProgress }}%</span>
            </div>
            <el-progress
              :percentage="uploadProgress"
              :status="uploadProgress === 100 ? 'success' : undefined"
              :stroke-width="8"
              striped
              striped-flow
            />
            <div v-if="uploadProgress === 100" class="upload-success">
              <el-icon color="#67c23a" size="16">
                <SuccessFilled />
              </el-icon>
              <span>文件上传成功！</span>
            </div>
          </div>
        </el-card>
      </el-col>

      <!-- 上传状态和提示 -->
      <el-col :span="8">
        <el-card class="status-card card-shadow">
          <template #header>
            <div class="card-header">
              <el-icon size="20">
                <InfoFilled />
              </el-icon>
              <span class="card-title">传输状态</span>
            </div>
          </template>

          <div class="status-content">
            <div class="connection-status">
              <div class="status-item">
                <div class="status-label">SSH连接</div>
                <div class="status-value">
                  <el-tag 
                    :type="sshStore.isConnected ? 'success' : 'danger'"
                    size="small"
                  >
                    {{ sshStore.isConnected ? '已连接' : '未连接' }}
                  </el-tag>
                </div>
              </div>
            </div>

            <el-divider />

            <div class="upload-tips">
              <el-alert
                title="使用提示"
                type="info"
                :closable="false"
                show-icon
              >
                <template #default>
                  <ul class="tips-list">
                    <li>请先建立SSH连接</li>
                    <li>支持上传任意类型文件</li>
                    <li>远程路径会自动创建</li>
                    <li>建议使用绝对路径</li>
                  </ul>
                </template>
              </el-alert>
            </div>

            <div v-if="lastUploadInfo" class="last-upload">
              <el-divider />
              <div class="upload-history">
                <h4>最近上传</h4>
                <div class="history-item">
                  <div class="history-label">文件名</div>
                  <div class="history-value">{{ lastUploadInfo.fileName }}</div>
                </div>
                <div class="history-item">
                  <div class="history-label">远程路径</div>
                  <div class="history-value">{{ lastUploadInfo.remotePath }}</div>
                </div>
                <div class="history-item">
                  <div class="history-label">文件大小</div>
                  <div class="history-value">{{ formatFileSize(lastUploadInfo.fileSize) }}</div>
                </div>
                <div class="history-item">
                  <div class="history-label">上传时间</div>
                  <div class="history-value">{{ lastUploadInfo.uploadTime }}</div>
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
import { ref, reactive } from 'vue'
import type { FormInstance, FormRules } from 'element-plus'
import { ElMessage } from 'element-plus'
import { open } from '@tauri-apps/api/dialog'
import { invoke } from '@tauri-apps/api/tauri'
import { useSshStore } from '@/stores/ssh'

const sshStore = useSshStore()
const uploadFormRef = ref<FormInstance>()
const isUploading = ref(false)
const uploadProgress = ref(0)

// 表单数据
const uploadForm = reactive({
  localPath: '',
  remotePath: '/ems/'
})

// 最近上传信息
const lastUploadInfo = ref<{
  fileName: string
  remotePath: string
  fileSize: number
  uploadTime: string
} | null>(null)

// 表单验证规则
const uploadRules: FormRules = {
  localPath: [
    { required: true, message: '请选择要上传的文件', trigger: 'change' }
  ],
  remotePath: [
    { required: true, message: '请输入远程路径', trigger: 'blur' }
  ]
}

// 选择文件
const selectFile = async () => {
  try {
    console.log('开始选择文件...')

    const selected = await open({
      multiple: false,
      filters: [
        {
          name: '所有文件',
          extensions: ['*']
        },
        {
          name: '图片文件',
          extensions: ['png', 'jpg', 'jpeg', 'gif', 'bmp', 'svg', 'webp', 'ico', 'tiff']
        },
        {
          name: '文档文件',
          extensions: ['md', 'pdf', 'doc', 'docx', 'txt', 'rtf', 'odt', 'pages']
        },
        {
          name: '文本文件',
          extensions: ['txt', 'log', 'conf', 'cfg', 'ini', 'json', 'xml', 'yaml', 'yml', 'csv']
        },
        {
          name: '脚本文件',
          extensions: ['sh', 'py', 'js', 'ts', 'php', 'pl', 'rb', 'go', 'rs', 'java', 'c', 'cpp']
        },
        {
          name: '压缩文件',
          extensions: ['zip', 'tar', 'gz', 'bz2', 'xz', 'rar', '7z', 'dmg', 'iso']
        },
        {
          name: '可执行文件',
          extensions: ['bin', 'exe', 'deb', 'rpm', 'pkg', 'app', 'msi']
        },
        {
          name: '音视频文件',
          extensions: ['mp3', 'mp4', 'avi', 'mov', 'wav', 'flac', 'mkv', 'webm', 'm4a']
        }
      ]
    })

    console.log('文件选择结果:', selected)

    if (selected && typeof selected === 'string') {
      uploadForm.localPath = selected
      ElMessage.success('文件选择成功')
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

// 处理上传
const handleUpload = async () => {
  if (!uploadFormRef.value) return

  const valid = await uploadFormRef.value.validate()
  if (!valid) return

  if (!sshStore.isConnected) {
    ElMessage.warning('请先建立SSH连接')
    return
  }

  let progressInterval: number | null = null

  try {
    console.log('开始文件上传...')
    console.log('本地文件路径:', uploadForm.localPath)
    console.log('远程文件路径:', uploadForm.remotePath)
    console.log('SSH连接状态:', sshStore.isConnected)

    isUploading.value = true
    uploadProgress.value = 0

    // 模拟上传进度
    progressInterval = setInterval(() => {
      if (uploadProgress.value < 90) {
        uploadProgress.value += Math.random() * 10
      }
    }, 200)

    console.log('调用 upload_file 命令...')
    const result = await invoke('upload_file', {
      config: {
        local_path: uploadForm.localPath,
        remote_path: uploadForm.remotePath
      }
    })

    console.log('上传结果:', result)

    // 清除进度条定时器
    if (progressInterval) {
      clearInterval(progressInterval)
      progressInterval = null
    }

    uploadProgress.value = 100

    // 保存上传信息
    const fileName = uploadForm.localPath.split(/[/\\]/).pop() || ''
    lastUploadInfo.value = {
      fileName,
      remotePath: uploadForm.remotePath,
      fileSize: 0, // 实际应该从结果中获取
      uploadTime: new Date().toLocaleString()
    }

    ElMessage.success('文件上传成功')
    console.log('文件上传成功完成')

    // 3秒后重置进度条
    setTimeout(() => {
      uploadProgress.value = 0
    }, 3000)

  } catch (error) {
    console.error('文件上传失败:', error)
    console.error('错误详情:', JSON.stringify(error, null, 2))

    // 确保清除进度条定时器
    if (progressInterval) {
      clearInterval(progressInterval)
      progressInterval = null
    }

    uploadProgress.value = 0
    ElMessage.error(`文件上传失败: ${error}`)
  } finally {
    // 确保清理状态
    if (progressInterval) {
      clearInterval(progressInterval)
    }
    isUploading.value = false
    console.log('上传操作结束，清理状态完成')
  }
}

// 格式化文件大小
const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}
</script>

<style scoped>
.file-transfer-container {
  padding: 20px 0;
}

.upload-card,
.status-card {
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

.file-input-container {
  display: flex;
  gap: 12px;
}

.file-input-container .el-input {
  flex: 1;
}

.upload-progress {
  margin-top: 24px;
  padding: 20px;
  background: #f8f9fa;
  border-radius: 8px;
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.progress-label {
  font-weight: 500;
  color: #606266;
}

.progress-percentage {
  font-weight: 600;
  color: #409eff;
}

.upload-success {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 12px;
  color: #67c23a;
  font-weight: 500;
}

.status-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.status-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.status-label {
  font-weight: 500;
  color: #606266;
}

.tips-list {
  margin: 0;
  padding-left: 20px;
  color: #606266;
}

.tips-list li {
  margin: 4px 0;
}

.upload-history h4 {
  margin: 0 0 12px 0;
  color: #303133;
  font-size: 14px;
}

.history-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
  border-bottom: 1px solid #f0f2f5;
  font-size: 13px;
}

.history-item:last-child {
  border-bottom: none;
}

.history-label {
  color: #909399;
  font-weight: 500;
}

.history-value {
  color: #606266;
  text-align: right;
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
