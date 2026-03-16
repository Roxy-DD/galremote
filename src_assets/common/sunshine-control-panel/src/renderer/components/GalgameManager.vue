<template>
  <div class="galgame-manager vnite-theme">
    <div class="vnite-layout">

      <!-- 2. 中间索引列表栏 -->
      <VniteLibraryBar 
        :games="games" 
        :collections="collections"
        :selectedGame="selectedGame"
        @select-game="selectGame"
        @update-collections="loadCollections"
        @launch-game="launchGame"
        @edit-game="editGame"
        @open-scraper="openScraper"
        @delete-game="deleteGame"
      />

      <!-- 3. 右侧画廊 / 详情栏 -->
      <div class="vnite-main-view">
        <template v-if="activeMenu === 'library'">
          <VniteGallery 
            v-if="!selectedGame" 
            :games="games"
            :scanning="scanning"
            :settings="galgameSettings"
            :collections="collections"
            @select-game="selectGame"
            @scan="scanSavePaths"
            @launch-game="launchGame"
            @create-snapshot="createSnapshot"
            @edit-game="editGame"
            @open-scraper="openScraper"
            @delete-game="deleteGame"
            @update-collections="loadCollections"
          />
          
          <VniteGameDetail 
            v-else 
            :game="selectedGame" 
            :snapshots="snapshots"
            :cloud-enabled="cloudEnabled"
            :settings="galgameSettings"
            :collections="collections"
            @close="selectedGame = null"
            @create-snapshot="createSnapshot"
            @restore-snapshot="restoreSnapshot"
            @delete-snapshot="deleteSnapshot"
            @open-backup="openBackupFolder"
            @open-scraper="openScraper"
            @delete-cloud-backups="deleteCloudBackups"
            @change-status="handleStatusChange"
            @launch-game="launchGame"
            @update-collections="loadCollections"
          />
        </template>

        <div v-else-if="activeMenu === 'records'" class="vnite-detail">
          <VniteRecords :games="games" />
        </div>
      </div>
    </div>

    <!-- 添加/编辑游戏对话框 -->
    <VniteAddGame 
      v-model="showAddGameDialog" 
      :editing-game="editingGame"
      @saved="loadGames"
    />

    <!-- 扫描结果对话框 -->
    <VniteScanResults 
      v-model="showScanResults"
      :candidates="scanCandidates"
      @quick-add="handleQuickAddFromScan"
      @batch-add="handleBatchAddFromScan"
    />

    <!-- 统一设置对话框 -->
    <VniteSettings 
      v-model="showSettings"
      :settings="galgameSettings"
      :cloud-settings="cloudSettings"
      @saved="loadCloudSettings"
      @sync-from-cloud="syncFromCloud"
      @prune-config="pruneConfig"
    />

    <!-- 启动扫描确认弹窗 -->
    <el-dialog
      v-model="showScanDialog"
      title="自动扫描游戏"
      width="420px"
      class="vnite-scan-starter-dialog"
      append-to-body
    >
      <div class="scan-starter-body">
        <div class="gura-hint">
          <p>正在搜寻您的游戏宝藏...</p>
          <p class="desc">我们将为您自动查找本地已安装的游戏存档与目录。</p>
        </div>
        <el-button 
          type="primary" 
          class="start-btn" 
          :loading="scanning" 
          @click="scanSavePaths"
        >
          开始全盘扫描
        </el-button>
      </div>
    </el-dialog>

    <!-- 云同步冲突解决对话框 -->
    <el-dialog v-model="showConflictDialog" title="☁️ 云同步冲突检测" width="550px" :close-on-click-modal="false" :show-close="false">
      <div class="conflict-dialog-body" v-if="conflictGame">
        <el-alert
          type="warning"
          show-icon
          :closable="false"
          style="margin-bottom: 20px;"
        >
          <template #title>
             发现存档冲突: <b>{{ conflictGame.name }}</b>
          </template>
          <template #default>
            <p style="margin: 0;">两端存档发生了离散变动。您的本地数据与网络可能产生了并行变更，请选择你要保留哪一方的数据。</p>
            <p style="margin: 4px 0 0; font-size: 12px;">（如果选择云端，您的本地现有数据将会被保存至安全目录下 <code>.bak_saves</code>）</p>
          </template>
        </el-alert>

        
        <div class="conflict-actions" style="display: flex; justify-content: space-between; align-items: center;">
           <el-button class="conflict-btn local-btn" @click="resolveConflict('local')" type="primary" plain style="flex:1; height: auto; padding: 15px; text-align: left;">
              <div class="btn-content" style="display: flex; gap: 10px; align-items: center;">
                 <el-icon :size="32"><Upload /></el-icon>
                 <div class="btn-text">
                   <div style="font-weight: bold; font-size: 16px;">保留本地上传到云端</div>
                   <div style="font-size: 12px; color: #666; margin-top: 4px; white-space: normal; line-height: 1.4;">覆盖云端记录。</div>
                 </div>
              </div>
           </el-button>

           <div style="margin: 0 15px; color: #999;">OR</div>

           <el-button class="conflict-btn cloud-btn" @click="resolveConflict('cloud')" type="success" plain style="flex:1; height: auto; padding: 15px; text-align: left;">
              <div class="btn-content" style="display: flex; gap: 10px; align-items: center;">
                 <el-icon :size="32"><Download /></el-icon>
                 <div class="btn-text">
                   <div style="font-weight: bold; font-size: 16px;">下载覆盖这台机子</div>
                   <div style="font-size: 12px; color: #666; margin-top: 4px; white-space: normal; line-height: 1.4;">使用上次同步的云进度。</div>
                 </div>
              </div>
           </el-button>
        </div>
      </div>
      <template #footer>
        <el-button @click="closeConflictDialog">稍后处理</el-button>
      </template>
    </el-dialog>

    <!-- 搜刮元数据对话框 -->
    <VniteScraper 
      v-model="showScraperDialog"
      :target-game="scraperTargetGame"
      @applied="handleMetadataApplied"
    />
  </div>
