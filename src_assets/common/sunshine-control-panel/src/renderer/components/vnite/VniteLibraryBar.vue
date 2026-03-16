```
<script setup>
import { ref, computed } from 'vue'
import { 
  Search, Folder, CollectionTag, User, Timer, Plus, Delete,
  VideoPlay, Edit, MagicStick 
} from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { invoke } from '@tauri-apps/api/core'
import VniteContextMenu from './VniteContextMenu.vue'

const props = defineProps({
  games: {
    type: Array,
    default: () => []
  },
  collections: {
    type: Array,
    default: () => []
  },
  selectedGame: Object
})

const emit = defineEmits(['select-game', 'update-collections', 'launch-game', 'edit-game', 'open-scraper', 'delete-game'])

const keyword = ref('')
const groupBy = ref('all') // 'all', 'developer', 'status', 'collection'

const filteredGames = computed(() => {
  if (!keyword.value) return props.games
  const k = keyword.value.toLowerCase()
  return props.games.filter(g => 
    g.name.toLowerCase().includes(k) || 
    (g.developer && g.developer.toLowerCase().includes(k)) ||
    (g.original_name && g.original_name.toLowerCase().includes(k))
  )
})

const groupedData = computed(() => {
  if (groupBy.value === 'all') {
    return [{ name: '所有游戏', items: filteredGames.value }]
  }
  
  const groups = {}
  filteredGames.value.forEach(game => {
    let key = '未分类'
    if (groupBy.value === 'developer') {
      key = game.developer || '未知开发商'
    } else if (groupBy.value === 'status') {
      key = getStatusLabel(game.status)
    }
    
    if (!groups[key]) groups[key] = []
    groups[key].push(game)
  })
  
  return Object.keys(groups).sort().map(name => ({
    name,
    items: groups[name]
  }))
})

const collectionGroups = computed(() => {
  if (groupBy.value !== 'collection') return []
  
  return props.collections.map(col => {
    const colGames = props.games.filter(g => col.games.includes(g.name))
    // Apply search filter if keyword exists
    const filteredColGames = !keyword.value ? colGames : colGames.filter(g => {
      const k = keyword.value.toLowerCase()
      return g.name.toLowerCase().includes(k) || 
             (g.developer && g.developer.toLowerCase().includes(k))
    })

    return {
      id: col.id,
      name: col.name,
      items: filteredColGames
    }
  })
})

const getStatusLabel = (status) => {
  const map = {
    'NotStarted': '未开始',
    'Playing': '游玩中',
    'Finished': '已通关',
    'Partial': '部分通关',
    'Multiple': '多周目',
    'Shelved': '搁置'
  }
  return map[status] || '未开始'
}

const getStatusColor = (status) => {
  const map = {
    'NotStarted': '#909399',
    'Playing': '#409EFF',
    'Finished': '#67C23A',
    'Partial': '#E6A23C',
    'Multiple': '#F56C6C',
    'Shelved': '#909399'
  }
  return map[status] || '#909399'
}

const handleDragStart = (e, game) => {
  e.dataTransfer.setData('gameName', game.name)
  e.dataTransfer.effectAllowed = 'move'
}

const handleDrop = async (e, collectionId) => {
  const gameName = e.dataTransfer.getData('gameName')
  if (gameName) {
    handleAddToCollection(collectionId, gameName)
  }
}

// Context Menu State
const menuVisible = ref(false)
const menuX = ref(0)
const menuY = ref(0)
const contextGame = ref(null)

const menuItems = computed(() => {
  if (!contextGame.value) return []
  return [
    { label: '启动游戏', icon: VideoPlay, command: 'launch' },
    { divider: true },
    { label: '编辑信息', icon: Edit, command: 'edit' },
    { label: '搜刮元数据', icon: MagicStick, command: 'scraper' },
    { divider: true },
    { label: '从库中删除', icon: Delete, command: 'delete', danger: true },
  ]
})

const onContextMenu = (e, game) => {
  contextGame.value = game
  menuX.value = e.clientX
  menuY.value = e.clientY
  menuVisible.value = true
}

const handleMenuCommand = (command) => {
  const game = contextGame.value
  if (!game) return

  switch (command) {
    case 'launch':
      emit('launch-game', game)
      break
    case 'edit':
      emit('edit-game', game)
      break
    case 'scraper':
      emit('open-scraper', game)
      break
    case 'delete':
      emit('delete-game', game)
      break
  }
}

const showCollectionSelector = (game) => {
  if (props.collections.length === 0) {
    ElMessage.warning('暂无收藏夹，请先创建一个')
    return
  }
  
  ElMessageBox.prompt('请选择收藏夹 (输入完整名称)', '添加到收藏夹', {
    confirmButtonText: '添加',
    cancelButtonText: '取消',
    inputPlaceholder: props.collections.map(c => c.name).join(', ')
  }).then(({ value }) => {
    const col = props.collections.find(c => c.name === value)
    if (col) {
      handleAddToCollection(col.id, game.name)
    } else {
      ElMessage.error('找不到该收藏夹')
    }
  })
}

const handleAddToCollection = async (collectionId, gameName) => {
  try {
    await invoke('galgame_add_to_collection', { collectionId, gameName })
    ElMessage.success('已添加')
    emit('update-collections')
  } catch (e) {
    ElMessage.error('失败: ' + e)
  }
}

const handleRemoveFromCollection = async (collectionId, gameName) => {
  try {
    await invoke('galgame_remove_from_collection', { collectionId, gameName })
    ElMessage.success('已移除')
    emit('update-collections')
  } catch (e) {
    ElMessage.error('失败: ' + e)
  }
}

const handleAddCollection = async () => {
  try {
    const { value: name } = await ElMessageBox.prompt('请输入收藏夹名称', '新建收藏夹', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      inputPattern: /\S+/,
      inputErrorMessage: '名称不能为空'
    })
    
    if (name) {
      await invoke('galgame_add_collection', { name })
      emit('update-collections')
      ElMessage.success('已创建收藏夹')
    }
  } catch (e) {
    if (e !== 'cancel') console.error(e)
  }
}

const handleDeleteCollection = async (id, name) => {
  try {
    await ElMessageBox.confirm(`确定要删除收藏夹 "${name}" 吗？`, '删除收藏夹', {
      type: 'warning'
    })
    await invoke('galgame_delete_collection', { id })
    emit('update-collections')
    ElMessage.success('已删除收藏夹')
  } catch (e) {
    if (e !== 'cancel') console.error(e)
  }
}
</script>

<template>
  <div class="vnite-library-bar">
    <!-- Redesigned Windows-style Context Menu -->
    <VniteContextMenu
      v-model:visible="menuVisible"
      :x="menuX"
      :y="menuY"
      :items="menuItems"
      @command="handleMenuCommand"
    />

    <div class="lb-header">
      <div class="lb-search-wrap">
        <el-input
          v-model="keyword"
          placeholder="搜索库..."
          size="small"
          clearable
          :prefix-icon="Search"
          class="lb-search"
        />
      </div>
      
      <div class="lb-filter-tabs">
        <div 
          class="lb-tab" 
          :class="{ active: groupBy === 'all' }" 
          @click="groupBy = 'all'"
          title="全部"
        >
          <el-icon><Folder /></el-icon>
        </div>
        <div 
          class="lb-tab" 
          :class="{ active: groupBy === 'developer' }" 
          @click="groupBy = 'developer'"
          title="按开发商"
        >
          <el-icon><User /></el-icon>
        </div>
        <div 
          class="lb-tab" 
          :class="{ active: groupBy === 'status' }" 
          @click="groupBy = 'status'"
          title="按状态"
        >
          <el-icon><Timer /></el-icon>
        </div>
        <div 
          class="lb-tab" 
          :class="{ active: groupBy === 'collection' }" 
          @click="groupBy = 'collection'"
          title="收藏夹"
        >
          <el-icon><CollectionTag /></el-icon>
        </div>
      </div>
    </div>

    <div class="lb-content">
      <el-scrollbar>
        <!-- 普通分组 -->
        <template v-if="groupBy !== 'collection'">
          <div v-for="group in groupedData" :key="group.name" class="lb-group">
            <div class="lb-group-header">
              <span class="lb-group-name">{{ group.name }}</span>
              <span class="lb-group-count">{{ group.items.length }}</span>
            </div>
            <div class="lb-items">
              <div 
                v-for="game in group.items" 
                :key="game.name"
                :class="['lb-game-item', { active: selectedGame?.name === game.name }]"
                @click="emit('select-game', game)"
                @contextmenu.prevent="onContextMenu($event, game)"
              >
                <div 
                  class="lb-game-dot" 
                  :style="{ background: getStatusColor(game.status) }"
                ></div>
                <span class="lb-game-name" :title="game.name">{{ game.name }}</span>
              </div>
            </div>
          </div>
        </template>

        <!-- 收藏夹分组 -->
        <template v-else>
          <div class="lb-collection-actions">
            <el-button size="small" link :icon="Plus" @click="handleAddCollection">新建收藏夹</el-button>
          </div>
          
          <div v-for="group in collectionGroups" :key="group.id" class="lb-group">
            <div 
              class="lb-group-header collection-header"
              @dragover.prevent
              @drop="handleDrop($event, group.id)"
            >
              <span class="lb-group-name">{{ group.name }}</span>
              <div class="lb-group-ops">
                <span class="lb-group-count">{{ group.items.length }}</span>
                <el-icon class="op-del" @click.stop="handleDeleteCollection(group.id, group.name)"><Delete /></el-icon>
              </div>
            </div>
            <div class="lb-items">
              <div 
                v-for="game in group.items" 
                :key="game.name"
                :class="['lb-game-item', { active: selectedGame?.name === game.name }]"
                @click="emit('select-game', game)"
                @contextmenu.prevent="onContextMenu($event, game)"
              >
                <div 
                  class="lb-game-dot" 
                  :style="{ background: getStatusColor(game.status) }"
                ></div>
                <span class="lb-game-name" :title="game.name">{{ game.name }}</span>
              </div>
              <div v-if="group.items.length === 0" class="lb-empty">暂无游戏</div>
            </div>
          </div>
          <div v-if="collections.length === 0" class="lb-empty-state">
            还没有收藏夹，点击上方按钮创建
          </div>
        </template>
      </el-scrollbar>
    </div>
  </div>
</template>

<style scoped lang="less">
.vnite-library-bar {
  width: 260px;
  height: 100%;
  background: var(--vnite-sidebar-bg);
  border-right: 1px solid var(--vnite-border);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  backdrop-filter: var(--vnite-blur);
}

.lb-header {
  padding: 16px 12px 8px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.lb-search-wrap {
  :deep(.el-input__wrapper) {
    background: var(--vnite-card-bg);
    box-shadow: none !important;
    border: 1px solid var(--vnite-border);
    border-radius: 6px;
    
    &:hover, &.is-focus {
      border-color: var(--el-color-primary);
    }
  }
}

.lb-filter-tabs {
  display: flex;
  background: var(--vnite-accent);
  padding: 2px;
  border-radius: 6px;
  gap: 2px;
}

.lb-tab {
  flex: 1;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--vnite-text-muted);
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.2s;
  
  &:hover {
    color: #fff;
    background: rgba(255, 255, 255, 0.05);
  }
  
  &.active {
    background: rgba(255, 255, 255, 0.1);
    color: #fff;
  }
}

.lb-content {
  flex: 1;
  overflow: hidden;
}

.lb-group {
  margin-bottom: 8px;
}

.lb-group-header {
  padding: 8px 16px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  color: var(--vnite-text-muted);
  opacity: 0.8;
  letter-spacing: 0.05em;
}

.lb-game-item {
  height: 32px;
  padding: 0 16px;
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
  transition: all 0.1s;
  
  &:hover {
    background: var(--vnite-accent);
  }
  
  &.active {
    background: rgba(var(--el-color-primary-rgb), 0.15);
    color: var(--el-color-primary);
  }
}

.lb-game-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

.lb-game-name {
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-weight: 450;
}

.lb-collection-actions {
  padding: 8px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  margin-bottom: 8px;
}

.collection-header {
  &:hover .op-del {
    display: inline-flex;
  }
}

.lb-group-ops {
  display: flex;
  align-items: center;
  gap: 8px;
}

.op-del {
  display: none;
  cursor: pointer;
  font-size: 12px;
  &:hover {
    color: #f56c6c;
  }
}

.lb-empty {
  padding: 4px 16px;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.2);
  font-style: italic;
}

.lb-empty-state {
  padding: 40px 20px;
  text-align: center;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.2);
  line-height: 1.6;
}
</style>
