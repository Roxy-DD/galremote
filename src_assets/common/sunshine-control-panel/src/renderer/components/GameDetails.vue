<template>
  <div class="game-details" v-if="game">
    <!-- Header Banner -->
    <div class="game-header" :style="{ backgroundImage: `url(${getCoverUrl(game)})` }">
      <div class="header-overlay">
        <el-button class="back-btn" @click="$emit('close')" circle>
          <el-icon><ArrowLeft /></el-icon>
        </el-button>
        
        <div class="header-content">
          <div class="game-cover-large">
            <el-image :src="getCoverUrl(game)" fit="cover" lazy>
              <template #error>
                <div class="image-slot"><el-icon><Picture /></el-icon></div>
              </template>
            </el-image>
          </div>
          
            <div class="title-with-score">
              <h1>{{ game.name }}</h1>
              <div class="game-score-badge" v-if="game.score">
                <el-icon><StarFilled /></el-icon>
                <span>{{ (game.score / 10).toFixed(1) }}</span>
              </div>
            </div>
            <div class="game-meta-tags">
              <el-tag size="small" type="info" v-if="game.developer">{{ game.developer }}</el-tag>
              <el-tag size="small" type="info" v-if="game.release_date">{{ game.release_date }}</el-tag>
              
              <div class="platform-indicators" v-if="game.platforms && game.platforms.length">
                <el-tag v-for="p in game.platforms" :key="p" size="small" effect="plain" class="platform-tag">
                  {{ p }}
                </el-tag>
              </div>
              
              <el-dropdown trigger="click" @command="$emit('change-status', $event)">
                 <el-button class="status-btn-small" :type="getGameStatusType(game.status)" size="small" round>
                   {{ getGameStatusLabel(game.status) }}<el-icon class="el-icon--right"><ArrowDown /></el-icon>
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
            
            <div class="play-action-area">
              <el-button 
                type="success" 
                size="large" 
                class="play-btn" 
                @click="$emit('launch-game', game)"
                :disabled="!hasExePath"
              >
                <el-icon :size="20"><VideoPlay /></el-icon>
                <span>开始游戏</span>
              </el-button>
              
              <div class="play-stats">
                <div class="stat-item">
                  <span class="stat-label">游玩时间</span>
                  <span class="stat-value">{{ formatPlayTime(game.total_play_time) || '--' }}</span>
                </div>
                <div class="stat-item">
                  <span class="stat-label">最后运行</span>
                  <span class="stat-value">{{ formatLastPlayed(game.last_played) || '从未' }}</span>
                </div>
                <div class="stat-item cloud-stat">
                  <span class="stat-label">云同步状态</span>
                  <span class="stat-value sync-status">
                    <el-icon v-if="cloudEnabled" color="#67C23A"><Check /></el-icon>
                    <el-icon v-else color="#909399"><Close /></el-icon>
                    {{ cloudEnabled ? '受保护' : '未开启' }}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Main Content Area -->
    <div class="game-body">
      <el-tabs v-model="activeTab" class="game-tabs">
        <el-tab-pane label="概览" name="overview">
          <div class="overview-pane">
            <div class="description-card">
              <h3>关于游戏</h3>
              <p v-if="game.description" class="description-text">{{ game.description }}</p>
              <el-empty v-else description="暂无游戏简介" :image-size="60"></el-empty>
              
              <div class="additional-meta" v-if="(game.genres && game.genres.length) || (game.tags && game.tags.length)">
                <div class="meta-block" v-if="game.genres && game.genres.length">
                  <h4>类型</h4>
                  <div class="tag-group">
                    <el-tag v-for="g in game.genres" :key="g" size="small" effect="light">{{ g }}</el-tag>
                  </div>
                </div>
                <div class="meta-block" v-if="game.tags && game.tags.length">
                  <h4>标签</h4>
                  <div class="tag-group">
                    <el-tag v-for="t in game.tags.slice(0, 15)" :key="t" size="small" type="info" effect="plain">{{ t }}</el-tag>
                    <span v-if="game.tags.length > 15" class="more-tags">...</span>
                  </div>
                </div>
              </div>
              
              <div style="margin-top: 20px;">
                <el-button type="primary" plain @click="$emit('open-scraper', game)">
                  <el-icon><MagicStick /></el-icon>
                  搜刮元数据 (VNDB)
                </el-button>
              </div>
            </div>
          </div>
        </el-tab-pane>
        
        <el-tab-pane label="云存档与快照" name="snapshots">
          <div class="snapshots-pane">
            <div class="snapshots-toolbar">
              <el-button type="primary" @click="$emit('create-snapshot', game)">
                <el-icon><Plus /></el-icon>
                创建游戏快照
              </el-button>
              <el-button @click="$emit('open-backup')">
                <el-icon><FolderOpened /></el-icon>
                打开本地安全备份目录
              </el-button>
              <el-button v-if="cloudEnabled" type="danger" plain @click="$emit('delete-cloud-backups')">
                <el-icon><Delete /></el-icon>
                清空云端备份
              </el-button>
            </div>
            
            <div class="snapshots-list">
              <el-table :data="snapshots" style="width: 100%" v-if="snapshots && snapshots.length > 0">
                <el-table-column label="时间" width="200">
                  <template #default="scope">
                    <div style="display: flex; align-items: center; gap: 8px;">
                      <el-icon><Calendar /></el-icon>
                      <span>{{ formatDate(scope.row.date) }}</span>
                    </div>
                  </template>
                </el-table-column>
                <el-table-column prop="describe" label="备注" min-width="150">
                  <template #default="scope">
                    {{ scope.row.describe || '自动快照' }}
                  </template>
                </el-table-column>
                <el-table-column label="来源设备" width="120">
                  <template #default="scope">
                    <el-tag size="small" v-if="scope.row.device_id">{{ scope.row.device_id.substring(0, 8) }}</el-tag>
                    <span v-else>--</span>
                  </template>
                </el-table-column>
                <el-table-column label="大小" width="100">
                  <template #default="scope">
                    {{ formatSize(scope.row.size) }}
                  </template>
                </el-table-column>
                <el-table-column label="操作" width="180" fixed="right">
                  <template #default="scope">
                    <el-button size="small" type="primary" @click="$emit('restore-snapshot', scope.row)">
                      恢复
                    </el-button>
                    <el-button size="small" type="danger" @click="$emit('delete-snapshot', scope.row)">
                      删除
                    </el-button>
                  </template>
                </el-table-column>
              </el-table>
              <el-empty v-else description="暂无存档快照" />
            </div>
          </div>
        </el-tab-pane>
      </el-tabs>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import {
  ArrowLeft, Picture, VideoPlay, ArrowDown, Check, Close,
  Plus, FolderOpened, MagicStick, Delete, Calendar, StarFilled
} from '@element-plus/icons-vue'

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
  }
})