</template>

<script setup>
import { ref, watch, onMounted, onUnmounted, computed, defineAsyncComponent } from 'vue'
import { invoke, convertFileSrc } from '@tauri-apps/api/core'
import VniteLibraryBar from './vnite/VniteLibraryBar.vue'
import VniteGallery from './vnite/VniteGallery.vue'
import VniteScraper from './vnite/VniteScraper.vue'
import VniteAddGame from './vnite/VniteAddGame.vue'
import VniteSettings from './vnite/VniteSettings.vue'
import VniteScanResults from './vnite/VniteScanResults.vue'
import VniteGameDetail from './vnite/VniteGameDetail.vue'
import VniteRecords from './vnite/VniteRecords.vue'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  Search, Plus, Upload, FolderOpened, Picture,
  DocumentAdd, Delete, Setting, Edit, RefreshRight,
  Folder, Connection, Download, VideoPlay, MagicStick, ArrowDown,
  CopyDocument, Loading
} from '@element-plus/icons-vue'

// State
const games = ref([])
const collections = ref([])
const selectedGame = ref(null)
const snapshots = ref([])
const scanning = ref(false)
const syncing = ref(false)
const syncingFrom = ref(false)
const testingCloud = ref(false)
const showAddGameDialog = ref(false)
const showScanDialog = ref(false)
const showScanResults = ref(false)
const showSettings = ref(false)
const showScraperDialog = ref(false)
const showConflictDialog = ref(false)
const conflictGame = ref(null)
const conflictDirection = ref('')
const editingGame = ref(null)
const editingOriginalName = ref(null)
const scraperTargetGame = ref(null)
const activeMenu = ref('library')
const scanCandidates = ref([])
const galgameSettings = ref({
  nsfw_blur: true,
  nsfw_blur_intensity: 20,
  theme: 'dark' // 'dark' or 'light'
})

