<template>
  <div class="galgame-manager">
    <!-- 标题栏 -->
    <div class="galgame-header">
      <h2>🎮 游戏存档管理</h2>
      <div class="header-actions">
        <el-button type="primary" @click="scanSavePaths" :loading="scanning">
          <el-icon><Search /></el-icon>
          扫描存档
        </el-button>
        <el-button @click="showAddGameDialog = true">
          <el-icon><Plus /></el-icon>
          添加游戏
        </el-button>
        <el-select v-model="filterStatus" placeholder="状态筛选" style="width: 110px">
          <el-option label="全部" value="All" />
          <el-option label="未开始" value="NotStarted" />
          <el-option label="游玩中" value="Playing" />
          <el-option label="已通关" value="Finished" />
          <el-option label="搁置" value="Shelved" />
        </el-select>

        <el-button @click="syncToCloud" :loading="syncing" :disabled="!cloudEnabled" type="success">
          <el-icon><Upload /></el-icon>
          同步
        </el-button>
      </div>
    </div>

    <!-- 游戏列表 -->
    <el-scrollbar class="games-container">
      <div v-if="games.length === 0" class="empty-state">
        <el-icon :size="80" class="empty-icon"><FolderOpened /></el-icon>
        <h3>尚未添加任何游戏</h3>
        <p>点击"扫描存档"自动检测，或"添加游戏"手动添加</p>
        <el-button type="primary" size="large" @click="scanSavePaths">
          <el-icon><Search /></el-icon>
          开始扫描
        </el-button>
      </div>

      <div v-else class="games-grid">
        <div
          v-for="game in filteredGames"
          :key="game.name"
          class="game-card"
          @click="selectGame(game)"
          :class="{ active: selectedGame?.name === game.name }"
        >
          <div class="game-cover">
            <img v-if="game.cover_image" :src="getCoverUrl(game)" :alt="game.name" />
            <div v-else class="cover-placeholder">
              <el-icon :size="40"><Picture /></el-icon>
              <span>{{ game.name.substring(0, 2) }}</span>
            </div>
            <div class="status-ribbon" v-if="game.status && game.status !== 'NotStarted'">
              <el-tag size="small" effect="dark" :type="getGameStatusType(game.status)">{{ getGameStatusLabel(game.status) }}</el-tag>
            </div>
          </div>
          <div class="game-info">
            <h4 :title="game.name">{{ game.name }}</h4>
            <div class="game-meta">
              <el-tag size="small" :type="getBackupModeType(game.backup_mode)">
                {{ getBackupModeLabel(game.backup_mode) }}
              </el-tag>
              <el-tag size="small" type="info" effect="plain" style="margin-left: 4px" v-if="game.total_play_time > 0">
                ⏱️ {{ formatPlayTime(game.total_play_time) }}
              </el-tag>
            </div>
          </div>
          <div class="game-actions">
            <el-tooltip content="创建快照" placement="top">
              <el-button size="small" circle type="primary" @click.stop="createSnapshot(game)">
                <el-icon><DocumentAdd /></el-icon>
              </el-button>
            </el-tooltip>
            <el-tooltip content="启动游戏" placement="top">
              <el-button size="small" circle type="success" @click.stop="launchGame(game)">
                <el-icon><VideoPlay /></el-icon>
              </el-button>
            </el-tooltip>
            <el-tooltip content="编辑游戏" placement="top">
              <el-button size="small" circle @click.stop="editGame(game)">
                <el-icon><Edit /></el-icon>
              </el-button>
            </el-tooltip>
            <el-tooltip content="删除游戏" placement="top">
              <el-button size="small" circle type="danger" @click.stop="deleteGame(game)">
                <el-icon><Delete /></el-icon>
              </el-button>
            </el-tooltip>

          </div>
        </div>
      </div>
    </el-scrollbar>

    <!-- 快照面板 -->
    <el-drawer
      v-model="showSnapshots"
      :title="selectedGame?.name + ' - 存档快照'"
      direction="rtl"
      size="420px"
    >
      <div v-if="selectedGame" class="drawer-game-overview">
        <div class="overview-content">
          <div class="overview-cover">
            <el-image :src="getCoverUrl(selectedGame)" fit="cover" style="width: 100%; height: 100%">
               <template #error>
                 <div class="image-slot"><el-icon><Picture /></el-icon></div>
               </template>
            </el-image>
          </div>
          <div class="overview-details">
             <div class="overview-tags">
               <el-tag size="small" type="info" v-if="selectedGame.developer">{{ selectedGame.developer }}</el-tag>
               <el-tag size="small" type="info" v-if="selectedGame.release_date">{{ selectedGame.release_date }}</el-tag>
             </div>
             <div class="overview-playtime" v-if="selectedGame.total_play_time > 0">
                <el-icon><VideoPlay /></el-icon> 
                <span>已游玩 {{ formatPlayTime(selectedGame.total_play_time) }}</span>
             </div>
             <div class="overview-desc" v-if="selectedGame.description" :title="selectedGame.description">
               {{ selectedGame.description }}
             </div>
          </div>
        </div>
        <el-divider style="margin: 16px 0" />
      </div>
      <div class="snapshots-header">
        <el-button type="primary" @click="createSnapshot(selectedGame)">
          <el-icon><Plus /></el-icon>
          创建新快照
        </el-button>
        <el-button @click="openBackupFolder">
          <el-icon><FolderOpened /></el-icon>
          目录
        </el-button>
        <el-button type="warning" @click="openScraper(selectedGame)">
          <el-icon><MagicStick /></el-icon>
          元数据
        </el-button>
        <el-button v-if="cloudEnabled" type="danger" plain @click="deleteCloudBackups">
          <el-icon><Delete /></el-icon>
          清空云端
        </el-button>
        <el-dropdown trigger="click" @command="handleStatusChange">
           <el-button class="status-btn" :type="getGameStatusType(selectedGame.status)" plain>
             {{ getGameStatusLabel(selectedGame.status) }}<el-icon class="el-icon--right"><ArrowDown /></el-icon>
           </el-button>
           <template #dropdown>
             <el-dropdown-menu>
               <el-dropdown-item command="NotStarted">未开始</el-dropdown-item>
               <el-dropdown-item command="Playing">游玩中</el-dropdown-item>
               <el-dropdown-item command="Finished">已通关</el-dropdown-item>
               <el-dropdown-item command="Shelved">搁置</el-dropdown-item>
             </el-dropdown-menu>
           </template>
        </el-dropdown>
      </div>
      <div class="snapshots-list">
        <div
          v-for="snapshot in snapshots"
          :key="snapshot.date"
          class="snapshot-item"
        >
          <div class="snapshot-info">
            <div class="snapshot-header">
              <span class="snapshot-date">{{ formatDate(snapshot.date) }}</span>
              <el-tag size="small" v-if="snapshot.device_id">{{ snapshot.device_id.substring(0, 8) }}</el-tag>
            </div>
            <span class="snapshot-desc">{{ snapshot.describe || '无备注' }}</span>
            <span class="snapshot-size">{{ formatSize(snapshot.size) }}</span>
          </div>
          <div class="snapshot-actions">
            <el-button size="small" type="primary" @click="restoreSnapshot(snapshot)">
              <el-icon><RefreshRight /></el-icon>
              恢复
            </el-button>
            <el-button size="small" type="danger" @click="deleteSnapshot(snapshot)">
              <el-icon><Delete /></el-icon>
              删除
            </el-button>
          </div>
        </div>
        <el-empty v-if="snapshots.length === 0" description="暂无快照" />
      </div>
    </el-drawer>

    <!-- 添加/编辑游戏对话框 -->
    <el-dialog v-model="showAddGameDialog" :title="editingGame ? '编辑游戏' : '添加游戏'" width="550px">
      <el-form :model="newGame" label-width="100px" label-position="left">
        <el-form-item label="游戏名称" required>
          <el-input v-model="newGame.name" placeholder="输入游戏名称" />
        </el-form-item>
        <el-form-item label="存档路径" required>
          <el-input v-model="newGame.savePath" placeholder="选择或输入存档目录">
            <template #append>
              <el-button @click="browseSavePath">浏览</el-button>
            </template>
          </el-input>
        </el-form-item>
        <el-form-item label="路径类型">
          <el-radio-group v-model="newGame.pathType">
            <el-radio value="Folder">文件夹</el-radio>
            <el-radio value="File">单个文件</el-radio>
          </el-radio-group>
        </el-form-item>
        <el-form-item label="备份模式">
          <el-select v-model="newGame.backupMode" style="width: 100%">
            <el-option label="手动备份" value="manual" />
            <el-option label="游戏退出时" value="on_game_exit" />
            <el-option label="定时备份" value="scheduled" />
            <el-option label="两者都启用" value="both" />
          </el-select>
        </el-form-item>
        <el-form-item label="备份间隔" v-if="newGame.backupMode === 'scheduled' || newGame.backupMode === 'both'">
          <el-input-number v-model="newGame.backupInterval" :min="5" :max="1440" />
          <span style="margin-left: 10px">分钟</span>
        </el-form-item>
        <el-form-item label="启动路径">
          <el-input v-model="newGame.exePath" placeholder="选择游戏可执行文件（可选）">
            <template #append>
              <el-button @click="browseExePath">浏览</el-button>
            </template>
          </el-input>
        </el-form-item>
        <el-form-item label="游戏封面">
          <el-input v-model="newGame.coverImage" placeholder="封面图片路径（可选）">
            <template #append>
              <el-button @click="browseCoverImage">选择</el-button>
            </template>
          </el-input>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showAddGameDialog = false">取消</el-button>
        <el-button type="primary" @click="addGame">{{ editingGame ? '保存' : '添加' }}</el-button>
      </template>
    </el-dialog>

    <!-- 扫描结果对话框 -->
    <el-dialog v-model="showScanResults" title="扫描结果 - 点击添加" width="650px">
      <el-scrollbar max-height="450px">
        <div class="scan-results">
          <div
            v-for="candidate in scanCandidates"
            :key="candidate.path"
            class="candidate-item"
            @click="quickAddFromScan(candidate)"
          >
            <div class="candidate-info">
              <el-icon :size="24" class="candidate-icon"><Folder /></el-icon>
              <div class="candidate-text">
                <span class="candidate-name">{{ candidate.game_name }}</span>
                <span class="candidate-path">{{ candidate.path }}</span>
              </div>
            </div>
            <div class="candidate-confidence">
              <span>{{ Math.round(candidate.confidence * 100) }}%</span>
              <el-progress
                :percentage="Math.round(candidate.confidence * 100)"
                :color="getConfidenceColor(candidate.confidence)"
                :stroke-width="8"
                :show-text="false"
                style="width: 60px"
              />
            </div>
          </div>
        </div>
      </el-scrollbar>
      <el-empty v-if="scanCandidates.length === 0" description="未找到存档目录" />
    </el-dialog>

    <!-- 云同步设置对话框 -->
    <el-dialog v-model="showCloudSettings" title="云同步设置" width="580px">
      <el-form :model="cloudSettings" label-width="100px" label-position="left">
        <el-form-item label="服务类型">
          <el-select v-model="cloudSettings.type" placeholder="选择云服务" style="width: 100%">
            <el-option label="禁用" value="Disabled" />
            <el-option-group label="WebDAV">
              <el-option label="坚果云" value="jianguoyun" />
              <el-option label="阿里云盘" value="aliyun" />
              <el-option label="自定义 WebDAV" value="WebDAV" />
            </el-option-group>
            <el-option-group label="对象存储">
              <el-option label="Amazon S3 / MinIO" value="S3" />
              <el-option label="阿里云 OSS (原生)" value="AliyunOSS" />
              <el-option label="GitHub" value="GitHub" />
            </el-option-group>
          </el-select>
        </el-form-item>

        <!-- WebDAV 预设 -->
        <template v-if="cloudSettings.type === 'jianguoyun'">
          <el-alert type="info" :closable="false" style="margin-bottom: 16px">
            坚果云 WebDAV 地址：https://dav.jianguoyun.com/dav/
            <br>请使用应用专用密码
          </el-alert>
        </template>
        <template v-if="cloudSettings.type === 'aliyun'">
          <el-alert type="info" :closable="false" style="margin-bottom: 16px">
            阿里云盘需要在 <a href="https://www.aliyundrive.com/webdav" target="_blank">阿里云WebDAV</a> 获取 WebDAV 地址
          </el-alert>
        </template>

        <template v-if="isWebDAVType">
          <el-form-item label="服务器地址">
            <el-input v-model="cloudSettings.endpoint" :placeholder="getWebDAVPlaceholder" />
          </el-form-item>
          <el-form-item label="用户名">
            <el-input v-model="cloudSettings.username" placeholder="WebDAV 用户名" />
          </el-form-item>
          <el-form-item label="密码">
            <el-input v-model="cloudSettings.password" type="password" show-password placeholder="WebDAV 密码或应用专用密码" />
          </el-form-item>
        </template>



        <template v-if="cloudSettings.type === 'S3' || cloudSettings.type === 'AliyunOSS'">
          <el-form-item label="端点地址">
            <el-input v-model="cloudSettings.endpoint" :placeholder="cloudSettings.type === 'S3' ? 'https://s3.amazonaws.com 或 MinIO 地址' : 'https://oss-cn-hangzhou.aliyuncs.com'" />
          </el-form-item>
          <el-form-item label="存储桶">
            <el-input v-model="cloudSettings.bucket" placeholder="bucket-name" />
          </el-form-item>
          <el-form-item label="区域" v-if="cloudSettings.type === 'S3'">
            <el-input v-model="cloudSettings.region" placeholder="us-east-1" />
          </el-form-item>
          <el-form-item label="Access Key">
            <el-input v-model="cloudSettings.accessKeyId" placeholder="Access Key ID" />
          </el-form-item>
          <el-form-item label="Secret Key">
            <el-input v-model="cloudSettings.secretAccessKey" type="password" show-password placeholder="Secret Access Key / Access Key Secret" />
          </el-form-item>
        </template>

        <template v-if="cloudSettings.type === 'GitHub'">
          <el-alert type="info" :closable="false" style="margin-bottom: 16px">
            请使用 GitHub Personal Access Token (Repo Scope)
          </el-alert>
          <el-form-item label="仓库名">
            <el-input v-model="cloudSettings.repo" placeholder="username/repo" />
          </el-form-item>
          <el-form-item label="分支">
            <el-input v-model="cloudSettings.branch" placeholder="master" />
          </el-form-item>
          <el-form-item label="Token">
            <el-input v-model="cloudSettings.token" type="password" show-password placeholder="ghp_..." />
          </el-form-item>
        </template>

        <el-form-item label="同步目录">
          <el-input v-model="cloudSettings.rootPath" placeholder="/galgame-saves" />
        </el-form-item>

        <el-form-item label="自动同步">
          <el-switch v-model="cloudSettings.autoSync" />
          <span style="margin-left: 10px; color: #909399">{{ cloudSettings.autoSync ? '已启用' : '已禁用' }}</span>
        </el-form-item>

        <el-form-item>
          <el-button type="primary" @click="testCloudConnection" :loading="testingCloud">
            <el-icon><Connection /></el-icon>
            测试连接
          </el-button>
          <el-button @click="syncFromCloud" :loading="syncingFrom" :disabled="!cloudEnabled">
            <el-icon><Download /></el-icon>
            从云端拉取
          </el-button>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showCloudSettings = false">取消</el-button>
        <el-button type="primary" @click="saveCloudSettings">保存设置</el-button>
      </template>
    </el-dialog>
    <!-- 搜刮元数据对话框 -->
    <el-dialog v-model="showScraperDialog" title="搜刮元数据 (VNDB)" width="700px">
      <div class="scraper-header">
        <el-input v-model="scraperQuery" placeholder="输入游戏原名或罗马音" @keyup.enter="searchVndb" style="flex: 1">
           <template #append>
             <el-button @click="searchVndb" :loading="scraping">
               <el-icon><Search /></el-icon>
             </el-button>
           </template>
        </el-input>
      </div>
      
      <el-scrollbar height="400px" v-loading="scraping">
        <div class="scraper-results">
          <div 
             v-for="item in scraperResults" 
             :key="item.id" 
             class="scraper-item" 
             @click="selectScraperItem(item)" 
             :class="{ active: selectedScraperItem?.id === item.id }"
          >
             <el-image :src="item.cover_url" fit="cover" class="scraper-cover" lazy>
               <template #error>
                 <div class="image-slot"><el-icon><Picture /></el-icon></div>
               </template>
             </el-image>
             <div class="scraper-info">
               <div class="scraper-title">{{ item.title }}</div>
               <div class="scraper-original">{{ item.original_title }}</div>
               <div class="scraper-meta">
                 <span>{{ item.release_date }}</span>
                 <span v-if="item.developer"> | {{ item.developer }}</span>
               </div>
               <div class="scraper-desc">{{ item.description ? item.description.substring(0, 100) + '...' : '暂无简介' }}</div>
             </div>
          </div>
          <el-empty v-if="!scraping && scraperResults.length === 0" description="未找到结果或请输入关键词" />
        </div>
      </el-scrollbar>

      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showScraperDialog = false">取消</el-button>
          <el-button type="primary" @click="applyMetadata" :disabled="!selectedScraperItem" :loading="applyingMetadata">
            应用元数据
          </el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, onMounted, computed } from 'vue'
