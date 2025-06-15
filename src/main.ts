import { createApp } from 'vue'
import App from './App.vue'

console.log('🚀 Starting Vue app...')

try {
  const app = createApp(App)
  console.log('📦 App created successfully')

  const mountedApp = app.mount('#app')
  console.log('✅ App mounted successfully!', mountedApp)
} catch (error) {
  console.error('❌ Error mounting app:', error)
}