defineEmits([
  'close', 'create-snapshot', 'restore-snapshot', 'delete-snapshot',
  'open-backup', 'open-scraper', 'delete-cloud-backups', 'change-status',
  'launch-game'
])

const activeTab = ref('overview')

const hasExePath = computed(() => !!props.game.exe_path)

function getCoverUrl(game) {
  if (game && game.cover_image && game.cover_image.trim()) {
    // 处理 Windows 反斜杠转义
    const path = game.cover_image.replace(/\\/g, '/')
    return `https://asset.localhost/${encodeURIComponent(path)}`
  }
  return ''
}

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
</script>

<style scoped lang="less">
.game-details {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  background-color: var(--el-bg-color-page);
  overflow-y: auto;
}

.game-header {
  position: relative;
  height: 380px;
  background-size: cover;
  background-position: center;
  background-repeat: no-repeat;
  flex-shrink: 0;

  &::before {
    content: '';
    position: absolute;
    inset: 0;
    backdrop-filter: blur(20px);
    background: linear-gradient(to top, var(--el-bg-color) 0%, rgba(0, 0, 0, 0.6) 50%, rgba(0, 0, 0, 0.8) 100%);
  }
}

.header-overlay {
  position: relative;
  height: 100%;
  padding: 40px;
  display: flex;
  flex-direction: column;
  color: #fff;
}

.back-btn {
  position: absolute;
  top: 20px;
  left: 20px;
  background: rgba(255, 255, 255, 0.2);
  border: none;
  color: #fff;
  transition: all 0.3s ease;
  z-index: 10;
  
  &:hover {
    background: rgba(255, 255, 255, 0.4);
    transform: translateX(-5px);
  }
}

.header-content {
  display: flex;
  gap: 40px;
  margin-top: auto;
  align-items: flex-end;
}

