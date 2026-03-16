<script setup>
import { computed, ref, watch } from 'vue'
import { 
  DocumentAdd, 
  VideoPlay, 
  ArrowDown, 
  Picture,
  FolderOpened,
  Search,
  Check
} from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import VniteImage from './VniteImage.vue'

const props = defineProps({
  games: {
    type: Array,
    default: () => []
  },
  selectedGame: Object,
  cloudEnabled: Boolean,
  scanning: Boolean,
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
  'select-game', 
  'create-snapshot', 
  'launch-game', 
  'edit-game', 
  'open-scraper', 
  'delete-game',
  'scan',
  'update-collections'
])

const recentGames = computed(() => {
  return [...props.games]
    .filter(g => g.last_played)
    .sort((a, b) => new Date(b.last_played) - new Date(a.last_played))
    .slice(0, 4)
})

const coverUrls = ref({})

const resolveCoverUrl = async (game) => {
  if (!game.cover_image) return
  if (game.cover_image.startsWith('http')) {
    coverUrls.value[game.name] = game.cover_image
    return
  }
  try {
    const dataUrl = await invoke('read_image_as_data_url', { path: game.cover_image })
    coverUrls.value[game.name] = dataUrl
  } catch (e) {
    console.warn('Failed to resolve cover for', game.name, e)
  }
}

watch(() => props.games, (newGames) => {
  newGames.forEach(game => {
    if (!coverUrls.value[game.name]) {
      resolveCoverUrl(game)
    }
  })
}, { immediate: true, deep: true })

const getCoverUrl = (game) => {
  return coverUrls.value[game.name] || ''
}

const formatPlayTime = (ms) => {
  if (!ms) return '0 分钟'
  const minutes = Math.floor(ms / 60000)
  if (minutes < 60) return `${minutes} 分钟`
  const hours = (minutes / 60).toFixed(1)
  return `${hours} 小时`
}

const getGameStatusLabel = (status) => {
  const map = {
    'NotStarted': '未开始',
    'Playing': '正在玩',
    'Finished': '已通关',
    'Partial': '部分通关',
    'Multiple': '多周目',
    'Shelved': '搁置'
  }
  return map[status] || '未开始'
}

const getGameStatusColor = (status) => {
  const map = {
    'Playing': 'var(--el-color-primary)',
    'Finished': 'var(--el-color-success)',
    'Partial': 'var(--el-color-warning)',
    'Multiple': 'var(--el-color-danger)',
    'Shelved': 'var(--el-text-color-secondary)'
  }
  return map[status] || 'transparent'
}

const handleToggleCollection = async (game, collection) => {
  try {
    const isAdded = collection.games.includes(game.name)
    if (isAdded) {
      await invoke('galgame_remove_from_collection', { collectionId: collection.id, gameName: game.name })
      ElMessage.success(`已从 "${collection.name}" 中移除`)
    } else {
      await invoke('galgame_add_to_collection', { collectionId: collection.id, gameName: game.name })
      ElMessage.success(`已添加到 "${collection.name}"`)
    }
    emit('update-collections')
  } catch (e) {
    ElMessage.error('操作失败: ' + e)
  }
}

</script>