import { invoke, convertFileSrc } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  Search, Plus, Upload, FolderOpened, Picture,
  DocumentAdd, Delete, Setting, Edit, RefreshRight,
  Folder, Connection, Download, VideoPlay, MagicStick, ArrowDown
} from '@element-plus/icons-vue'

// State
const games = ref([])
const selectedGame = ref(null)
const snapshots = ref([])
const scanning = ref(false)
const syncing = ref(false)
const syncingFrom = ref(false)
const testingCloud = ref(false)
const showSnapshots = ref(false)
const showAddGameDialog = ref(false)
const showScanResults = ref(false)
const showCloudSettings = ref(false)
const scanCandidates = ref([])
const editingGame = ref(null)
const coverCache = ref({})
const filterStatus = ref('All')

const filteredGames = computed(() => {
  let result = games.value
  if (filterStatus.value !== 'All') {
     // Check for status field, default to NotStarted if missing
     result = result.filter(g => (g.status || 'NotStarted') === filterStatus.value)
  }
  return result
})

const newGame = ref({
  name: '',
  savePath: '',
  pathType: 'Folder',
  backupMode: 'manual',
  backupInterval: 60,
  exePath: '',
  coverImage: ''
})

const cloudSettings = ref({
  type: 'Disabled',
  endpoint: '',
  username: '',
  password: '',
  bucket: '',
  region: '',
  password: '',
  endpoint: '',
  bucket: '',
  region: '',
  accessKeyId: '',
  secretAccessKey: '',
  repo: '',
  branch: 'master',
  token: '',
  rootPath: '/galgame-saves',
  autoSync: false
})