const cloudSettings = ref({
  type: 'Disabled',
  endpoint: '',
  username: '',
  password: '',
  bucket: '',
  region: '',
  accessKeyId: '',
  secretAccessKey: '',
  repo: '',
  branch: 'main',
  token: '',
  rootPath: '/galgame-saves',
  autoSync: false
})

onMounted(async () => {
  await loadCloudSettings()
  await loadGames()
  await loadCollections()
})

const cloudEnabled = computed(() => cloudSettings.value.type !== 'Disabled')


const loadGames = async () => {
  try {
    const result = await invoke('galgame_list_games')
    games.value = result

    if (selectedGame.value) {
      const refreshed = result.find(g => g.name === selectedGame.value.name)
      if (refreshed) {
        selectedGame.value = refreshed
      } else {
        selectedGame.value = null
        snapshots.value = []
      }
    }
  } catch (e) {
    console.error('加载游戏失败:', e)
  }
}

const loadCollections = async () => {
  try {
    const result = await invoke('galgame_list_collections')
    collections.value = result
  } catch (e) {
    console.error('加载收藏夹失败:', e)
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
        branch: backend.branch || 'main',
        token: backend.token || '',
        rootPath: config.cloud_settings.root_path || '/galgame-saves',
        autoSync: config.cloud_settings.always_sync || false
      }
    }
    if (config.collections?.collections) {
      collections.value = config.collections.collections
    }
    if (config.settings) {
      galgameSettings.value = { ...galgameSettings.value, ...config.settings }
    }
  } catch (e) {
    console.error('加载云设置失败:', e)
  }
}

const handleStatusChange = async (status) => {
  if (!selectedGame.value) return
  selectedGame.value.status = status
  const game = games.value.find(g => g.name === selectedGame.value.name)
  if (game) game.status = status
  
  try {
    const config = await invoke('galgame_get_config')
    const idx = config.games.findIndex(g => g.name === game.name)
    if (idx !== -1) {
       config.games[idx].status = status
       await invoke('galgame_save_config', { config })
       ElMessage.success(`状态已更新`)
    }
  } catch (e) {
    ElMessage.error('更新状态失败: ' + e)
  }
}

function selectGame(game) {
  selectedGame.value = game
  loadSnapshots(game)
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
  const { value: describe } = await ElMessageBox.prompt('输入备注', '创建快照').catch(() => ({ value: null }))
  if (describe === null) return
  try {
    await invoke('galgame_create_snapshot', { gameName: game.name, describe: describe || '' })
    ElMessage.success('快照创建成功')
    if (selectedGame.value?.name === game.name) await loadSnapshots(game)
  } catch (e) {
    ElMessage.error('创建失败: ' + e)
  }
}

async function restoreSnapshot(snapshot) {
  try {
    await ElMessageBox.confirm('是否覆盖当前存档？', '恢复快照', { type: 'warning' })
    await invoke('galgame_restore_snapshot', { gameName: selectedGame.value.name, snapshotDate: snapshot.date })
    ElMessage.success('恢复成功')
  } catch (e) {
    if (e !== 'cancel') ElMessage.error('失败: ' + e)
  }
}

async function deleteSnapshot(snapshot) {
  try {
    await ElMessageBox.confirm('确定删除？', '确认删除', { type: 'warning' })
    await invoke('galgame_delete_snapshot', { gameName: selectedGame.value.name, snapshotDate: snapshot.date })
    ElMessage.success('已删除')
    await loadSnapshots(selectedGame.value)
  } catch (e) {
    if (e !== 'cancel') ElMessage.error('失败: ' + e)
  }
}

async function openBackupFolder() {
  try {
    await invoke('galgame_open_backup_folder', { gameName: selectedGame.value.name })
  } catch (e) {
    ElMessage.error('打开失败: ' + e)
  }
}

