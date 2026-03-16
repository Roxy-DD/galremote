<script setup>
import { ref, watch, computed } from 'vue'
import { 
  Setting, 
  View, 
  Cloudy, 
  Clock, 
  Delete,
  InfoFilled,
  Monitor,
  Connection,
  Download,
  Loading
} from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox } from 'element-plus'

const props = defineProps({
  modelValue: Boolean,
  settings: Object,
  cloudSettings: Object
})

const emit = defineEmits(['update:modelValue', 'saved', 'sync-from-cloud', 'prune-config'])

const activeTab = ref('general')
const localSettings = ref({ ...props.settings })
const localCloud = ref({ ...props.cloudSettings })

// Internal states for GitHub OAuth
const githubLoading = ref(false)
const githubDeviceCode = ref('')
const githubVerificationUri = ref('')
const testingCloud = ref(false)

watch(() => props.modelValue, (val) => {
  if (val) {
    localSettings.value = JSON.parse(JSON.stringify(props.settings))
    localCloud.value = JSON.parse(JSON.stringify(props.cloudSettings))
  }
})

const isWebDAVType = computed(() => ['WebDAV', 'jianguoyun', 'aliyun'].includes(localCloud.value.type))
const cloudEnabled = computed(() => localCloud.value.type !== 'Disabled')

const getWebDAVPlaceholder = computed(() => {
  if (localCloud.value.type === 'jianguoyun') return 'https://dav.jianguoyun.com/dav/'
  if (localCloud.value.type === 'aliyun') return 'https://您的WebDAV地址'
  return 'https://your-webdav-server.com/dav/'
})

const buildBackendObject = () => {
  const type = localCloud.value.type
  if (type === 'Disabled') return { type: 'Disabled' }
  
  if (['WebDAV', 'jianguoyun', 'aliyun'].includes(type)) {
    return {
      type: 'WebDAV',
      endpoint: localCloud.value.endpoint || (type === 'jianguoyun' ? 'https://dav.jianguoyun.com/dav/' : ''),
      username: localCloud.value.username,
      password: localCloud.value.password
    }
  }
  
  if (type === 'S3' || type === 'AliyunOSS') {
    if (type === 'AliyunOSS') {
      return {
        type: 'AliyunOSS',
        endpoint: localCloud.value.endpoint,
        bucket: localCloud.value.bucket,
        access_key_id: localCloud.value.accessKeyId,
        access_key_secret: localCloud.value.secretAccessKey
      }
    } else {
      return {
        type: 'S3',
        endpoint: localCloud.value.endpoint,
        bucket: localCloud.value.bucket,
        region: localCloud.value.region,
        access_key_id: localCloud.value.accessKeyId,
        secret_access_key: localCloud.value.secretAccessKey
      }
    }
  }

  if (type === 'GitHub') {
    const [owner, repo] = (localCloud.value.repo || '').split('/')
    return {
      type: 'GitHub',
      owner: (owner || '').trim(),
      repo: (repo || '').trim(),
      branch: (localCloud.value.branch || 'main').trim(),
      token: (localCloud.value.token || '').trim()
    }
  }
  
  return { type: 'Disabled' }
}

const handleTestConnection = async () => {
  testingCloud.value = true
  try {
    const backend = buildBackendObject()
    await invoke('galgame_check_cloud_connection', { backend })
    ElMessage.success('云端连接成功！')
  } catch (e) {
    ElMessage.error('云端连接失败: ' + e)
  } finally {
    testingCloud.value = false
  }
}

const handleStartGithubOauth = async () => {
  if (githubLoading.value) return
  githubLoading.value = true
  try {
    const res = await invoke('galgame_github_oauth_request')
    githubDeviceCode.value = res.user_code
    githubVerificationUri.value = res.verification_uri
    await invoke('open_external_url', { url: res.verification_uri })
    
    const token = await invoke('galgame_github_oauth_poll', {
        deviceCode: res.device_code,
        interval: res.interval,
        expiresIn: res.expires_in
    })
    
    ElMessage.success('GitHub 授权成功！正在配置仓库...')
    const repoPath = await invoke('galgame_github_setup_repo', { token })
    
    localCloud.value.type = 'GitHub'
    localCloud.value.repo = repoPath
    localCloud.value.branch = 'main'
    localCloud.value.token = token
    ElMessage.success(`设置成功：${repoPath}`)
  } catch (e) {
    ElMessage.error(`GitHub 授权失败: ${e}`)
  } finally {
    githubLoading.value = false
    githubDeviceCode.value = ''
    githubVerificationUri.value = ''
  }
}