const cloudEnabled = computed(() => cloudSettings.value.type !== 'Disabled')
const isWebDAVType = computed(() => ['WebDAV', 'jianguoyun', 'aliyun'].includes(cloudSettings.value.type))
const getWebDAVPlaceholder = computed(() => {
  if (cloudSettings.value.type === 'jianguoyun') return 'https://dav.jianguoyun.com/dav/'
  if (cloudSettings.value.type === 'aliyun') return 'https://您的WebDAV地址'
  return 'https://your-webdav-server.com/dav/'
})

const loadCover = async (game) => {
  if (!game.cover_image) return
  if (coverCache.value[game.name]) return

  try {
    const dataUrl = await invoke('read_image_as_data_url', { path: game.cover_image })
    coverCache.value[game.name] = dataUrl
  } catch (e) {
    console.error(`Failed to load cover for ${game.name}:`, e)
  }
}

const getCoverUrl = (game) => {
  if (!game.cover_image) return ''
  // Return blob URL if available, otherwise try convertFileSrc as fallback
  return coverCache.value[game.name] || convertFileSrc(game.cover_image)
}

function formatPlayTime(seconds) {
  if (!seconds) return '0分钟'
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  if (h > 0) {
    return `${h}小时${m}分钟`
  }
  return `${m}分钟`
}