<template>
  <div class="vnite-gallery">
    <el-scrollbar class="gallery-scroll">
      <div v-if="games.length === 0" class="empty-state">
        <el-icon :size="80" class="empty-icon"><FolderOpened /></el-icon>
        <h3>尚未添加任何游戏</h3>
        <p>点击下方按钮自动扫描目录，或在侧边栏手动添加</p>
        <el-button type="primary" size="large" @click="emit('scan')" :loading="scanning">
          <el-icon><Search /></el-icon>
          开始扫描
        </el-button>
      </div>

      <div v-else class="gallery-content">
        <!-- Recently Played Showcase -->
        <section v-if="recentGames.length > 0" class="gallery-section">
          <h2 class="section-title">最近游玩</h2>
          <div class="recent-grid">
            <div 
              v-for="game in recentGames" 
              :key="game.name" 
              class="recent-card"
              @click="emit('select-game', game)"
            >
              <div class="card-cover">
                <VniteImage 
                  :src="game.cover_image"
                  :style="{ 
                    filter: (game.nsfw && settings.nsfw_blur) ? `blur(${settings.nsfw_blur_intensity}px)` : 'none'
                  }"
                />
              </div>
              <div class="recent-overlay">
                <div class="recent-info">
                  <div class="recent-name">{{ game.name }}</div>
                  <div class="recent-meta">
                    ⏱ {{ formatPlayTime(game.total_play_time) }}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </section>

        <!-- Main Library Grid -->
        <section class="gallery-section">
          <h2 class="section-title">所有游戏库</h2>
          <div class="game-grid">
            <article
              v-for="game in games"
              :key="game.name"
              class="game-card"
              :class="{ selected: selectedGame?.name === game.name }"
              @click="emit('select-game', game)"
            >
              <div
                class="game-cover"
                :style="{ 
                  backgroundImage: game.cover_image ? `url(${getCoverUrl(game)})` : '',
                  filter: (game.nsfw && settings.nsfw_blur) ? `blur(${settings.nsfw_blur_intensity}px)` : 'none'
                }"
              >
                <div v-if="!game.cover_image" class="cover-placeholder">
                  <el-icon :size="24"><Picture /></el-icon>
                </div>
                <div 
                  class="game-status-badge" 
                  v-if="game.status && game.status !== 'NotStarted'"
                  :style="{ backgroundColor: getGameStatusColor(game.status) }"
                >
                  {{ getGameStatusLabel(game.status) }}
                </div>
                <div class="game-info-overlay">
                  <div class="game-name" :title="game.name">{{ game.name }}</div>
                </div>
              </div>
              <div class="game-card-footer">
                <div class="game-actions">
                  <el-tooltip content="启动游戏" placement="top">
                    <el-button
                      size="small"
                      type="primary"
                      circle
                      @click.stop="emit('launch-game', game)"
                    >
                      <el-icon><VideoPlay /></el-icon>
                    </el-button>
                  </el-tooltip>
                  <el-tooltip content="创建快照" placement="top">
                    <el-button
                      size="small"
                      circle
                      @click.stop="emit('create-snapshot', game)"
                    >
                      <el-icon><DocumentAdd /></el-icon>
                    </el-button>
                  </el-tooltip>
                  <el-dropdown trigger="click" @click.stop>
                    <el-button size="small" circle>
                      <el-icon><ArrowDown /></el-icon>
                    </el-button>
                    <template #dropdown>
                      <el-dropdown-menu>
                        <el-dropdown-item @click="emit('edit-game', game)">编辑游戏</el-dropdown-item>
                        <el-dropdown-item @click="emit('open-scraper', game)">元数据搜刮</el-dropdown-item>
                        
                        <el-dropdown-item divided disabled style="font-size: 11px; color: rgba(255,255,255,0.4)">── 收藏夹 ──</el-dropdown-item>
                        <el-dropdown-item 
                          v-for="col in collections" 
                          :key="col.id"
                          @click="handleToggleCollection(game, col)"
                        >
                          <el-icon v-if="col.games.includes(game.name)"><Check /></el-icon>
                          <span :style="{ marginLeft: col.games.includes(game.name) ? '0' : '18px' }">{{ col.name }}</span>
                        </el-dropdown-item>
                        <el-dropdown-item v-if="collections.length === 0" disabled>暂无收藏夹</el-dropdown-item>

                        <el-dropdown-item divided @click="emit('delete-game', game)">
                          <span style="color: var(--el-color-danger)">删除游戏</span>
                        </el-dropdown-item>
                      </el-dropdown-menu>
                    </template>
                  </el-dropdown>
                </div>
              </div>
            </article>
          </div>
        </section>
      </div>
    </el-scrollbar>
  </div>