const saveSettings = async () => {
  try {
    const backend = buildBackendObject()
    const config = await invoke('galgame_get_config')
    
    config.settings = { ...localSettings.value }
    config.cloud_settings = {
      backend,
      root_path: localCloud.value.rootPath,
      always_sync: localCloud.value.autoSync
    }
    
    await invoke('galgame_save_config', { config })
    ElMessage.success('设置已保存')
    emit('saved')
    handleClose()
  } catch (e) {
    ElMessage.error('保存失败: ' + e)
  }
}

const handlePrune = () => {
  emit('prune-config')
}

const handleClose = () => {
  emit('update:modelValue', false)
}
</script>

<template>
  <el-dialog
    :model-value="modelValue"
    @update:model-value="emit('update:modelValue', $event)"
    title="首选项"
    width="850px"
    class="vnite-settings-dialog"
    destroy-on-close
    append-to-body
  >
    <div class="settings-container">
      <el-tabs v-model="activeTab" tab-position="left" class="settings-tabs">
        <!-- 通用设置 -->
        <el-tab-pane name="general">
          <template #label>
            <el-icon><Setting /></el-icon><span>通用</span>
          </template>
          <div class="settings-pane">
            <h3>界面与交互</h3>
            <div class="settings-item">
              <div class="info">
                <div class="label">恢复前自动备份</div>
                <div class="desc">在恢复快照前，自动为当前存档创建一个备份</div>
              </div>
              <el-switch v-model="localSettings.backup_before_restore" />
            </div>
            <div class="settings-item">
              <div class="info">
                <div class="label">退出时最小化到托盘</div>
                <div class="desc">关闭主窗口时程序继续在后台运行 (配合托盘图标)</div>
              </div>
              <el-switch v-model="localSettings.exit_to_tray" />
            </div>

            <div class="settings-item">
              <div class="info">
                <div class="label">界面主题</div>
                <div class="desc">切换深色或浅色模式</div>
              </div>
              <el-radio-group v-model="localSettings.theme" size="small">
                <el-radio-button label="dark">深色</el-radio-button>
                <el-radio-button label="light">浅色</el-radio-button>
              </el-radio-group>
            </div>

            <div class="settings-item">
              <div class="info">
                <div class="label">网络代理 (HTTP/SOCKS5)</div>
                <div class="desc">GitHub 授权及云同步访问受限时配置 (如: http://127.0.0.1:7890)</div>
              </div>
              <el-input v-model="localSettings.http_proxy" placeholder="为空即不使用代理" style="width: 250px" clearable />
            </div>
            
            <el-divider />
            
            <h3>维护与优化</h3>
            <div class="settings-item">
              <div class="info">
                <div class="label">清理游戏库</div>
                <div class="desc">扫描并移除路径失效或重复的僵尸记录</div>
              </div>
              <el-button type="warning" plain @click="handlePrune">立即清理</el-button>
            </div>

            <el-divider />
            
            <h3>视觉辅助 (NSFW)</h3>
            <div class="settings-item">
              <div class="info">
                <div class="label">启用 NSFW 封面模糊</div>
                <div class="desc">对标记为 NSFW 的内容应用高斯模糊滤镜</div>
              </div>
              <el-switch v-model="localSettings.nsfw_blur" />
            </div>
            <div class="settings-item" v-if="localSettings.nsfw_blur">
              <div class="info">
                <div class="label">模糊强度</div>
                <div class="desc">调整模糊半径 ({{ localSettings.nsfw_blur_intensity }}px)</div>
              </div>
              <el-slider 
                v-model="localSettings.nsfw_blur_intensity" 
                :min="5" 
                :max="40" 
                style="width: 200px" 
              />
            </div>
          </div>
        </el-tab-pane>

        <!-- 云同步设置 -->
        <el-tab-pane name="cloud">
          <template #label>
            <el-icon><Cloudy /></el-icon><span>云同步</span>
          </template>
          <div class="settings-pane">
            <h3>云端集成</h3>
            <el-form :model="localCloud" label-width="100px" label-position="top">
              <el-form-item label="服务提供商">
                <el-select v-model="localCloud.type" style="width: 100%">
                  <el-option label="禁用" value="Disabled" />
                  <el-option-group label="WebDAV">
                    <el-option label="坚果云" value="jianguoyun" />
                    <el-option label="阿里云盘" value="aliyun" />
                    <el-option label="自定义 WebDAV" value="WebDAV" />
                  </el-option-group>
                  <el-option-group label="对象存储">
                    <el-option label="Amazon S3 / MinIO" value="S3" />
                    <el-option label="阿里云 OSS" value="AliyunOSS" />
                    <el-option label="GitHub (推荐)" value="GitHub" />
                  </el-option-group>
                </el-select>
              </el-form-item>

              <div v-if="localCloud.type === 'GitHub'" class="github-section">
                <div class="github-auth-card" v-if="!githubDeviceCode">
                  <el-button type="primary" @click="handleStartGithubOauth" :loading="githubLoading">
                    一键连接 GitHub
                  </el-button>
                </div>
                <div v-else class="github-polling-card">
                  <div class="device-code">{{ githubDeviceCode }}</div>
                  <p>请点击链接并输入上方验证码：</p>
                  <el-link :href="githubVerificationUri" target="_blank" type="primary">{{ githubVerificationUri }}</el-link>
                </div>
                
                <div class="github-manual-fields" v-if="cloudEnabled">
                   <el-form-item label="仓库路径 (Owner/Repo)">
                     <el-input v-model="localCloud.repo" placeholder="e.g. username/my-saves" />
                   </el-form-item>
                   <el-form-item label="访问令牌 (Token)">
                     <el-input v-model="localCloud.token" type="password" show-password />
                   </el-form-item>
                </div>
              </div>

              <template v-if="isWebDAVType">
                <el-form-item label="服务器地址">
                  <el-input v-model="localCloud.endpoint" :placeholder="getWebDAVPlaceholder" />
                </el-form-item>
                <el-form-item label="用户名">
                  <el-input v-model="localCloud.username" />
                </el-form-item>
                <el-form-item label="应用密码">
                  <el-input v-model="localCloud.password" type="password" show-password />
                </el-form-item>
              </template>

              <template v-if="localCloud.type === 'S3' || localCloud.type === 'AliyunOSS'">
                <el-form-item label="端点 (Endpoint)">
                  <el-input v-model="localCloud.endpoint" />
                </el-form-item>
                <el-form-item label="存储桶 (Bucket)">
                  <el-input v-model="localCloud.bucket" />
                </el-form-item>
                <el-form-item label="Access Key">
                  <el-input v-model="localCloud.accessKeyId" />
                </el-form-item>
                <el-form-item label="Secret Key">
                  <el-input v-model="localCloud.secretAccessKey" type="password" show-password />
                </el-form-item>
              </template>

              <el-divider v-if="cloudEnabled" />
              
              <template v-if="cloudEnabled">
                <el-form-item label="云端根目录">
                  <el-input v-model="localCloud.rootPath" placeholder="/galgame-saves" />
                </el-form-item>
                <div class="settings-item compact">
                  <div class="info">
                    <div class="label">修改即同步</div>
                    <div class="desc">存档变动后自动上传到云端</div>
                  </div>
                  <el-switch v-model="localCloud.autoSync" />
                </div>
                
                <div class="cloud-actions" style="margin-top: 20px;">
                  <el-button @click="handleTestConnection" :loading="testingCloud" type="primary">测试连接</el-button>
                  <el-button @click="emit('sync-from-cloud')" type="success">从云拉取全部</el-button>
                </div>
              </template>
            </el-form>
          </div>
        </el-tab-pane>

        <!-- 关于 -->
        <el-tab-pane name="about">
          <template #label>
            <el-icon><Monitor /></el-icon><span>关于</span>
          </template>
          <div class="settings-pane about-pane">
            <h2>Sunshine Control Panel</h2>
            <p class="version">Version 1.1.0</p>
            <el-divider />
            <p>基于 Tauri 2.0 与 Rust 构建的 Galgame 聚合管理平台。</p>
            <div class="links">
              <el-link type="primary" href="https://github.com/qiin2333/sunshine" target="_blank">GitHub Projects</el-link>
            </div>
          </div>
        </el-tab-pane>
      </el-tabs>
    </div>
    
    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose">取消</el-button>
        <el-button type="primary" @click="saveSettings">应用并保存</el-button>
      </div>
    </template>
  </el-dialog>
</template>

<style lang="less">
.vnite-settings-dialog {
  .el-dialog {
    background: var(--vnite-bg) !important;
    backdrop-filter: var(--vnite-blur) !important;
    -webkit-backdrop-filter: var(--vnite-blur) !important;
    border: 1px solid var(--vnite-border) !important;
    border-radius: 20px !important;
    box-shadow: var(--vnite-shadow) !important;
    overflow: hidden;
  }
  
  .el-dialog__header {
    margin: 0;
    padding: 24px 32px 12px;
    background: transparent !important;
    border-bottom: none !important; // 移除局部边框
    
    .el-dialog__title {
      color: var(--vnite-text);
      font-weight: 700;
      font-size: 18px;
    }
  }

  .el-dialog__body { 
    padding: 0 !important; 
    background: transparent !important;
    display: flex;
    flex-direction: column;
  }
}

.settings-container {
  height: 600px;
  display: flex;
  background: transparent;
  color: var(--vnite-text);
  transition: all 0.3s;
  overflow: hidden;

  .settings-tabs {
    width: 100%;
    height: 100%;
    border: none;
    
    :deep(.el-tabs__header) {
      margin-right: 0 !important;
      background: var(--vnite-sidebar-bg);
      border-right: none !important;
      padding: 0; // 移除内边距，靠子项撑开
      backdrop-filter: blur(10px);
      height: 100%;
    }

    :deep(.el-tabs__nav-wrap) {
      padding-top: 24px;
    }

    :deep(.el-tabs__item) {
      display: flex;
      align-items: center;
      gap: 12px;
      height: 56px;
      color: var(--vnite-text-muted);
      padding: 0 40px !important;
      transition: all @transition-fast;
      font-size: 15px;
      
      &.is-active {
        color: var(--vnite-primary);
        background: var(--vnite-accent);
        font-weight: 700;
      }

      &:hover:not(.is-active) {
        color: var(--vnite-text);
        background: rgba(255, 255, 255, 0.3);
      }
    }

    .el-tabs__content {
      flex: 1;
      height: 100%;
      overflow-y: auto;
      background: transparent;
    }
  }
}

.settings-pane {
  padding: 40px 48px;

  h3 {
    margin: 0 0 28px;
    font-size: 16px;
    font-weight: 700;
    color: var(--vnite-text);
    opacity: 0.9;
    letter-spacing: 0.5px;
    display: flex;
    align-items: center;
    gap: 8px;

    &::before {
      content: '';
      width: 4px;
      height: 16px;
      background: var(--vnite-primary);
      border-radius: 2px;
    }
  }
}

.settings-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 32px;
  padding: 16px 20px;
  background: var(--vnite-card-bg);
  border: 1px solid var(--vnite-border);
  border-radius: 12px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.02);
  
  &.compact { margin-bottom: 16px; }

  .info {
    .label { font-size: 15px; font-weight: 600; margin-bottom: 6px; color: var(--vnite-text); }
    .desc { font-size: 13px; color: var(--vnite-text-muted); opacity: 0.8; }
  }
}

.github-auth-card {
  background: var(--vnite-card-bg);
  border: 1px solid var(--vnite-border);
  padding: 32px;
  text-align: center;
  border-radius: 12px;
  margin-bottom: 24px;
  box-shadow: var(--vnite-shadow);
}

.github-polling-card {
  background: var(--vnite-accent);
  padding: 24px;
  border-radius: 12px;
  text-align: center;
  margin-bottom: 24px;
  border: 1px solid var(--vnite-border);
  
  .device-code {
    font-size: 32px;
    font-weight: 800;
    color: var(--vnite-primary);
    margin-bottom: 12px;
  }
}

.cloud-actions {
  display: flex;
  gap: 12px;
  margin-top: 10px;

  .el-button--primary, .el-button--success {
    color: #fff !important;
    font-weight: 600;
  }
}

.about-pane {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  height: 100%;
  
  h2 { color: var(--vnite-text); margin-bottom: 8px; }
  .version { font-size: 13px; color: var(--vnite-text-muted); opacity: 0.8; }
}

.dialog-footer {
  padding: 12px 32px 32px;
  background: transparent !important;
  border-top: none !important;
  backdrop-filter: none !important;
  margin-top: -12px; // 稍微向上压缩，减少空旷感
}
</style>
