<script setup>
import { 
  Collection, 
  DataBoard, 
  Search, 
  Files, 
  Plus, 
  Setting,
  HomeFilled 
} from '@element-plus/icons-vue'

const props = defineProps({
  activeMenu: {
    type: String,
    default: 'library'
  }
})

const emit = defineEmits(['update:activeMenu', 'open-add', 'open-settings'])

const menus = [
  { id: 'library', icon: Collection, tooltip: '游戏库' },
  { id: 'records', icon: DataBoard, tooltip: '游玩记录' },
  { id: 'scanner', icon: Search, tooltip: '扫描器' },
  { id: 'transformer', icon: Files, tooltip: '转换器' }
]

const selectMenu = (id) => {
  emit('update:activeMenu', id)
}
</script>

<template>
  <aside class="vnite-sidebar">
    <div class="sidebar-top">
      <div class="sidebar-logo" @click="selectMenu('library')">
        <el-icon :size="24" color="var(--el-color-primary)"><HomeFilled /></el-icon>
      </div>
      
      <div class="sidebar-nav">
        <div 
          v-for="menu in menus" 
          :key="menu.id"
          :class="['sidebar-icon-wrap', { active: activeMenu === menu.id }]"
          @click="selectMenu(menu.id)"
        >
          <el-tooltip :content="menu.tooltip" placement="right" :show-after="500">
            <el-icon :size="20"><component :is="menu.icon" /></el-icon>
          </el-tooltip>
        </div>
      </div>
    </div>

    <div class="sidebar-bottom">
      <div class="sidebar-icon-wrap" @click="emit('open-add')">
        <el-tooltip content="添加游戏" placement="right" :show-after="500">
          <el-icon :size="20"><Plus /></el-icon>
        </el-tooltip>
      </div>
      
      <div class="sidebar-icon-wrap" :class="{ active: activeMenu === 'settings' }" @click="emit('open-settings')">
        <el-tooltip content="设置" placement="right" :show-after="500">
          <el-icon :size="20"><Setting /></el-icon>
        </el-tooltip>
      </div>
    </div>
  </aside>
</template>

<style scoped lang="less">
.vnite-sidebar {
  width: 56px;
  height: 100%;
  background: var(--vnite-sidebar-bg);
  backdrop-filter: blur(10px);
  border-right: 1px solid var(--vnite-border);
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  padding: 12px 0;
  flex-shrink: 0;
  z-index: 100;
}

.sidebar-top {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
}

.sidebar-logo {
  cursor: pointer;
  margin-bottom: 10px;
  transition: transform 0.2s;
  
  &:hover {
    transform: scale(1.1);
  }
}

.sidebar-nav {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.sidebar-icon-wrap {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  cursor: pointer;
  color: var(--vnite-text-muted);
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  
  &:hover {
    background: var(--vnite-accent);
    color: var(--vnite-text);
  }
  
  &.active {
    background: rgba(var(--el-color-primary-rgb), 0.15);
    color: var(--el-color-primary);
  }
}

.sidebar-bottom {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

/* 适配 Element Plus 变量 */
:deep(.el-tooltip__trigger) {
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