// Load games on mount
onMounted(async () => {
  await loadGames()
  await loadCloudSettings()
  
  listen('galgame-auto-backup', (event) => {
    ElMessage.success(event.payload)
  })
  
  listen('galgame-auto-backup-error', (event) => {
    ElMessage.error(event.payload)
  })

  listen('galgame-playtime-update', (event) => {
    const updatedGame = event.payload
    const idx = games.value.findIndex(g => g.name === updatedGame.name)
    if (idx !== -1) {
      games.value[idx] = updatedGame
    }
  })
})

async function loadGames() {
  try {
    const result = await invoke('galgame_list_games')
    games.value = result
    // Load covers for all games
    result.forEach(game => loadCover(game))
  } catch (e) {
    console.error('加载游戏失败:', e)
  }
}

async function loadCloudSettings() {
  try {
    const config = await invoke('galgame_get_config')
    if (config.cloud_settings) {
      const backend = config.cloud_settings.backend || {}
      cloudSettings.value = {
        type: backend.type || 'Disabled',
        endpoint: backend.endpoint || '',
        username: backend.username || '',
        password: backend.password || '',
        bucket: backend.bucket || '',
        region: backend.region || '',
        accessKeyId: backend.access_key_id || '',
        secretAccessKey: backend.secret_access_key || backend.access_key_secret || '',
        repo: (backend.owner && backend.repo) ? `${backend.owner}/${backend.repo}` : '',
        branch: backend.branch || 'master',
        token: backend.token || '',
        rootPath: config.cloud_settings.root_path || '/galgame-saves',
        autoSync: config.cloud_settings.always_sync || false
      }
    }
  } catch (e) {
    console.error('加载云设置失败:', e)
  }
}