.game-cover-large {
  width: 200px;
  height: 280px;
  border-radius: 8px;
  overflow: hidden;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
  background: var(--el-fill-color-dark);
  flex-shrink: 0;
  border: 1px solid rgba(255, 255, 255, 0.1);
  
  .el-image {
    width: 100%;
    height: 100%;
    
    .image-slot {
      display: flex;
      justify-content: center;
      align-items: center;
      width: 100%;
      height: 100%;
      background: var(--el-fill-color-dark);
      color: var(--el-text-color-secondary);
      font-size: 48px;
    }
  }
}

.game-title-area {
  flex: 1;
  padding-bottom: 20px;
  
  h1 {
    font-size: 42px;
    margin: 0;
    text-shadow: 0 2px 4px rgba(0,0,0,0.5);
    font-weight: 700;
  }
}

.title-with-score {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 15px;
}

.game-score-badge {
  display: flex;
  align-items: center;
  gap: 4px;
  background: rgba(255, 193, 7, 0.2);
  border: 1px solid rgba(255, 193, 7, 0.4);
  color: #ffc107;
  padding: 4px 10px;
  border-radius: 6px;
  font-weight: bold;
  font-size: 18px;
  
  .el-icon {
    font-size: 16px;
  }
}

.platform-indicators {
  display: flex;
  gap: 8px;
  margin-right: 12px;
  
  .platform-tag {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    color: #bbb;
    font-size: 11px;
    text-transform: uppercase;
  }
}

.game-meta-tags {
  display: flex;
  gap: 12px;
  margin-bottom: 30px;
  align-items: center;
  
  .el-tag {
    background: rgba(255, 255, 255, 0.15);
    border: 1px solid rgba(255, 255, 255, 0.3);
    color: #fff;
  }
}

.play-action-area {
  display: flex;
  align-items: center;
  gap: 40px;
}

.play-btn {
  height: 56px;
  padding: 0 40px;
  font-size: 20px;
  font-weight: bold;
  border-radius: 28px;
  letter-spacing: 2px;
  background: linear-gradient(135deg, #67C23A, #529b2e);
  border: none;
  box-shadow: 0 6px 15px rgba(103, 194, 58, 0.4);
  transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
  
  &:hover:not(:disabled) {
    transform: translateY(-2px) scale(1.02);
    box-shadow: 0 8px 20px rgba(103, 194, 58, 0.5);
  }
  
  &:active:not(:disabled) {
    transform: translateY(1px);
  }
}

.play-stats {
  display: flex;
  gap: 30px;
  
  .stat-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
    
    .stat-label {
      font-size: 12px;
      color: rgba(255, 255, 255, 0.6);
      text-transform: uppercase;
      letter-spacing: 1px;
    }
    
    .stat-value {
      font-size: 16px;
      font-weight: 500;
      color: #fff;
      display: flex;
      align-items: center;
      gap: 6px;
    }
  }
}

.game-body {
  padding: 0 40px 40px;
  flex: 1;
}

.game-tabs {
  margin-top: -10px; // Pull tabs up slightly to overlap fade
  
  :deep(.el-tabs__item) {
    font-size: 16px;
    padding: 0 20px;
    height: 50px;
    line-height: 50px;
  }
}

.overview-pane, .snapshots-pane {
  padding-top: 20px;
}

.description-card {
  background: var(--el-bg-color);
  border-radius: 8px;
  padding: 24px;
  box-shadow: 0 2px 12px 0 rgba(0,0,0,0.05);
  border: 1px solid var(--el-border-color-light);
  line-height: 1.6;
  font-size: 15px;
  color: var(--el-text-color-regular);
  
  h3 {
    margin-top: 0;
    margin-bottom: 16px;
    color: var(--el-text-color-primary);
  }
}

.description-text {
  white-space: pre-wrap;
  color: var(--el-text-color-regular);
}

.additional-meta {
  margin-top: 32px;
  display: flex;
  flex-direction: column;
  gap: 24px;
  padding-top: 24px;
  border-top: 1px solid var(--el-border-color-lighter);
}

.meta-block {
  h4 {
    margin: 0 0 12px 0;
    font-size: 14px;
    color: var(--el-text-color-secondary);
  }
}

.tag-group {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  
  .more-tags {
    font-size: 12px;
    color: var(--el-text-color-secondary);
    align-self: flex-end;
  }
}

.snapshots-toolbar {
  margin-bottom: 20px;
  display: flex;
  gap: 12px;
}
</style>
