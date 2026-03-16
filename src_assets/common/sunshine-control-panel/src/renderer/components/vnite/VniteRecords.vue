<template>
  <div class="vnite-records">
    <div class="settings-pane">
      <div class="records-header">
        <h3>📋 游玩记录</h3>
        <p class="subtitle">历史游玩统计与存档同步概览</p>
      </div>

      <!-- 统计卡片区 -->
      <div class="stats-grid">
        <div class="stat-card">
          <div class="icon-box"><el-icon><Timer /></el-icon></div>
          <div class="content">
            <div class="value">{{ formatTotalTime(stats.totalTime) }}</div>
            <div class="label">累计游玩时长</div>
          </div>
        </div>
        <div class="stat-card">
          <div class="icon-box"><el-icon><Monitor /></el-icon></div>
          <div class="content">
            <div class="value">{{ stats.totalGames }}</div>
            <div class="label">已录入游戏数</div>
          </div>
        </div>
        <div class="stat-card">
          <div class="icon-box"><el-icon><Calendar /></el-icon></div>
          <div class="content">
            <div class="value">{{ stats.lastPlayedName || '暂无数据' }}</div>
            <div class="label">最近一次游玩</div>
          </div>
        </div>
      </div>

      <div class="records-layout">
        <!-- 最近活动列表 -->
        <div class="recent-records">
          <h4 class="section-title"><el-icon><List /></el-icon> 最近活动</h4>
          <div v-if="recentHistory.length === 0" class="empty-state">
            暂无游玩记录
          </div>
          <div v-else class="history-list">
            <div v-for="(record, index) in recentHistory" :key="index" class="history-item">
              <div class="cover-wrapper">
                <VniteImage :src="record.cover" class="game-cover" />
              </div>
              <div class="record-info">
                <div class="game-name">{{ record.gameName }}</div>
                <div class="play-details">
                  <span class="date">{{ formatDate(record.startTime) }}</span>
                  <span class="dot">·</span>
                  <span class="duration">时长: {{ formatMinutes(record.duration) }}</span>
                </div>
              </div>
              <div class="device-tag">
                <el-icon><Upload /></el-icon> {{ record.deviceId }}
              </div>
            </div>
          </div>
        </div>

        <!-- 状态分布 -->
        <div class="status-summary">
          <h4 class="section-title"><el-icon><PieChart /></el-icon> 库状态分布</h4>
          <div class="status-list">
            <div v-for="(count, status) in statusDistribution" :key="status" class="status-item">
              <span class="status-label">{{ formatStatus(status) }}</span>
              <div class="progress-bar">
                <div class="progress" :style="{ width: (count / stats.totalGames * 100) + '%', backgroundColor: getStatusColor(status) }"></div>
              </div>
              <span class="status-count">{{ count }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import VniteImage from './VniteImage.vue'
import {
  Timer, Monitor, Calendar, List, Upload, PieChart
} from '@element-plus/icons-vue'

const props = defineProps({
  games: {
    type: Array,
    default: () => []
  }
})

const stats = computed(() => {
  let totalTime = 0
  let lastPlayedTime = 0
  let lastPlayedName = ''

  props.games.forEach(game => {
    totalTime += (game.total_play_time || 0)
    if (game.last_played && game.last_played > lastPlayedTime) {
      lastPlayedTime = game.last_played
      lastPlayedName = game.name
    }
  })

  return {
    totalTime,
    totalGames: props.games.length,
    lastPlayedName
  }
})

const recentHistory = computed(() => {
  const history = []
  props.games.forEach(game => {
    if (game.play_history) {
      game.play_history.forEach(session => {
        history.push({
          gameName: game.name,
          cover: game.cover_image,
          startTime: session.start_time,
          duration: session.duration_seconds,
          deviceId: session.device_id
        })
      })
    }
  })

  return history
    .sort((a, b) => b.startTime - a.startTime)
    .slice(0, 10)
})

const statusDistribution = computed(() => {
  const dist = {
    'Playing': 0,
    'Finished': 0,
    'NotStarted': 0,
    'Shelved': 0
  }
  
  props.games.forEach(game => {
    const status = game.status || 'NotStarted'
    if (dist[status] !== undefined) {
      dist[status]++
    } else {
      dist[status] = (dist[status] || 0) + 1
    }
  })
  
  return dist
})

// Helpers
function formatTotalTime(seconds) {
  const hours = Math.floor(seconds / 3600)
  if (hours > 0) return `${hours} 小时`
  const minutes = Math.floor(seconds / 60)
  return `${minutes} 分钟`
}

function formatMinutes(seconds) {
  const m = Math.ceil(seconds / 60)
  return `${m} 分钟`
}

function formatDate(timestamp) {
  if (!timestamp) return ''
  const date = new Date(timestamp * 1000)
  return `${date.getMonth() + 1}月${date.getDate()}日 ${date.getHours()}:${date.getMinutes().toString().padStart(2, '0')}`
}

function formatStatus(status) {
  const maps = {
    'NotStarted': '未开始',
    'Playing': '正在游玩',
    'Finished': '已通关',
    'Partial': '部分通关',
    'Multiple': '多周目',
    'Shelved': '已搁置'
  }
  return maps[status] || status
}

function getStatusColor(status) {
  const maps = {
    'NotStarted': '#94a3b8',
    'Playing': '#4ecdc4',
    'Finished': '#ff6b6b',
    'Shelved': '#64748b'
  }
  return maps[status] || '#4ecdc4'
}
</script>

<style lang="less" scoped>
.vnite-records {
  width: 100%;
  height: 100%;
}

.records-header {
  margin-bottom: 24px;
  h3 { margin-bottom: 4px; }
  .subtitle { font-size: 13px; color: var(--vnite-text-muted); opacity: 0.8; }
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 20px;
  margin-bottom: 40px;
}

.stat-card {
  background: var(--vnite-card-bg);
  border: 1px solid var(--vnite-border);
  border-radius: 16px;
  padding: 24px;
  display: flex;
  align-items: center;
  gap: 16px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.03);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);

  &:hover {
    transform: translateY(-4px);
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.05);
  }

  .icon-box {
    width: 48px;
    height: 48px;
    border-radius: 12px;
    background: var(--vnite-accent);
    color: var(--vnite-primary);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 24px;
  }

  .value {
    font-size: 20px;
    font-weight: 800;
    color: var(--vnite-text);
  }

  .label {
    font-size: 13px;
    color: var(--vnite-text-muted);
    margin-top: 2px;
  }
}

