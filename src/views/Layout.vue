<template>
  <div class="layout-container">
    <!-- 顶部导航栏 -->
    <div class="layout-header">
      <div class="header-content">
        <div class="logo-section">
          <el-icon class="logo-icon" size="28">
            <Tools />
          </el-icon>
          <h1 class="app-title">EMS 远程管理工具</h1>
        </div>

        <div class="connection-status">
          <el-tag
            :type="connectionTagType"
            :effect="sshStore.isConnected ? 'dark' : 'plain'"
            size="large"
          >
            <span
              :class="[
                'status-indicator',
                sshStore.isConnected ? 'status-connected' :
                sshStore.isConnecting ? 'status-connecting' : 'status-disconnected'
              ]"
            ></span>
            {{ sshStore.connectionInfo }}
          </el-tag>
        </div>
      </div>
    </div>

    <div class="layout-body">
      <!-- 侧边栏 -->
      <div class="layout-aside">
        <el-menu
          :default-active="$route.path"
          class="sidebar-menu"
          router
          :collapse="false"
        >
          <el-menu-item
            v-for="route in menuRoutes"
            :key="route.path"
            :index="route.path"
            class="menu-item"
          >
            <el-icon>
              <component :is="route.meta.icon" />
            </el-icon>
            <template #title>{{ route.meta.title }}</template>
          </el-menu-item>
        </el-menu>
      </div>

      <!-- 主内容区域 -->
      <div class="layout-main">
        <div class="main-content">
          <router-view />
        </div>
      </div>
    </div>

    <!-- 底部状态栏 -->
    <div class="layout-footer">
      <div class="footer-content">
        <div class="status-info">
          <el-text size="small" type="info">
            {{ sshStore.connectionStatus }}
          </el-text>
        </div>
        <div class="app-info">
          <el-text size="small" type="info">
            EMS Tools v0.1.0
          </el-text>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useSshStore } from '@/stores/ssh'

const router = useRouter()
const sshStore = useSshStore()

// 获取菜单路由
const menuRoutes = computed(() => {
  return router.getRoutes()
    .filter(route => route.meta?.title)
    .sort((a, b) => {
      const order = ['SSH连接', '文件传输', '网络配置', '服务管理', '终端控制台', '操作日志']
      return order.indexOf(a.meta.title as string) - order.indexOf(b.meta.title as string)
    })
})

// 连接状态标签类型
const connectionTagType = computed(() => {
  if (sshStore.isConnected) return 'success'
  if (sshStore.isConnecting) return 'warning'
  return 'danger'
})
</script>

<style scoped>
.layout-container {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  width: 100vw;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #f5f7fa;
  overflow: hidden;
}

.layout-header {
  height: 64px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  padding: 0 32px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  z-index: 1000;
  flex-shrink: 0;
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  height: 100%;
}

.logo-section {
  display: flex;
  align-items: center;
  gap: 12px;
}

.logo-icon {
  color: white;
}

.app-title {
  font-size: 20px;
  font-weight: 600;
  margin: 0;
}

.connection-status {
  display: flex;
  align-items: center;
}

.layout-body {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.layout-aside {
  width: 180px;
  background: white;
  box-shadow: 2px 0 8px rgba(0, 0, 0, 0.1);
  z-index: 999;
  flex-shrink: 0;
  overflow-y: auto;
}

.sidebar-menu {
  border-right: none;
  height: 100%;
}

.menu-item {
  margin: 4px 8px;
  border-radius: 8px;
  transition: all 0.3s ease;
}

.menu-item:hover {
  background-color: #f0f2f5;
}

.menu-item.is-active {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.layout-main {
  flex: 1;
  padding: 32px;
  background: #f5f7fa;
  overflow-y: auto;
}

.main-content {
  width: 100%;
  height: 100%;
}

.layout-footer {
  height: 48px;
  background: white;
  border-top: 1px solid #e4e7ed;
  padding: 0 32px;
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.footer-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

/* 状态指示器动画 */
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
</style>