</template>

<style scoped lang="less">
.vnite-gallery {
  flex: 1;
  height: 100%;
  background: transparent;
  overflow: hidden;
}

.gallery-scroll {
  height: 100%;
}

.gallery-content {
  padding: 24px 32px;
}

.gallery-section {
  margin-bottom: 40px;
}

.section-title {
  font-size: 18px;
  font-weight: 600;
  margin-bottom: 20px;
  color: var(--vnite-text);
  opacity: 0.9;
  display: flex;
  align-items: center;
  gap: 10px;
  
  &::before {
    content: '';
    width: 4px;
    height: 18px;
    background: var(--el-color-primary);
    border-radius: 2px;
  }
}

.recent-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 20px;
}

.recent-card {
  height: 160px;
  border-radius: 12px;
  overflow: hidden;
  position: relative;
  cursor: pointer;
  box-shadow: 0 8px 16px rgba(0,0,0,0.3);
  transition: transform 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);

  &:hover {
    transform: translateY(-4px) scale(1.02);
    
    .recent-overlay {
      background: linear-gradient(transparent, rgba(0,0,0,0.8));
    }
  }
}

.recent-cover {
  width: 100%;
  height: 100%;
  background-size: cover;
  background-position: center;
  background-color: var(--vnite-accent);
}

.recent-overlay {
  position: absolute;
  inset: 0;
  background: linear-gradient(transparent, rgba(0,0,0,0.6));
  display: flex;
  align-items: flex-end;
  padding: 16px;
  transition: background 0.3s;
}

.recent-info {
  color: #fff;
}

.recent-name {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 4px;
  text-shadow: 0 2px 4px rgba(0,0,0,0.5);
}

.recent-meta {
  font-size: 12px;
  opacity: 0.8;
}

.game-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 20px;
}

.game-card {
  background: var(--vnite-card-bg);
  border: 1px solid var(--vnite-border);
  border-radius: 12px;
  overflow: hidden;
  transition: all @transition-smooth;
  box-shadow: var(--vnite-shadow);
  backdrop-filter: var(--vnite-blur);
  position: relative;
  
  &::before {
    content: '';
    position: absolute;
    inset: 0;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    z-index: 1;
    pointer-events: none;
  }

  &:hover {
    transform: translateY(-6px);
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.2);
    border-color: rgba(var(--el-color-primary-rgb), 0.5);
    
    .game-info-overlay {
       opacity: 1;
    }
  }

  &.selected {
    border-color: var(--el-color-primary);
    box-shadow: 0 0 0 1px var(--el-color-primary);
  }
}

.game-cover {
  aspect-ratio: 2/3;
  background-size: cover;
  background-position: center;
  background-color: #222;
  position: relative;
}

.game-status-badge {
  position: absolute;
  top: 8px;
  left: 8px;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 10px;
  color: #fff;
  font-weight: 600;
  box-shadow: 0 2px 4px rgba(0,0,0,0.2);
}

.game-info-overlay {
  position: absolute;
  inset: 0;
  background: linear-gradient(transparent, rgba(0,0,0,0.7));
  display: flex;
  align-items: flex-end;
  padding: 8px;
  opacity: 0.8;
  transition: opacity 0.2s;
}

.game-name {
  color: #fff;
  font-size: 12px;
  font-weight: 500;
  line-break: anywhere;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.game-card-footer {
  padding: 8px;
}

.game-actions {
  display: flex;
  justify-content: center;
  gap: 8px;
}

.cover-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: rgba(255, 255, 255, 0.1);
}

.empty-state {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding-top: 100px;
  color: var(--vnite-text-muted);

  h3 {
    margin: 20px 0 10px;
    color: var(--vnite-text);
  }
  
  p {
    margin-bottom: 30px;
  }
}

.empty-icon {
  opacity: 0.2;
}
</style>
