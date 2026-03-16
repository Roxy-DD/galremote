<script setup>
import { ref, watch, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import { Connection, Download, Loading } from '@element-plus/icons-vue'

const props = defineProps({
  modelValue: {
    type: Boolean,
    default: false
  },
  initialSettings: {
    type: Object,
    required: true
  }
})

const emit = defineEmits(['update:modelValue', 'saved', 'sync-from-cloud'])

const settings = ref({ ...props.initialSettings })
const testing = ref(false)
const syncingFrom = ref(false)
const githubLoading = ref(false)
const githubDeviceCode = ref('')
const githubVerificationUri = ref('')

watch(() => props.modelValue, (val) => {
  if (val) {
    settings.value = JSON.parse(JSON.stringify(props.initialSettings))
    githubDeviceCode.value = ''
    githubVerificationUri.value = ''
  }
})

const isWebDAVType = computed(() => ['WebDAV', 'jianguoyun', 'aliyun'].includes(settings.value.type))
const cloudEnabled = computed(() => settings.value.type !== 'Disabled')

const getWebDAVPlaceholder = computed(() => {
  if (settings.value.type === 'jianguoyun') return 'https://dav.jianguoyun.com/dav/'
  if (settings.value.type === 'aliyun') return 'https://您的WebDAV地址'
  return 'https://your-webdav-server.com/dav/'
})

const buildBackendObject = () => {
  const type = settings.value.type
  if (type === 'Disabled') return { type: 'Disabled' }
  
  if (['WebDAV', 'jianguoyun', 'aliyun'].includes(type)) {
    let endpoint = settings.value.endpoint
    if (type === 'jianguoyun' && !endpoint) {
      endpoint = 'https://dav.jianguoyun.com/dav/'
    }
    return {
      type: 'WebDAV',
      endpoint,
      username: settings.value.username,
      password: settings.value.password
    }
  }
  
  if (type === 'S3' || type === 'AliyunOSS') {
    let endpoint = (settings.value.endpoint || '').trim()
    const bucket = (settings.value.bucket || '').trim()
    const region = (settings.value.region || '').trim()
    const ak = (settings.value.accessKeyId || '').trim()
    const sk = (settings.value.secretAccessKey || '').trim()
    
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
    const [owner, repo] = (settings.value.repo || '').trim().split('/')
    return {
      type: 'GitHub',
      owner: (owner || '').trim(),
      repo: (repo || '').trim(),
      branch: (settings.value.branch || 'main').trim(),
      token: (settings.value.token || '').trim()
    }
  }
  
  return { type: 'Disabled' }
}

const handleTestConnection = async () => {
  testing.value = true
  try {
    const backend = buildBackendObject()
    await invoke('galgame_check_cloud_connection', { backend })
    ElMessage.success('连接成功！')
  } catch (e) {
    ElMessage.error('连接失败: ' + e)
  } finally {
    testing.value = false
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
    
    ElMessage.success('GitHub 授权成功！正在自动配置仓库...')
    const repoPath = await invoke('galgame_github_setup_repo', { token: token })
    
    settings.value.type = 'GitHub'
    settings.value.repo = repoPath
    settings.value.branch = 'main'
    settings.value.token = token
    
    ElMessage.success(`仓库 ${repoPath} 设置完毕，别忘了点击底部保存！`)
  } catch (e) {
    ElMessage.error(`GitHub 授权失败: ${e}`)
  } finally {
    githubLoading.value = false
    githubDeviceCode.value = ''
    githubVerificationUri.value = ''
  }
}

const handleSave = () => {
  const backend = buildBackendObject()
  emit('saved', {
    autoSync: settings.value.autoSync,
    rootPath: settings.value.rootPath,
    backend
  })
  emit('update:modelValue', false)
}

const handleSyncFromCloud = () => {
  emit('sync-from-cloud')
}
</script>

<template>
  <el-dialog 
    :model-value="modelValue" 
    @update:model-value="emit('update:modelValue', $event)"
    title="云同步设置" 
    width="600px"
    class="vnite-cloud-dialog"
    append-to-body
  >
    <el-form :model="settings" label-width="100px" label-position="left">
      <el-form-item label="服务类型">
        <el-select v-model="settings.type" placeholder="选择云服务" style="width: 100%">
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

      <div v-if="settings.type === 'jianguoyun'" class="alert-info">
        坚果云 WebDAV 地址：https://dav.jianguoyun.com/dav/
        <br>请在坚果云官网生成“应用专用密码”使用。
      </div>

      <template v-if="isWebDAVType">
        <el-form-item label="服务器地址">
          <el-input v-model="settings.endpoint" :placeholder="getWebDAVPlaceholder" />
        </el-form-item>
        <el-form-item label="用户名">
          <el-input v-model="settings.username" placeholder="WebDAV 账户" />
        </el-form-item>
        <el-form-item label="授权密码">
          <el-input v-model="settings.password" type="password" show-password placeholder="WebDAV 密码" />
        </el-form-item>
      </template>

      <template v-if="settings.type === 'S3' || settings.type === 'AliyunOSS'">
        <el-form-item label="端点地址">
          <el-input v-model="settings.endpoint" :placeholder="settings.type === 'S3' ? 'https://s3.amazonaws.com' : 'https://oss-cn-hangzhou.aliyuncs.com'" />
        </el-form-item>
        <el-form-item label="存储桶">
          <el-input v-model="settings.bucket" placeholder="bucket-name" />
        </el-form-item>
        <el-form-item label="区域" v-if="settings.type === 'S3'">
          <el-input v-model="settings.region" placeholder="us-east-1" />
        </el-form-item>
        <el-form-item label="Access Key">
          <el-input v-model="settings.accessKeyId" placeholder="Access Key ID" />
        </el-form-item>
        <el-form-item label="Secret Key">
          <el-input v-model="settings.secretAccessKey" type="password" show-password placeholder="Secret Access Key" />
        </el-form-item>
      </template>

      <template v-if="settings.type === 'GitHub'">
        <div class="github-auth-card" v-if="!githubDeviceCode">
          <el-button 
            type="primary" 
            class="oauth-btn"
            @click="handleStartGithubOauth" 
            :loading="githubLoading"
          >
            一键连接 GitHub 账户
          </el-button>
          <p class="auth-hint">我们将自动为您创建备份仓库并配置权限</p>
        </div>
        
        <div v-else class="github-polling-card">
          <p>请在浏览器中输入验证码或确认授权：</p>
          <div class="device-code">{{ githubDeviceCode }}</div>
          <p class="verification-uri">
            <a :href="githubVerificationUri" target="_blank">{{ githubVerificationUri }}</a>
          </p>
          <div class="polling-status">
            <el-icon class="is-loading"><Loading /></el-icon>
            <span>正在等待授权结果...</span>
          </div>
        </div>

        <el-form-item label="仓库名">
          <el-input v-model="settings.repo" placeholder="username/repo" />
        </el-form-item>
        <el-form-item label="分支">
          <el-input v-model="settings.branch" placeholder="main" />
        </el-form-item>
        <el-form-item label="Access Token">
          <el-input v-model="settings.token" type="password" show-password placeholder="ghp_..." />
        </el-form-item>
      </template>

      <el-divider />

      <el-form-item label="云端根路径">
        <el-input v-model="settings.rootPath" placeholder="/galgame-saves" />
      </el-form-item>

      <el-form-item label="后台自动同步">
        <el-switch v-model="settings.autoSync" />
        <span class="switch-hint">{{ settings.autoSync ? '修改存档后自动上传' : '手动触发同步' }}</span>
      </el-form-item>

      <el-form-item>
        <div class="action-buttons">
          <el-button type="primary" plain @click="handleTestConnection" :loading="testing">
            <el-icon><Connection /></el-icon> 测试连接
          </el-button>
          <el-button plain @click="handleSyncFromCloud" :disabled="!cloudEnabled">
            <el-icon><Download /></el-icon> 从云端下载全部
          </el-button>
        </div>
      </el-form-item>
    </el-form>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="emit('update:modelValue', false)">取消</el-button>
        <el-button type="primary" @click="handleSave">保存配置</el-button>
      </div>
    </template>
  </el-dialog>
</template>

<style scoped lang="less">
.alert-info {
  background: rgba(var(--el-color-primary-rgb), 0.1);
  border-left: 4px solid var(--el-color-primary);
  padding: 10px 14px;
  border-radius: 4px;
  font-size: 13px;
  color: var(--el-text-color-regular);
  margin-bottom: 20px;
}

.github-auth-card {
  background: rgba(255, 255, 255, 0.05);
  border: 1px dashed rgba(255, 255, 255, 0.15);
  border-radius: 8px;
  padding: 24px;
  text-align: center;
  margin-bottom: 24px;

  .oauth-btn {
    width: 240px;
    height: 40px;
    font-weight: 600;
  }

  .auth-hint {
    font-size: 12px;
    color: rgba(255, 255, 255, 0.4);
    margin-top: 12px;
  }
}

.github-polling-card {
  background: #1a1a1a;
  border-radius: 8px;
  padding: 20px;
  text-align: center;
  margin-bottom: 24px;
  border: 1px solid var(--el-color-primary);

  .device-code {
    font-size: 28px;
    font-weight: 800;
    letter-spacing: 4px;
    color: var(--el-color-primary);
    margin: 15px 0;
    padding: 10px;
    background: #000;
    border-radius: 4px;
  }

  .verification-uri {
    font-size: 14px;
    margin-bottom: 15px;
    a { color: var(--el-color-primary); }
  }

  .polling-status {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    font-size: 13px;
    color: rgba(255, 255, 255, 0.6);
  }
}

.switch-hint {
  margin-left: 12px;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.4);
}

.action-buttons {
  display: flex;
  gap: 12px;
  margin-top: 10px;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}
</style>
