import { createApp } from 'vue'
import './styles/global.less'
import './styles/dialog.less'
import App from './App.vue'
import './tauri-polyfill.js'
import router from './router.js'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'

const app = createApp(App)
app.use(router)
app.use(ElementPlus)
app.mount('#app')