.records-layout {
  display: grid;
  grid-template-columns: 1.5fr 1fr;
  gap: 32px;
}

.section-title {
  font-size: 16px;
  font-weight: 700;
  margin-bottom: 20px;
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--vnite-text);
}

.history-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.history-item {
  display: flex;
  align-items: center;
  padding: 12px;
  background: var(--vnite-card-bg);
  border: 1px solid var(--vnite-border);
  border-radius: 12px;
  gap: 16px;
  transition: all 0.2s;

  &:hover {
    background: var(--vnite-accent);
    border-color: var(--vnite-primary);
  }

  .cover-wrapper {
    width: 60px;
    height: 80px;
    border-radius: 8px;
    overflow: hidden;
    flex-shrink: 0;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .game-cover {
    width: 100% !important;
    height: 100% !important;
    object-fit: cover;
  }

  .record-info {
    flex: 1;
    min-width: 0;

    .game-name {
      font-size: 15px;
      font-weight: 600;
      color: var(--vnite-text);
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      margin-bottom: 4px;
    }

    .play-details {
      font-size: 12px;
      color: var(--vnite-text-muted);
      display: flex;
      align-items: center;
      gap: 6px;
    }
  }

  .device-tag {
    font-size: 11px;
    color: var(--vnite-text-muted);
    padding: 4px 8px;
    background: rgba(0,0,0,0.05);
    border-radius: 6px;
    display: flex;
    align-items: center;
    gap: 4px;
  }
}

.status-list {
  padding: 20px;
  background: var(--vnite-card-bg);
  border: 1px solid var(--vnite-border);
  border-radius: 16px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.status-item {
  display: flex;
  align-items: center;
  gap: 12px;

  .status-label {
    width: 70px;
    font-size: 13px;
    color: var(--vnite-text);
  }

  .progress-bar {
    flex: 1;
    height: 8px;
    background: rgba(0,0,0,0.05);
    border-radius: 4px;
    overflow: hidden;

    .progress {
      height: 100%;
      border-radius: 4px;
      transition: width 1s ease-out;
    }
  }

  .status-count {
    width: 24px;
    font-size: 13px;
    font-weight: 600;
    text-align: right;
    color: var(--vnite-text);
  }
}

.empty-state {
  text-align: center;
  padding: 40px;
  color: var(--vnite-text-muted);
  font-size: 14px;
  background: var(--vnite-card-bg);
  border-radius: 12px;
  border: 1px dashed var(--vnite-border);
}
</style>