const handleStatusChange = async (status) => {
  if (!selectedGame.value) return
  selectedGame.value.status = status
  // Update in games list too
  const game = games.value.find(g => g.name === selectedGame.value.name)
  if (game) game.status = status
  
  // Save config
  try {
    const config = await invoke('galgame_get_config')
    // Update game in config
    const idx = config.games.findIndex(g => g.name === game.name)
    if (idx !== -1) {
       config.games[idx].status = status
       await invoke('galgame_save_config', { config })
       ElMessage.success(`状态已更新: ${getGameStatusLabel(status)}`)
    }
  } catch (e) {
    ElMessage.error('更新状态失败: ' + e)
  }
}

function getBackupModeLabel(mode) {
  const labels = {
    'manual': '手动',
    'on_game_exit': '自动',
    'scheduled': '定时',
    'both': '自动+定时'
  }
  return labels[mode] || '手动'
}

function getBackupModeType(mode) {
  const types = {
    'manual': 'info',
    'on_game_exit': 'success',
    'scheduled': 'warning',
    'both': 'primary'
  }
  return types[mode] || 'info'
}

function getGameStatusLabel(status) {
  const map = {
    'NotStarted': '未开始',
    'Playing': '游玩中',
    'Finished': '已通关',
    'Shelved': '搁置'
  }
  return map[status] || '未开始'
}

function getGameStatusType(status) {
  const map = {
    'NotStarted': 'info',
    'Playing': 'primary',
    'Finished': 'success',
    'Shelved': 'warning'
  }
  return map[status] || 'info'
}

function selectGame(game) {
  selectedGame.value = game
  loadSnapshots(game)
  showSnapshots.value = true
}

async function loadSnapshots(game) {
  try {
    const result = await invoke('galgame_list_snapshots', { gameName: game.name })
    snapshots.value = result
  } catch (e) {
    console.error('加载快照失败:', e)
    snapshots.value = []
  }
}

async function createSnapshot(game) {
  if (!game) return
  
  const { value: describe } = await ElMessageBox.prompt(
    '输入快照备注（可选）',
    '创建快照',
    { inputPlaceholder: '如：通关前、第三章结束' }
  ).catch(() => ({ value: null }))

  if (describe === null) return

  try {
    await invoke('galgame_create_snapshot', {
      gameName: game.name,
      describe: describe || ''
    })
    ElMessage.success('快照创建成功')
    if (selectedGame.value?.name === game.name) {
      await loadSnapshots(game)
    }
  } catch (e) {
    ElMessage.error('创建快照失败: ' + e)
  }
}

async function restoreSnapshot(snapshot) {
  try {
    await ElMessageBox.confirm(
      '恢复此快照将覆盖当前存档，是否继续？',
      '恢复快照',
      { type: 'warning' }
    )
    await invoke('galgame_restore_snapshot', {
      gameName: selectedGame.value.name,
      snapshotDate: snapshot.date
    })
    ElMessage.success('存档恢复成功')
  } catch (e) {
    if (e !== 'cancel') {
      ElMessage.error('恢复失败: ' + e)
    }
  }
}

async function deleteSnapshot(snapshot) {
  try {
    await ElMessageBox.confirm('确定删除此快照？', '确认删除', { type: 'warning' })
    await invoke('galgame_delete_snapshot', {
      gameName: selectedGame.value.name,
      snapshotDate: snapshot.date
    })
    ElMessage.success('快照已删除')
    await loadSnapshots(selectedGame.value)
  } catch (e) {
    if (e !== 'cancel') {
      ElMessage.error('删除失败: ' + e)
    }
  }
}

async function openBackupFolder() {
  try {
    await invoke('galgame_open_backup_folder', { gameName: selectedGame.value.name })
  } catch (e) {
    ElMessage.error('打开目录失败: ' + e)
  }
}

async function launchGame(game) {
  try {
    await invoke('galgame_launch_game', { gameName: game.name })
    ElMessage.success('正在启动游戏...')
  } catch (e) {
    ElMessage.error('启动失败: ' + e)
  }
}

async function deleteCloudBackups() {
  try {
    await ElMessageBox.confirm(
      `确定清空 "${selectedGame.value.name}" 的所有云端备份吗？此操作不可逆。`,
      '清空云端',
      { type: 'warning', confirmButtonText: '确定清空', confirmButtonClass: 'el-button--danger' }
    )
    
    await invoke('galgame_delete_cloud_game', { gameName: selectedGame.value.name })
    ElMessage.success('云端已清空')
  } catch (e) {
    if (e !== 'cancel') {
      ElMessage.error('操作失败: ' + e)
    }
  }
}

