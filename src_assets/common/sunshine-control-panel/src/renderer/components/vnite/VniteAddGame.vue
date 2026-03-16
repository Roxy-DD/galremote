<script setup>
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { ElMessage } from 'element-plus'

const props = defineProps({
  modelValue: {
    type: Boolean,
    default: false
  },
  editingGame: {
    type: Object,
    default: null
  }
})

const emit = defineEmits(['update:modelValue', 'saved'])

const form = ref({
  name: '',
  savePath: '',
  pathType: 'Folder',
  backupMode: 'manual',
  backupInterval: 60,
  exePath: '',
  coverImage: '',
})

const loading = ref(false)

// Initialize form when dialog opens or editingGame changes
watch(() => props.modelValue, (val) => {
  if (val) {
    if (props.editingGame) {
      const g = props.editingGame
      form.value = {
        name: g.name,
        savePath: g.save_paths?.[0]?.paths?.default || '',
        pathType: String(g.save_paths?.[0]?.unit_type).toLowerCase() === 'file' ? 'File' : 'Folder',
        backupMode: g.backup_mode || 'manual',
        backupInterval: g.auto_backup_interval || 60,
        exePath: g.exe_path || '',
        coverImage: g.cover_image || '',
      }
    } else {
      form.value = {
        name: '',
        savePath: '',
        pathType: 'Folder',
        backupMode: 'manual',
        backupInterval: 60,
        exePath: '',
        coverImage: '',
      }
    }
  }
})

const handleBrowseSavePath = async () => {
  try {
    const isFolder = form.value.pathType === 'Folder'
    const selected = await open({ 
      directory: isFolder,
      multiple: false,
      title: isFolder ? '选择存档目录' : '选择存档文件'
    })
    const selectedPath = Array.isArray(selected) ? selected[0] : selected
    if (selectedPath) {
      form.value.savePath = selectedPath
    }
  } catch (e) {
    ElMessage.error('选择失败: ' + e)
  }
}

const handleBrowseExePath = async () => {
  try {
    const selected = await open({ 
      filters: [{ name: '可执行文件', extensions: ['exe', 'lnk', 'bat', 'cmd'] }],
      title: '选择启动文件'
    })
    const selectedPath = Array.isArray(selected) ? selected[0] : selected
    if (selectedPath) {
      form.value.exePath = selectedPath
    }
  } catch (e) {
    console.error(e)
  }
}

const handleBrowseCover = async () => {
  try {
    const selected = await open({ 
      filters: [{ name: '图片', extensions: ['png', 'jpg', 'jpeg', 'webp'] }],
      title: '选择游戏封面'
    })
    const selectedPath = Array.isArray(selected) ? selected[0] : selected
    if (selectedPath) {
      form.value.coverImage = selectedPath
    }
  } catch (e) {
    console.error(e)
  }
}

const handleSave = async () => {
  const name = form.value.name.trim()
  const savePath = form.value.savePath.trim()

  if (!name || !savePath) {
    ElMessage.warning('请填写游戏名称和存档路径')
    return
  }

  loading.value = true
  try {
    const game = {
      name,
      save_paths: [{
        unit_type: form.value.pathType.toLowerCase(),
        paths: { default: savePath },
        delete_before_apply: false,
      }],
      game_paths: {},
      exe_path: form.value.exePath.trim() || null,
      backup_mode: form.value.backupMode,
      auto_backup_interval: form.value.backupInterval,
      cover_image: form.value.coverImage.trim() || null,
    }

    await invoke('galgame_add_game', {
      game,
      update: !!props.editingGame,
      oldName: props.editingGame ? props.editingGame.name : null,
    })

    ElMessage.success(props.editingGame ? '游戏已更新' : '游戏添加成功')
    emit('saved')
    emit('update:modelValue', false)
  } catch (e) {
    ElMessage.error('操作失败: ' + e)
  } finally {
    loading.value = false
  }
}

const handleClose = () => {
  emit('update:modelValue', false)
}
</script>

<template>
  <el-dialog 
    :model-value="modelValue" 
    @update:model-value="emit('update:modelValue', $event)"
    :title="editingGame ? '编辑游戏' : '添加游戏'" 
    width="560px"
    class="vnite-add-dialog"
    append-to-body
  >
    <el-form :model="form" label-width="100px" label-position="left">
      <el-form-item label="游戏名称" required>
        <el-input v-model="form.name" placeholder="例如：Summer Pockets" />
      </el-form-item>
      
      <el-form-item label="存档路径" required>
        <el-input v-model="form.savePath" placeholder="存档所在的目录或文件">
          <template #append>
            <el-button @click="handleBrowseSavePath">浏览</el-button>
          </template>
        </el-input>
      </el-form-item>

      <el-form-item label="路径类型">
        <el-radio-group v-model="form.pathType">
          <el-radio value="Folder">文件夹</el-radio>
          <el-radio value="File">单个文件</el-radio>
        </el-radio-group>
      </el-form-item>

      <el-divider content-position="left">高级设置</el-divider>

      <el-form-item label="备份模式">
        <el-select v-model="form.backupMode" style="width: 100%">
          <el-option label="手动备份" value="manual" />
          <el-option label="退出游戏时自动备份" value="on_game_exit" />
          <el-option label="定时备份" value="scheduled" />
          <el-option label="退出+定时备份" value="both" />
        </el-select>
      </el-form-item>

      <el-form-item 
        label="备份间隔" 
        v-if="form.backupMode === 'scheduled' || form.backupMode === 'both'"
      >
        <el-input-number v-model="form.backupInterval" :min="5" :max="1440" />
        <span style="margin-left: 10px">分钟</span>
      </el-form-item>

      <el-form-item label="启动路径">
        <el-input v-model="form.exePath" placeholder="游戏 .exe 文件路径">
          <template #append>
            <el-button @click="handleBrowseExePath">浏览</el-button>
          </template>
        </el-input>
      </el-form-item>

      <el-form-item label="自定义封面">
        <el-input v-model="form.coverImage" placeholder="本地图片路径">
          <template #append>
            <el-button @click="handleBrowseCover">选择</el-button>
          </template>
        </el-input>
      </el-form-item>
    </el-form>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose">取消</el-button>
        <el-button type="primary" @click="handleSave" :loading="loading">
          {{ editingGame ? '保存变更' : '立即添加' }}
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<style scoped lang="less">
.vnite-add-dialog {
  :deep(.el-form-item__label) {
    font-weight: 600;
    color: var(--vnite-text);
  }

  :deep(.el-divider) {
    margin: 32px 0 24px;
    border-top: 1px solid var(--vnite-border);
  }

  :deep(.el-divider__text) {
    background-color: transparent !important;
    color: var(--vnite-primary);
    font-weight: 700;
    font-size: 13px;
    letter-spacing: 1px;
    padding: 0 16px;
  }
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 8px 0 12px;
}
</style>
