<script setup>
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import {
  ArrowLeft, Picture, VideoPlay, ArrowDown, Check, Close,
  Plus, FolderOpened, MagicStick, Delete, Calendar, StarFilled,
  CircleClose, CollectionTag
} from '@element-plus/icons-vue'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox } from 'element-plus'
import VniteImage from './VniteImage.vue'

const props = defineProps({
  game: {
    type: Object,
    required: true
  },
  snapshots: {
    type: Array,
    default: () => []
  },
  cloudEnabled: {
    type: Boolean,
    default: false
  },
  settings: {
    type: Object,
    default: () => ({ nsfw_blur: true, nsfw_blur_intensity: 20 })
  },
  collections: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits([
  'close', 'create-snapshot', 'restore-snapshot', 'delete-snapshot',
  'open-backup', 'open-scraper', 'delete-cloud-backups', 'change-status',
  'launch-game', 'update-collections'
])

const activeTab = ref('overview')
const isRunning = ref(false)

const hasExePath = computed(() => !!props.game.exe_path)

let unlistenRunning = null
let unlistenStopped = null

onMounted(async () => {
  // Check if current game is already running
  try {
    const runningGame = await invoke('galgame_get_running_game')
    if (runningGame === props.game.name) {
      isRunning.value = true
    }
  } catch (e) {
    console.error('Failed to check running status:', e)
  }

  // Listen for global events
  unlistenRunning = await listen('galgame-game-running', (event) => {
    if (event.payload === props.game.name) {
      isRunning.value = true
    }
  })

  unlistenStopped = await listen('galgame-game-stopped', (event) => {
    if (event.payload === props.game.name) {
      isRunning.value = false
    }
  })
})

onUnmounted(() => {
  if (unlistenRunning) unlistenRunning()
  if (unlistenStopped) unlistenStopped()
})

async function handleKillGame() {
  try {
    await invoke('galgame_kill_game', { gameName: props.game.name })
    isRunning.value = false
  } catch (e) {
    console.error('Failed to kill game:', e)
  }
}

const bannerUrl = ref('')

const resolveBannerUrl = async (path) => {
  if (!path || !path.trim()) {
    bannerUrl.value = ''
    return
  }
  if (path.startsWith('http') || path.startsWith('data:')) {
    bannerUrl.value = path
    return
  }
  try {
    const dataUrl = await invoke('read_image_as_data_url', { path })
    bannerUrl.value = dataUrl
  } catch (e) {
    console.error('Failed to resolve banner:', e)
  }
}

watch(() => props.game?.background_image || props.game?.cover_image, (newPath) => {
  if (newPath) resolveBannerUrl(newPath)
}, { immediate: true })

function getGameStatusType(status) {
  const map = {
    NotStarted: 'info',
    Playing: 'primary',
    Partial: '',
    Finished: 'success',
    Multiple: 'danger',
    Shelved: 'warning',
  }
  return map[status] || 'info'
}

function getGameStatusLabel(status) {
  const map = {
    NotStarted: '未开始',
    Playing: '游玩中',
    Partial: '部分通关',
    Finished: '已通关',
    Multiple: '多周目',
    Shelved: '搁置',
  }
  return map[status] || '未知'
}

function formatPlayTime(seconds) {
  if (!seconds) return '0小时'
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  if (h > 0) return `${h}小时${m}分`
  return `${m}分钟`
}

function formatLastPlayed(timestamp) {
  if (!timestamp) return ''
  const date = new Date(timestamp * 1000)
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`
}

function formatDate(dateStr) {
  if (!dateStr) return ''
  return dateStr.replace(/_/g, ' ').replace(/-(\d{2})-(\d{2})$/, ' $1:$2')
}

function formatSize(bytes) {
  if (!bytes) return ''
  const units = ['B', 'KB', 'MB', 'GB']
  let i = 0
  while (bytes >= 1024 && i < units.length - 1) {
    bytes /= 1024
    i++
  }
  return bytes.toFixed(1) + ' ' + units[i]
}

const gameCollections = computed(() => {
  return props.collections.filter(c => c.games.includes(props.game.name))
})

async function handleToggleCollection(col) {
  try {
    const isAdded = col.games.includes(props.game.name)
    if (isAdded) {
      await invoke('galgame_remove_from_collection', { collectionId: col.id, gameName: props.game.name })
      ElMessage.success(`已从 "${col.name}" 移除`)
    } else {
      await invoke('galgame_add_to_collection', { collectionId: col.id, gameName: props.game.name })
      ElMessage.success(`已添加到 "${col.name}"`)
    }
    emit('update-collections')
  } catch (e) {
    ElMessage.error('操作失败: ' + e)
  }
}

</script>

<template>
  <div class="vnite-game-detail" v-if="game">
    <!-- Header Banner -->
    <div 
      class="detail-banner" 
      :style="{ 
        backgroundImage: bannerUrl ? `url(${bannerUrl})` : 'none',
        filter: (game.nsfw && settings.nsfw_blur) ? `blur(${settings.nsfw_blur_intensity}px)` : 'none'
      }"
    >
      <div class="banner-overlay">
        <el-button class="back-btn" circle @click="$emit('close')">
          <el-icon><ArrowLeft /></el-icon>
        </el-button>
        
        <div class="header-main-content">
          <div class="game-cover-box">
            <VniteImage 
              :src="game.cover_image"
              radius="12px"
              class="cover-image-main"
              :style="{
                '--nsfw-blur': (game.nsfw && settings.nsfw_blur) ? `${settings.nsfw_blur_intensity}px` : '0px'
              }"
            />
          </div>
          
          <div class="game-title-info">
            <div class="title-row">
              <h1>{{ game.name }}</h1>
              <div class="score-badge" v-if="game.score">
                <el-icon><StarFilled /></el-icon>
                <span>{{ (game.score / 10).toFixed(1) }}</span>
              </div>
            </div>
            
            <div class="tags-row">
              <el-tag size="small" :type="getGameStatusType(game.status)">{{ getGameStatusLabel(game.status) }}</el-tag>
              
              <template v-if="gameCollections.length">
                <el-tag 
                  v-for="col in gameCollections" 
                  :key="col.id" 
                  size="small" 
                  type="success" 
                  effect="dark"
                  class="collection-tag"
                >
                  <el-icon><CollectionTag /></el-icon> {{ col.name }}
                </el-tag>
              </template>

              <el-tag size="small" type="info" v-if="game.developer">{{ game.developer }}</el-tag>
              
              <div class="platform-tags" v-if="game.platforms && game.platforms.length">
                <el-tag v-for="p in game.platforms" :key="p" size="small" effect="plain" class="platform-tag">
                  {{ p }}
                </el-tag>
              </div>

              <el-dropdown trigger="click">
                 <el-button class="status-edit-btn" size="small" round>
                   管理收藏<el-icon class="el-icon--right"><Plus /></el-icon>
                 </el-button>
                 <template #dropdown>
                   <el-dropdown-menu>
                     <el-dropdown-item v-for="col in collections" :key="col.id" @click="handleToggleCollection(col)">
                       <el-icon v-if="col.games.includes(game.name)"><Check /></el-icon>
                       {{ col.name }}
                     </el-dropdown-item>
                     <el-dropdown-item v-if="collections.length === 0" disabled>暂无收藏夹</el-dropdown-item>
                   </el-dropdown-menu>
                 </template>
              </el-dropdown>

              <el-dropdown trigger="click" @command="$emit('change-status', $event)">
                 <el-button class="status-edit-btn" size="small" round>
                   修改状态<el-icon class="el-icon--right"><ArrowDown /></el-icon>
                 </el-button>
                 <template #dropdown>
                   <el-dropdown-menu>
                     <el-dropdown-item command="NotStarted">未开始</el-dropdown-item>
                     <el-dropdown-item command="Playing">游玩中</el-dropdown-item>
                     <el-dropdown-item command="Partial">部分通关</el-dropdown-item>
                     <el-dropdown-item command="Finished">已通关</el-dropdown-item>
                     <el-dropdown-item command="Multiple">多周目</el-dropdown-item>
                     <el-dropdown-item command="Shelved">搁置</el-dropdown-item>
                   </el-dropdown-menu>
                 </template>
              </el-dropdown>
            </div>
            
            <div class="actions-row">
              <el-button 
                v-if="!isRunning"
                type="primary" 
                size="large" 
                class="launch-btn" 
                @click="$emit('launch-game', game)"
                :disabled="!hasExePath"
              >
                <el-icon :size="20"><VideoPlay /></el-icon>
                <span>开始游戏</span>
              </el-button>
              <el-button 
                v-else
                type="danger" 
                size="large" 
                class="launch-btn running" 
                @click="handleKillGame"
              >
                <el-icon :size="20"><CircleClose /></el-icon>
                <span>正在运行 (强制结束)</span>
              </el-button>
              
              <div class="quick-stats">
                <div class="stat-box">
                  <span class="label">游玩时间</span>
                  <span class="value">{{ formatPlayTime(game.total_play_time) }}</span>
                </div>
                <div class="stat-box">
                  <span class="label">最后运行</span>
                  <span class="value">{{ formatLastPlayed(game.last_played) || '从未' }}</span>
                </div>
                <div class="stat-box">
                  <span class="label">同步</span>
                  <span class="value">
                    <el-icon :color="cloudEnabled ? '#67C23A' : '#909399'">
                      <Check v-if="cloudEnabled" /><Close v-else />
                    </el-icon>
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Scrollable Body -->
    <div class="detail-body">
      <el-tabs v-model="activeTab" class="detail-tabs">
        <el-tab-pane label="概览" name="overview">
          <div class="pane-content overview">
            <div class="info-section">
              <div class="section-header">
                <h3>关于游戏</h3>
                <el-button 
                  class="scraper-glass-btn" 
                  size="small" 
                  @click="$emit('open-scraper', game)"
                >
                  <el-icon><MagicStick /></el-icon> <span>搜刮元数据</span>
                </el-button>
              </div>
              <p v-if="game.description" class="description">{{ game.description }}</p>
              <el-empty v-else description="暂无游戏简介" :image-size="60" />
              
              <div class="meta-grid" v-if="(game.genres && game.genres.length) || (game.tags && game.tags.length)">
                <div class="meta-group" v-if="game.genres && game.genres.length">
                  <h4>游戏类型</h4>
                  <div class="tags">
                    <el-tag v-for="g in game.genres" :key="g" size="small">{{ g }}</el-tag>
                  </div>
                </div>
                <div class="meta-group" v-if="game.tags && game.tags.length">
                  <h4>游戏标签</h4>
                  <div class="tags">
                    <el-tag v-for="t in game.tags.slice(0, 20)" :key="t" size="small" type="info" effect="plain">{{ t }}</el-tag>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </el-tab-pane>
        
        <el-tab-pane label="备份与快照" name="snapshots">
          <div class="pane-content snapshots">
            <div class="snapshots-toolbar">
              <el-button type="primary" @click="$emit('create-snapshot', game)">
                <el-icon><Plus /></el-icon> 创建快照
              </el-button>
              <el-button @click="$emit('open-backup')">
                <el-icon><FolderOpened /></el-icon> 本地存档目录
              </el-button>
              <el-button v-if="cloudEnabled" type="danger" plain @click="$emit('delete-cloud-backups')">
                <el-icon><Delete /></el-icon> 清空云端
              </el-button>
            </div>
            
            <el-table :data="snapshots" style="width: 100%" v-if="snapshots?.length">
              <el-table-column label="创建时间" width="160">
                <template #default="scope">
                  <span class="snapshot-date">{{ formatDate(scope.row.date) }}</span>
                </template>
              </el-table-column>
              <el-table-column label="状态" width="100">
                <template #default="scope">
                  <el-tag v-if="scope.row.status" size="small" :type="getGameStatusType(scope.row.status)">
                    {{ getGameStatusLabel(scope.row.status) }}
                  </el-tag>
                </template>
              </el-table-column>
              <el-table-column label="游玩时长" width="120">
                <template #default="scope">
                  <span class="snapshot-playtime" v-if="scope.row.total_play_time !== undefined">
                    {{ formatPlayTime(scope.row.total_play_time) }}
                  </span>
                </template>
              </el-table-column>
              <el-table-column prop="describe" label="备注" min-width="120">
                <template #default="scope">
                  <span class="snapshot-desc">{{ scope.row.describe || '自动备份' }}</span>
                </template>
              </el-table-column>
              <el-table-column label="大小" width="90">
                <template #default="scope">
                  <span class="snapshot-size">{{ formatSize(scope.row.size) }}</span>
                </template>
              </el-table-column>
              <el-table-column label="操作" width="160" fixed="right">
                <template #default="scope">
                  <el-button size="small" @click="$emit('restore-snapshot', scope.row)">恢复</el-button>
                  <el-button size="small" type="danger" plain @click="$emit('delete-snapshot', scope.row)">删除</el-button>
                </template>
              </el-table-column>
            </el-table>
            <el-empty v-else description="暂无存档快照" />
          </div>
        </el-tab-pane>
      </el-tabs>
    </div>
  </div>
</template>

<style scoped lang="less">
.vnite-game-detail {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--vnite-bg);
  color: var(--vnite-text);
  overflow: hidden;
}

.detail-banner {
  height: 360px;
  background-size: cover;
  background-position: center;
  position: relative;
  flex-shrink: 0;

  &::after {
    content: '';
    position: absolute;
    inset: 0;
    background: var(--vnite-header-overlay);
    backdrop-filter: blur(40px);
  }
}

.banner-overlay {
  position: relative;
  z-index: 1;
  height: 100%;
  padding: 30px;
  display: flex;
  flex-direction: column;
}

.back-btn {
  background: rgba(255, 255, 255, 0.1);
  border: none;
  color: #fff;
  backdrop-filter: blur(10px);
  
  &:hover {
    background: rgba(255, 255, 255, 0.2);
    transform: translateX(-4px);
  }
}

.header-main-content {
  margin-top: auto;
  display: flex;
  gap: 30px;
  align-items: flex-end;
}

.game-cover-box {
  width: 180px;
  height: 250px;
  border-radius: 12px;
  overflow: hidden;
  box-shadow: var(--vnite-shadow);
  flex-shrink: 0;
  background: var(--vnite-card-bg);
  border: 1px solid var(--vnite-border);

  .cover-image-main { 
    width: 100%; 
    height: 100%; 
    transition: transform 0.5s ease;
  }
  
  &:hover .cover-image-main {
    transform: scale(1.05);
  }

  .image-placeholder {
    height: 100%; display: flex; align-items: center; justify-content: center;
    color: var(--vnite-text-muted); font-size: 32px;
    background: var(--vnite-accent);
  }
}

.game-title-info {
  flex: 1;
  min-width: 0;
  color: var(--vnite-text);
  padding-bottom: 10px;

  .title-row {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 12px;

    h1 {
      font-size: 36px;
      font-weight: 800;
      margin: 0;
      text-shadow: var(--vnite-title-shadow);
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      color: var(--vnite-text);
    }

    .score-badge {
      display: flex;
      align-items: center;
      gap: 4px;
      background: rgba(255, 193, 7, 0.2);
      border: 1px solid rgba(255, 193, 7, 0.4);
      color: #ffc107;
      padding: 4px 10px;
      border-radius: 8px;
      font-weight: 700;
      font-size: 18px;
    }
  }
}

.tags-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 24px;
  flex-wrap: wrap;

  .el-tag {
    background: var(--vnite-accent);
    border: 1px solid var(--vnite-border);
    color: var(--vnite-text-muted);
  }

  .status-edit-btn {
    background: transparent;
    border: 1px dashed var(--vnite-border);
    color: var(--vnite-text-muted);
    &:hover { color: var(--vnite-text); border-color: var(--vnite-primary); }
  }
}

.collection-tag {
  display: flex;
  align-items: center;
  gap: 4px;
  background: var(--el-color-success-dark-2) !important;
  border: none !important;
  color: #fff !important;
  font-weight: 500;
}

.actions-row {
  display: flex;
  align-items: center;
  gap: 30px;

  .launch-btn {
    height: 52px;
    padding: 0 32px;
    border-radius: 26px;
    font-size: 18px;
    font-weight: 700;
    box-shadow: 0 8px 16px rgba(var(--el-color-primary-rgb), 0.3);
  }
}

.quick-stats {
  display: flex;
  gap: 24px;

  .stat-box {
    display: flex;
    flex-direction: column;
    gap: 4px;

    .label { font-size: 11px; color: var(--vnite-text-muted); opacity: 0.6; text-transform: uppercase; letter-spacing: 1px; }
    .value { font-size: 15px; font-weight: 500; color: var(--vnite-text); }
  }
}

.detail-body {
  flex: 1;
  padding: 0 30px 30px;
  overflow-y: auto;

  :deep(.el-tabs__header) { margin-bottom: 24px; }
  :deep(.el-tabs__item) { color: var(--vnite-text-muted); height: 50px; font-size: 15px; }
  :deep(.el-tabs__active-bar) { height: 3px; }
}

.pane-content {
  color: var(--vnite-text);
  
  &.overview {
    .info-section {
      background: var(--vnite-card-bg);
      border: 1px solid var(--vnite-border);
      border-radius: 16px;
      padding: 24px;

      .section-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 20px;
        h3 { margin: 0; font-size: 18px; color: var(--vnite-primary); font-weight: 700; }
        
        .scraper-glass-btn {
          background: var(--vnite-accent);
          border: 1px solid var(--vnite-border);
          color: var(--vnite-text);
          border-radius: 8px;
          height: 32px;
          padding: 0 12px;
          transition: all 0.3s;
          
          &:hover {
            background: var(--vnite-primary);
            color: #fff;
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(var(--el-color-primary-rgb), 0.2);
          }
        }
      }

      .description {
        line-height: 1.8;
        font-size: 14px;
        color: var(--vnite-text);
        opacity: 0.8;
        white-space: pre-wrap;
      }
    }

    .meta-grid {
      margin-top: 30px;
      display: flex;
      flex-direction: column;
      gap: 24px;
      padding-top: 24px;
      border-top: 1px solid rgba(255, 255, 255, 0.08);

      h4 { margin: 0 0 12px; font-size: 14px; color: rgba(255, 255, 255, 0.4); }
      .tags { display: flex; flex-wrap: wrap; gap: 8px; }
    }
  }

  &.snapshots {
    .snapshots-toolbar {
      display: flex;
      gap: 12px;
      margin-bottom: 20px;
    }
  }
}

.snapshot-date { font-weight: 500; }
.snapshot-desc { color: rgba(255, 255, 255, 0.6); }
.snapshot-size { color: rgba(255, 255, 255, 0.4); font-size: 12px; }

:deep(.el-table) {
  --el-table-bg-color: transparent;
  --el-table-tr-bg-color: transparent;
  --el-table-header-bg-color: var(--vnite-accent);
  --el-table-border-color: var(--vnite-border);
  color: var(--vnite-text);
  
  .el-table__header th { color: var(--vnite-text-muted); font-weight: 500; }
  .el-table__cell { padding: 12px 0; border-bottom: 1px solid var(--el-table-border-color); }
  
  &:before, &:after { display: none; }
}
</style>
