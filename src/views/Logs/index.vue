<template>
  <div class="logs-container">
    <el-card class="logs-card card-shadow">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <el-icon size="20">
              <Document />
            </el-icon>
            <span class="card-title">操作日志</span>
            <el-tag :type="logLevelFilter ? 'primary' : 'info'" size="small">
              {{ filteredLogs.length }} 条记录
            </el-tag>
          </div>
          
          <div class="header-actions">
            <el-select
              v-model="logLevelFilter"
              placeholder="筛选级别"
              clearable
              size="small"
              style="width: 120px"
            >
              <el-option label="全部" value="" />
              <el-option label="INFO" value="INFO" />
              <el-option label="WARN" value="WARN" />
              <el-option label="ERROR" value="ERROR" />
              <el-option label="DEBUG" value="DEBUG" />
            </el-select>
            
            <el-button
              type="primary"
              size="small"
              :loading="logsStore.isLoading"
              @click="refreshLogs"
            >
              <el-icon><Refresh /></el-icon>
              刷新
            </el-button>
            
            <el-button
              type="danger"
              size="small"
              @click="clearLogs"
            >
              <el-icon><Delete /></el-icon>
              清空
            </el-button>
          </div>
        </div>
      </template>

      <div class="logs-content">
        <div v-if="filteredLogs.length === 0" class="no-logs">
          <el-empty description="暂无日志记录">
            <el-button type="primary" @click="refreshLogs">
              刷新日志
            </el-button>
          </el-empty>
        </div>

        <div v-else class="logs-table-container">
          <el-table
            :data="paginatedLogs"
            stripe
            :height="tableHeight"
            style="width: 100%"
            :default-sort="{ prop: 'timestamp', order: 'descending' }"
          >
            <el-table-column
              prop="timestamp"
              label="时间"
              width="180"
              sortable
            >
              <template #default="{ row }">
                <div class="timestamp-cell">
                  <el-icon size="14">
                    <Clock />
                  </el-icon>
                  <span>{{ row.timestamp }}</span>
                </div>
              </template>
            </el-table-column>

            <el-table-column
              prop="level"
              label="级别"
              width="100"
              sortable
            >
              <template #default="{ row }">
                <el-tag
                  :type="getLevelTagType(row.level)"
                  size="small"
                  :class="`log-level-${row.level}`"
                >
                  {{ row.level }}
                </el-tag>
              </template>
            </el-table-column>

            <el-table-column
              prop="message"
              label="消息"
              min-width="300"
              show-overflow-tooltip
            >
              <template #default="{ row }">
                <div class="message-cell">
                  <el-icon
                    size="14"
                    :color="getLevelColor(row.level)"
                  >
                    <component :is="getLevelIcon(row.level)" />
                  </el-icon>
                  <span>{{ row.message }}</span>
                </div>
              </template>
            </el-table-column>

            <el-table-column
              label="操作"
              width="80"
              fixed="right"
            >
              <template #default="{ row }">
                <el-button
                  type="text"
                  size="small"
                  @click="showLogDetail(row)"
                >
                  详情
                </el-button>
              </template>
            </el-table-column>
          </el-table>

          <!-- 分页 -->
          <div class="pagination-container">
            <el-pagination
              v-model:current-page="currentPage"
              v-model:page-size="pageSize"
              :page-sizes="[20, 50, 100, 200]"
              :total="filteredLogs.length"
              layout="total, sizes, prev, pager, next, jumper"
              background
            />
          </div>
        </div>
      </div>
    </el-card>

    <!-- 日志详情对话框 -->
    <el-dialog
      v-model="showDetailDialog"
      title="日志详情"
      width="600px"
      :before-close="closeDetailDialog"
    >
      <div v-if="selectedLog" class="log-detail">
        <div class="detail-item">
          <div class="detail-label">时间:</div>
          <div class="detail-value">{{ selectedLog.timestamp }}</div>
        </div>
        <div class="detail-item">
          <div class="detail-label">级别:</div>
          <div class="detail-value">
            <el-tag
              :type="getLevelTagType(selectedLog.level)"
              :class="`log-level-${selectedLog.level}`"
            >
              {{ selectedLog.level }}
            </el-tag>
          </div>
        </div>
        <div class="detail-item">
          <div class="detail-label">消息:</div>
          <div class="detail-value message-content">{{ selectedLog.message }}</div>
        </div>
      </div>
      
      <template #footer>
        <el-button @click="closeDetailDialog">关闭</el-button>
        <el-button type="primary" @click="copyLogContent">复制内容</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useLogsStore, type LogEntry } from '@/stores/logs'

const logsStore = useLogsStore()

