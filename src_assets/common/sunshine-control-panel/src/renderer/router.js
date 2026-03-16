import { createRouter, createWebHashHistory } from 'vue-router'
import SunshineFrame from './components/SunshineFrame.vue'

const routes = [
  {
    path: '/',
    name: 'Dashboard',
    component: SunshineFrame
  }
]

const router = createRouter({
  history: createWebHashHistory(),
  routes
})

export default router