async function scanSavePaths() {
  scanning.value = true
  try {
    const result = await invoke('galgame_scan_save_paths')
    scanCandidates.value = result
    showScanResults.value = true
    if (result.length === 0) {
      ElMessage.info('未找到存档目录，请尝试手动添加')
    }
  } catch (e) {
    ElMessage.error('扫描失败: ' + e)
  } finally {
    scanning.value = false
  }
}

async function quickAddFromScan(candidate) {
  newGame.value.name = candidate.game_name
  newGame.value.savePath = candidate.path
  newGame.value.pathType = 'Folder'
  newGame.value.backupMode = 'manual'
  await addGame()
  showScanResults.value = false
}

async function browseSavePath() {
  try {
    const isFolder = newGame.value.pathType === 'Folder'
    const selected = await open({ 
      directory: isFolder,
      multiple: false,
      title: isFolder ? '选择存档目录' : '选择存档文件'
    })
    if (selected) {
      newGame.value.savePath = selected
    }
  } catch (e) {
    console.error('浏览失败:', e)
    ElMessage.error('选择失败: ' + e)
  }
}

async function browseExePath() {
   try {
    const selected = await open({ 
      filters: [{ name: '可执行文件', extensions: ['exe', 'lnk', 'bat', 'cmd'] }],
      title: '选择启动文件'
    })
    if (selected) {
      newGame.value.exePath = selected
    }
  } catch (e) {
    console.error('选择启动文件失败:', e)
  }
}

async function browseCoverImage() {
  try {
    const selected = await open({ 
      filters: [{ name: '图片', extensions: ['png', 'jpg', 'jpeg', 'webp'] }],
      title: '选择游戏封面'
    })
    if (selected) {
      newGame.value.coverImage = selected
    }
  } catch (e) {
    console.error('选择封面失败:', e)
  }
}

function editGame(game) {
  editingGame.value = game
  newGame.value = {
    name: game.name,
    savePath: game.save_paths?.[0]?.paths?.default || '',
    pathType: game.save_paths?.[0]?.unit_type || 'folder',
    backupMode: game.backup_mode || 'manual',
    backupInterval: game.auto_backup_interval || 60,
    exePath: game.exe_path || '',
    coverImage: game.cover_image || ''
  }
  showAddGameDialog.value = true
}

async function addGame() {
  if (!newGame.value.name || !newGame.value.savePath) {
    ElMessage.warning('请填写游戏名称和存档路径')
    return
  }

  try {
    const game = {
      name: newGame.value.name,
      save_paths: [{
        unit_type: newGame.value.pathType.toLowerCase(),
        paths: { default: newGame.value.savePath },
        delete_before_apply: false
      }],
      game_paths: {},
      exe_path: newGame.value.exePath || null,
      backup_mode: newGame.value.backupMode,
      auto_backup_interval: newGame.value.backupInterval,
      cover_image: newGame.value.coverImage || null
    }
    await invoke('galgame_add_game', { game, update: !!editingGame.value })
    ElMessage.success(editingGame.value ? '游戏已更新' : '游戏添加成功')
    showAddGameDialog.value = false
    editingGame.value = null
    newGame.value = { name: '', savePath: '', pathType: 'Folder', backupMode: 'manual', backupInterval: 60, exePath: '', coverImage: '' }
    await loadGames()
  } catch (e) {
    ElMessage.error('操作失败: ' + e)
  }
}

async function deleteGame(game) {
  try {
    await ElMessageBox.confirm(`确定删除游戏 "${game.name}"？\n存档快照也会被删除。`, '确认删除', { type: 'warning' })
    await invoke('galgame_delete_game', { gameName: game.name })
    ElMessage.success('游戏已删除')
    await loadGames()
  } catch (e) {
    if (e !== 'cancel') {
      ElMessage.error('删除失败: ' + e)
    }
  }
}

async function syncToCloud() {
  syncing.value = true
  try {
    const count = await invoke('galgame_sync_to_cloud')
    ElMessage.success(`已同步 ${count} 个文件到云端`)
  } catch (e) {
    ElMessage.error('同步失败: ' + e)
  } finally {
    syncing.value = false
  }
}

async function syncFromCloud() {
  syncingFrom.value = true
  try {
    const count = await invoke('galgame_sync_from_cloud')
    ElMessage.success(`已从云端拉取 ${count} 个文件`)
    await loadGames()
    
    if (selectedGame.value) {
      const updated = games.value.find(g => g.name === selectedGame.value.name)
      if (updated) {
        selectedGame.value = updated
        loadSnapshots(updated)
      }
    }
  } catch (e) {
    ElMessage.error('拉取失败: ' + e)
  } finally {
    syncingFrom.value = false
  }
}

async function testCloudConnection() {
  testingCloud.value = true
  try {
    const backend = buildBackendObject()
    await invoke('galgame_check_cloud_connection', { backend })
    ElMessage.success('连接成功！')
  } catch (e) {
    ElMessage.error('连接失败: ' + e)
  } finally {
    testingCloud.value = false
  }
}