async function launchGame(game) {
  try {
    await invoke('galgame_launch_game', { gameName: game.name })
    ElMessage.success('正在启动...')
  } catch (e) {
    ElMessage.error('启动失败: ' + e)
  }
}

async function deleteCloudBackups() {
  try {
    await ElMessageBox.confirm('清空云端备份？', '确认', { type: 'warning' })
    await invoke('galgame_delete_cloud_game', { gameName: selectedGame.value.name })
    ElMessage.success('云端已清空')
  } catch (e) {
    if (e !== 'cancel') ElMessage.error('失败: ' + e)
  }
}

const handleQuickAddFromScan = async (candidate) => {
  try {
    const game = {
      name: candidate.game_name,
      save_paths: [{ unit_type: 'folder', paths: { default: candidate.path }, delete_before_apply: false }],
      backup_mode: 'manual', auto_backup_interval: 60,
    }
    await invoke('galgame_add_game', { game, update: false, oldName: null })
    ElMessage.success('添加成功')
    await loadGames()
    showScanResults.value = false
  } catch (e) {
    ElMessage.error('失败: ' + e)
  }
}

const handleBatchAddFromScan = async (selectedCandidates) => {
  try {
    const gamesToAdd = selectedCandidates.map(candidate => ({
      name: candidate.game_name,
      game_paths: { default: candidate.path },
      save_paths: [{ unit_type: 'folder', paths: { default: candidate.path }, delete_before_apply: false }],
      backup_mode: 'manual',
      auto_backup_interval: 60,
      nsfw: false
    }))
    
    const count = await invoke('galgame_batch_add_games', { games: gamesToAdd })
    ElMessage.success(`成功批量添加 ${count} 个游戏`)
    await loadGames()
    showScanResults.value = false
  } catch (e) {
    ElMessage.error('批量添加失败: ' + e)
  }
}

function openAddGameDialog() {
  editingGame.value = null
  showAddGameDialog.value = true
}

function editGame(game) {
  editingGame.value = game
  showAddGameDialog.value = true
}

async function scanSavePaths() {
  scanning.value = true
  try {
    const results = await invoke('galgame_scan_save_paths')
    scanCandidates.value = results
    showScanDialog.value = false // 扫描开始或结束后关闭启动窗
    showScanResults.value = true
  } catch (e) {
    ElMessage.error('扫描失败: ' + e)
  } finally {
    scanning.value = false
  }
}

async function pruneConfig() {
  try {
    const prunedCount = await invoke('galgame_prune_config')
    if (prunedCount > 0) {
      ElMessage.success(`成功清理了 ${prunedCount} 条无效记录`)
      await loadGames()
    } else {
      ElMessage.info('未发现需要清理的记录')
    }
  } catch (e) {
    ElMessage.error('清理失败: ' + e)
  }
}

function openScraper(game) {
  scraperTargetGame.value = game
  showScraperDialog.value = true
}

function openScanDialog() {
  showScanDialog.value = true
}

const setActiveView = (view) => {
  activeMenu.value = view
  if (view === 'library') {
    selectedGame.value = null
  }
}

async function deleteGame(game) {
  try {
    await ElMessageBox.confirm(`确定删除 "${game.name}" 吗？此操作不可撤销。`, '确认删除', {
      confirmButtonText: '确定删除',
      cancelButtonText: '取消',
      type: 'warning'
    })
    await invoke('galgame_delete_game', { gameName: game.name })
    ElMessage.success('删除成功')
    await loadGames()
  } catch (e) {
    if (e !== 'cancel') {
      ElMessage.error('删除失败: ' + e)
    }
  }
}

async function syncFromCloud() {
  if (syncingFrom.value) return
  syncingFrom.value = true
  try {
    await invoke('galgame_sync_from_cloud')
    ElMessage.success('云端同步完成')
    await loadGames()
    await loadCollections()
  } catch (e) {
    ElMessage.error('同步失败: ' + e)
  } finally {
    syncingFrom.value = false
  }
}

function handleMetadataApplied() {
  loadGames()
}

