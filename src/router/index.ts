import { createRouter, createWebHashHistory } from 'vue-router'
import Layout from '@/views/Layout.vue'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/',
      component: Layout,
      redirect: '/ssh',
      children: [
        {
          path: '/ssh',
          name: 'SSH',
          component: () => import('@/views/SSH/index.vue'),
          meta: { title: 'SSH连接', icon: 'Connection' }
        },
        {
          path: '/file-transfer',
          name: 'FileTransfer',
          component: () => import('@/views/FileTransfer/index.vue'),
          meta: { title: '文件传输', icon: 'Upload' }
        },
        {
          path: '/network',
          name: 'Network',
          component: () => import('@/views/Network/index.vue'),
          meta: { title: '网络配置', icon: 'Setting' }
        },
        {
          path: '/services',
          name: 'Services',
          component: () => import('@/views/Services/index.vue'),
          meta: { title: '服务管理', icon: 'Tools' }
        },
        {
          path: '/terminal',
          name: 'Terminal',
          component: () => import('@/views/Terminal/index.vue'),
          meta: { title: '终端控制台', icon: 'Monitor' }
        },
        {
          path: '/logs',
          name: 'Logs',
          component: () => import('@/views/Logs/index.vue'),
          meta: { title: '操作日志', icon: 'Document' }
        }
      ]
    }
  ]
})

export default router