function buildBackendObject() {
  const type = cloudSettings.value.type
  if (type === 'Disabled') return { type: 'Disabled' }
  
  if (['WebDAV', 'jianguoyun', 'aliyun'].includes(type)) {
    let endpoint = cloudSettings.value.endpoint
    if (type === 'jianguoyun' && !endpoint) {
      endpoint = 'https://dav.jianguoyun.com/dav/'
    }
    return {
      type: 'WebDAV',
      endpoint,
      username: cloudSettings.value.username,
      password: cloudSettings.value.password
    }
  }
  
  if (type === 'S3' || type === 'AliyunOSS') {
    let endpoint = (cloudSettings.value.endpoint || '').trim()
    const bucket = (cloudSettings.value.bucket || '').trim()
    const region = (cloudSettings.value.region || '').trim()
    const ak = (cloudSettings.value.accessKeyId || '').trim()
    const sk = (cloudSettings.value.secretAccessKey || '').trim()
    
    // 自动修复 Endpoint (移除误填的 Bucket 前缀)
    if (bucket && endpoint.includes(bucket + '.')) {
      endpoint = endpoint.replace(bucket + '.', '')
    }

    if (type === 'AliyunOSS') {
      return {
        type: 'AliyunOSS',
        endpoint: endpoint,
        bucket: bucket,
        access_key_id: ak,
        access_key_secret: sk
      }
    } else {
      return {
        type: 'S3',
        endpoint: endpoint,
        bucket: bucket,
        region: region,
        access_key_id: ak,
        secret_access_key: sk
      }
    }
  }

  if (type === 'GitHub') {
    const [owner, repo] = (cloudSettings.value.repo || '').trim().split('/')
    return {
      type: 'GitHub',
      owner: (owner || '').trim(),
      repo: (repo || '').trim(),
      branch: (cloudSettings.value.branch || 'master').trim(),
      token: (cloudSettings.value.token || '').trim()
    }
  }
  
  return { type: 'Disabled' }
}

async function saveCloudSettings() {
  try {
    const config = await invoke('galgame_get_config')
    config.cloud_settings = {
      always_sync: cloudSettings.value.autoSync,
      root_path: cloudSettings.value.rootPath,
      backend: buildBackendObject()
    }
    await invoke('galgame_save_config', { config })
    ElMessage.success('云同步设置已保存')
    showCloudSettings.value = false
  } catch (e) {
    ElMessage.error('保存失败: ' + e)
  }
}

function formatDate(dateStr) {
  if (!dateStr) return ''
  // Format: 2024-01-15_14-30-00 or similar
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

function getConfidenceColor(confidence) {
  if (confidence >= 0.7) return '#67c23a'
  if (confidence >= 0.5) return '#e6a23c'
  return '#909399'
}

const showScraperDialog = ref(false)
const scraperQuery = ref('')
const scraperResults = ref([])
const selectedScraperItem = ref(null)
const scraping = ref(false)
const applyingMetadata = ref(false)
const scraperTargetGame = ref(null)

const openScraper = (game) => {
  scraperTargetGame.value = game
  scraperQuery.value = game.name
  scraperResults.value = []
  selectedScraperItem.value = null
  showScraperDialog.value = true
}

const searchVndb = async () => {
  if (!scraperQuery.value) return
  if (!navigator.onLine) {
    ElMessage.error('无网络连接，无法访问 VNDB')
    return
  }
  
  scraping.value = true
  try {
    const res = await invoke('galgame_search_metadata', { keyword: scraperQuery.value })
    scraperResults.value = res
  } catch (e) {
    ElMessage.error('搜索失败: ' + e)
  } finally {
    scraping.value = false
  }
}

const selectScraperItem = (item) => {
  selectedScraperItem.value = item
}

const applyMetadata = async () => {
  if (!selectedScraperItem.value || !scraperTargetGame.value) return
  
  applyingMetadata.value = true
  try {
    const targetName = scraperTargetGame.value.name
    await invoke('galgame_apply_metadata', { 
       gameName: targetName,
       data: selectedScraperItem.value
    })
    ElMessage.success('元数据已更新')
    showScraperDialog.value = false
    await loadGames()
    
    // Refresh selectedGame references to show new data immediately
    if (selectedGame.value && selectedGame.value.name === targetName) {
       const updated = games.value.find(g => g.name === targetName)
       if (updated) {
         selectedGame.value = updated
       }
    }
  } catch (e) {
    ElMessage.error('应用失败: ' + e)
  } finally {
    applyingMetadata.value = false
  }
}

defineExpose({ showCloudSettings })
</script>

<style scoped lang="less">
.galgame-manager {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 24px;
  background: var(--el-bg-color);
}

.galgame-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--el-border-color-lighter);

  h2 {
    margin: 0;
    font-size: 22px;
    color: var(--el-text-color-primary);
  }

  .header-actions {
    display: flex;
    gap: 12px;
  }
}

.games-container {
  flex: 1;
  overflow: hidden;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 400px;
  color: var(--el-text-color-secondary);

  .empty-icon {
    opacity: 0.5;
    margin-bottom: 16px;
  }

  h3 {
    margin: 0 0 8px 0;
    font-size: 18px;
    color: var(--el-text-color-primary);
  }

  p {
    margin: 0 0 24px 0;
    font-size: 14px;
  }
}

.games-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 20px;
  padding: 4px;
}