defineExpose({
  showSettings,
  openAddGameDialog,
  openScanDialog,
  setActiveView
})
</script>

<style>
/* Vnite Layout & Theme integration with global theme.less variables */
.galgame-manager {
  height: 100%;
  width: 100%;
}

.vnite-layout {
  display: flex;
  height: 100%;
  width: 100%;
  overflow: hidden;
  background: transparent;
  color: var(--vnite-text);
  transition: background 0.3s, color 0.3s;
  backdrop-filter: blur(20px);
}

.vnite-main-view {
  flex: 1;
  height: 100%;
  overflow: hidden;
  position: relative;
}

.vnite-detail {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--vnite-bg);
  overflow-y: auto;
}

.detail-header-banner {
  height: 240px;
  background-size: cover;
  background-position: center 20%;
  position: relative;
  flex-shrink: 0;
  
  &::after {
    content: '';
    position: absolute;
    inset: 0;
    background: linear-gradient(to bottom, transparent, var(--vnite-bg));
  }
}

.banner-overlay {
  position: absolute;
  top: 16px;
  left: 20px;
  z-index: 10;
  
  .back-btn {
    background: rgba(0, 0, 0, 0.4);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: #fff;
    &:hover {
      background: rgba(255, 255, 255, 0.1);
    }
  }
}

.gm-detail-content {
  padding: 0 48px 48px;
  margin-top: -80px;
  position: relative;
  z-index: 5;
}

.gm-detail-header {
  display: flex;
  gap: 32px;
  align-items: flex-end;
  margin-bottom: 40px;
}

.gm-detail-cover {
  width: 200px;
  aspect-ratio: 2/3;
  background-size: cover;
  background-position: center;
  border-radius: 12px;
  box-shadow: 0 15px 40px rgba(0,0,0,0.6);
  background-color: #222;
  flex-shrink: 0;
}

.gm-detail-title-caption {
  flex: 1;
  padding-bottom: 8px;
  
  h1 {
    margin: 0 0 16px;
    font-size: 36px;
    font-weight: 800;
    line-height: 1.2;
    text-shadow: 0 2px 10px rgba(0,0,0,0.5);
  }
}

.gm-detail-meta-row {
  display: flex;
  align-items: center;
  gap: 16px;
  color: rgba(255, 255, 255, 0.5);
  font-size: 15px;
  
  .meta-separator {
    opacity: 0.3;
  }
}

.gm-detail-main-actions {
  display: flex;
  gap: 12px;
  padding-bottom: 8px;
}

.gm-detail-section {
  margin-top: 32px;
  
  h3 {
    font-size: 18px;
    margin-bottom: 16px;
    color: rgba(255,255,255,0.9);
  }
}

.description-text {
  font-size: 15px;
  line-height: 1.8;
  color: rgba(255,255,255,0.7);
  white-space: pre-wrap;
}

.snapshot-toolbar {
  margin-bottom: 20px;
  display: flex;
  gap: 12px;
}

/* Scrollbar styling for vnite theme - uses global attribute selector */
.vnite-layout ::-webkit-scrollbar {
  width: 6px;
}
.vnite-layout ::-webkit-scrollbar-track {
  background: transparent;
}
.vnite-layout ::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 3px;
}
.vnite-layout ::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.2);
}
.vnite-scan-starter-dialog {
  .scan-starter-body {
    padding: 10px 0 20px;
    text-align: center;
    
    .gura-hint {
      margin-bottom: 32px;
      p {
        font-size: 16px;
        font-weight: 700;
        color: var(--vnite-primary);
        margin-bottom: 8px;
      }
      .desc {
        font-size: 13px;
        font-weight: 400;
        color: var(--vnite-text-muted);
        opacity: 0.8;
      }
    }
    
    .start-btn {
      width: 200px;
      height: 48px;
      font-size: 16px;
      font-weight: 700;
      border-radius: 12px;
    }
  }
}
</style>