const logLevelFilter = ref('')
const currentPage = ref(1)
const pageSize = ref(50)
const tableHeight = ref(500)
const showDetailDialog = ref(false)
const selectedLog = ref<LogEntry | null>(null)

// 筛选后的日志
const filteredLogs = computed(() => {
  if (!logLevelFilter.value) {
    return logsStore.logs
  }
  return logsStore.logs.filter(log => log.level === logLevelFilter.value)
})

// 分页后的日志
const paginatedLogs = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  const end = start + pageSize.value
  return filteredLogs.value.slice(start, end)
})

// 获取级别标签类型
const getLevelTagType = (level: string) => {
  switch (level) {
    case 'INFO':
      return 'primary'
    case 'WARN':
      return 'warning'
    case 'ERROR':
      return 'danger'
    case 'DEBUG':
      return 'info'
    default:
      return 'info'
  }
}

// 获取级别颜色
const getLevelColor = (level: string) => {
  switch (level) {
    case 'INFO':
      return '#409eff'
    case 'WARN':
      return '#e6a23c'
    case 'ERROR':
      return '#f56c6c'
    case 'DEBUG':
      return '#909399'
    default:
      return '#909399'
  }
}

// 获取级别图标
const getLevelIcon = (level: string) => {
  switch (level) {
    case 'INFO':
      return 'InfoFilled'
    case 'WARN':
      return 'WarningFilled'
    case 'ERROR':
      return 'CircleCloseFilled'
    case 'DEBUG':
      return 'Tools'
    default:
      return 'InfoFilled'
  }
}

// 刷新日志
const refreshLogs = async () => {
  await logsStore.fetchLogs()
  ElMessage.success('日志已刷新')
}

// 清空日志
const clearLogs = async () => {
  try {
    await ElMessageBox.confirm(
      '确定要清空所有日志吗？此操作不可恢复。',
      '确认清空',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning',
      }
    )
    
    await logsStore.clearLogs()
    currentPage.value = 1
  } catch {
    // 用户取消操作
  }
}

// 显示日志详情
const showLogDetail = (log: LogEntry) => {
  selectedLog.value = log
  showDetailDialog.value = true
}

// 关闭详情对话框
const closeDetailDialog = () => {
  showDetailDialog.value = false
  selectedLog.value = null
}

// 复制日志内容
const copyLogContent = async () => {
  if (!selectedLog.value) return
  
  const content = `时间: ${selectedLog.value.timestamp}\n级别: ${selectedLog.value.level}\n消息: ${selectedLog.value.message}`
  
  try {
    await navigator.clipboard.writeText(content)
    ElMessage.success('日志内容已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败，请手动复制')
  }
}

// 计算表格高度
const calculateTableHeight = () => {
  const windowHeight = window.innerHeight
  const headerHeight = 200 // 大概的头部高度
  const paginationHeight = 60 // 分页高度
  const padding = 100 // 其他间距
  tableHeight.value = windowHeight - headerHeight - paginationHeight - padding
}

// 监听窗口大小变化
const handleResize = () => {
  calculateTableHeight()
}

onMounted(() => {
  calculateTableHeight()
  window.addEventListener('resize', handleResize)
  
  // 初始加载日志
  logsStore.fetchLogs()
  
  // 定期刷新日志
  const interval = setInterval(() => {
    logsStore.fetchLogs()
  }, 10000)
  
  // 组件卸载时清除定时器
  onUnmounted(() => {
    clearInterval(interval)
  })
})

onUnmounted(() => {
  window.removeEventListener('resize', handleResize)
})
</script>

<style scoped>
.logs-container {
  padding: 20px 0;
}

.logs-card {
  border-radius: 12px;
  border: none;
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

.logs-content {
  margin-top: 16px;
}

.no-logs {
  padding: 60px 20px;
}

.logs-table-container {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.timestamp-cell,
.message-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.pagination-container {
  display: flex;
  justify-content: center;
  padding: 16px 0;
}

.log-detail {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.detail-item {
  display: flex;
  gap: 12px;
}

.detail-label {
  font-weight: 600;
  color: #606266;
  min-width: 60px;
}

.detail-value {
  flex: 1;
  color: #303133;
}

.message-content {
  word-break: break-all;
  white-space: pre-wrap;
  line-height: 1.6;
  background: #f8f9fa;
  padding: 12px;
  border-radius: 6px;
  border: 1px solid #e4e7ed;
}

/* 日志级别样式 */
.log-level-INFO {
  color: #409eff;
}

.log-level-WARN {
  color: #e6a23c;
}

.log-level-ERROR {
  color: #f56c6c;
}

.log-level-DEBUG {
  color: #909399;
}
</style>