.game-card {
  background: var(--el-bg-color-overlay);
  border-radius: 16px;
  padding: 16px;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  border: 2px solid transparent;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);

  &:hover {
    transform: translateY(-4px);
    box-shadow: 0 12px 28px rgba(0, 0, 0, 0.12);
  }

  &.active {
    border-color: var(--el-color-primary);
  }

  .game-cover {
    width: 100%;
    aspect-ratio: 16/10;
    background: linear-gradient(135deg, var(--el-fill-color) 0%, var(--el-fill-color-light) 100%);
    border-radius: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    margin-bottom: 14px;
    position: relative;

    img {
      width: 100%;
      height: 100%;
      object-fit: cover;
    }

    .status-ribbon {
      position: absolute;
      top: 6px;
      right: 6px;
      z-index: 2;
      box-shadow: 0 2px 4px rgba(0,0,0,0.2);
    }


    .cover-placeholder {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 8px;
      color: var(--el-text-color-placeholder);
      
      span {
        font-size: 24px;
        font-weight: 600;
        text-transform: uppercase;
      }
    }
  }

  .game-info {
    h4 {
      margin: 0 0 8px 0;
      font-size: 15px;
      font-weight: 600;
      color: var(--el-text-color-primary);
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }

    .game-meta {
      display: flex;
      gap: 6px;
    }
  }

  .game-actions {
    display: flex;
    gap: 8px;
    margin-top: 14px;
    justify-content: flex-end;
  }
}

.snapshots-header {
  display: flex;
  gap: 8px;
  margin-bottom: 20px;
  flex-wrap: wrap;
  
  .el-button {
    margin-left: 0 !important;
  }
}

.snapshots-list {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.snapshot-item {
  padding: 16px;
  background: var(--el-fill-color-light);
  border-radius: 12px;

  .snapshot-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
  }

  .snapshot-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 12px;

    .snapshot-date {
      font-weight: 600;
      color: var(--el-text-color-primary);
    }

    .snapshot-desc {
      font-size: 13px;
      color: var(--el-text-color-secondary);
    }

    .snapshot-size {
      font-size: 12px;
      color: var(--el-text-color-placeholder);
    }
  }

  .snapshot-actions {
    display: flex;
    gap: 10px;
  }
}

.scan-results {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.candidate-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 16px;
  background: var(--el-fill-color-light);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s;

  &:hover {
    background: var(--el-color-primary-light-9);
    transform: translateX(4px);
  }

  .candidate-info {
    display: flex;
    align-items: center;
    gap: 12px;
    flex: 1;
    overflow: hidden;

    .candidate-icon {
      color: var(--el-color-primary);
    }

    .candidate-text {
      display: flex;
      flex-direction: column;
      flex: 1;
      overflow: hidden;
    }

    .candidate-name {
      font-weight: 600;
      color: var(--el-text-color-primary);
    }

    .candidate-path {
      font-size: 12px;
      color: var(--el-text-color-secondary);
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
  }

  .candidate-confidence {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--el-text-color-secondary);
    font-size: 13px;
  }
}

.drawer-game-overview {
  margin-bottom: 20px;
  
  .overview-content {
    display: flex;
    gap: 16px;
    
    .overview-cover {
      width: 100px;
      height: 140px;
      flex-shrink: 0;
      border-radius: 8px;
      overflow: hidden;
      background: var(--el-fill-color-light);
      box-shadow: 0 4px 12px rgba(0,0,0,0.1);
      
      .image-slot {
        display: flex;
        justify-content: center;
        align-items: center;
        width: 100%;
        height: 100%;
        color: var(--el-text-color-secondary);
        font-size: 24px;
      }
    }
    
    .overview-details {
      flex: 1;
      display: flex;
      flex-direction: column;
      overflow: hidden;
      
      .overview-tags {
        display: flex;
        flex-wrap: wrap;
        gap: 6px;
        margin-bottom: 8px;
      }
      
      .overview-playtime {
        display: flex;
        align-items: center;
        gap: 6px;
        font-size: 13px;
        color: var(--el-color-primary);
        margin-bottom: 8px;
        font-weight: 500;
      }
      
      .overview-desc {
        font-size: 12px;
        color: var(--el-text-color-regular);
        line-height: 1.5;
        display: -webkit-box;
        -webkit-line-clamp: 4;
        -webkit-box-orient: vertical;
        overflow: hidden;
        text-overflow: ellipsis;
      }
    }
  }
}

.scraper-header {
  margin-bottom: 20px;
  display: flex;
}
.scraper-results {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.scraper-item {
  display: flex;
  gap: 12px;
  padding: 10px;
  border-radius: 8px;
  background: var(--el-fill-color-light);
  cursor: pointer;
  border: 2px solid transparent;
  transition: all 0.2s;
  
  &:hover {
    background: var(--el-fill-color);
  }
  &.active {
    border-color: var(--el-color-primary);
    background: var(--el-color-primary-light-9);
  }
  
  .scraper-cover {
    width: 60px;
    min-width: 60px;
    height: 85px;
    border-radius: 4px;
    background: var(--el-fill-color-dark);
    display: flex;
    justify-content: center;
    align-items: center;
  }
  .scraper-info {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    
    .scraper-title {
      font-weight: bold;
      margin-bottom: 4px;
      font-size: 15px;
    }
    .scraper-original {
      font-size: 12px;
      color: var(--el-text-color-secondary);
      margin-bottom: 4px;
    }
    .scraper-meta {
      font-size: 12px;
      color: var(--el-text-color-placeholder);
      margin-bottom: 6px;
    }
    .scraper-desc {
      font-size: 12px;
      color: var(--el-text-color-regular);
      line-height: 1.4;
      display: -webkit-box;
      -webkit-line-clamp: 2;
      -webkit-box-orient: vertical;
      overflow: hidden;
    }
  }
}
</style>
