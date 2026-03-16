<script setup>
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Search, Picture } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'

const props = defineProps({
  modelValue: {
    type: Boolean,
    default: false
  },
  targetGame: {
    type: Object,
    default: null
  }
})

const emit = defineEmits(['update:modelValue', 'applied'])

const query = ref('')
const results = ref([])
const selectedItem = ref(null)
const loading = ref(false)
const applying = ref(false)

// Sync query with target game name when dialog opens
watch(() => props.modelValue, (val) => {
  if (val && props.targetGame) {
    query.value = props.targetGame.name
    results.value = []
    selectedItem.value = null
  }
})

const handleSearch = async () => {
  if (!query.value.trim()) return
  
  loading.value = true
  try {
    const res = await invoke('galgame_search_metadata', { keyword: query.value, source: 'all' })
    results.value = res
    if (res.length === 0) {
      ElMessage.info('未找到相关结果')
    }
  } catch (e) {
    ElMessage.error('搜刮失败: ' + e)
    console.error(e)
  } finally {
    loading.value = false
  }
}

const handleApply = async () => {
  if (!selectedItem.value || !props.targetGame) return
  
  applying.value = true
  try {
    await invoke('galgame_apply_metadata', { 
       gameName: props.targetGame.name,
       data: selectedItem.value
    })
    ElMessage.success('元数据应用成功')
    emit('applied', props.targetGame.name)
    emit('update:modelValue', false)
  } catch (e) {
    ElMessage.error('应用失败: ' + e)
    console.error(e)
  } finally {
    applying.value = false
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
    title="元数据搜刮 (多源聚合)" 
    width="760px"
    class="vnite-scraper-dialog"
    append-to-body
  >
    <div class="scraper-header">
      <el-input 
        v-model="query" 
        placeholder="搜索游戏名称..." 
        @keyup.enter="handleSearch"
        clearable
      >
        <template #append>
          <el-button @click="handleSearch" :loading="loading">
            <el-icon><Search /></el-icon>
          </el-button>
        </template>
      </el-input>
    </div>

    <el-scrollbar height="420px" v-loading="loading">
      <div class="scraper-results-grid">
        <div 
          v-for="item in results" 
          :key="item.id + item.source" 
          class="scraper-card" 
          :class="{ active: selectedItem?.id === item.id && selectedItem?.source === item.source }"
          @click="selectedItem = item"
        >
          <div class="card-cover">
            <el-image :src="item.cover_url" fit="cover" lazy>
              <template #error>
                <div class="image-placeholder"><el-icon><Picture /></el-icon></div>
              </template>
            </el-image>
            <div class="source-badge">{{ item.source }}</div>
          </div>
          <div class="card-info">
            <div class="card-title" :title="item.title">{{ item.title }}</div>
            <div class="card-subtitle" v-if="item.original_title">{{ item.original_title }}</div>
            <div class="card-meta">
              <span>{{ item.release_date || '未知日期' }}</span>
              <span v-if="item.developer"> | {{ item.developer }}</span>
            </div>
            <div class="card-desc">{{ item.description || '暂无简介' }}</div>
          </div>
        </div>
        <el-empty v-if="!loading && results.length === 0" description="请输入关键词搜索游戏元数据" />
      </div>
    </el-scrollbar>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose">取消</el-button>
        <el-button 
          type="primary" 
          @click="handleApply" 
          :disabled="!selectedItem" 
          :loading="applying"
        >
          应用元数据
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<style scoped lang="less">
.vnite-scraper-dialog {
  :deep(.el-dialog) {
    background: var(--vnite-bg) !important;
    border: 1px solid var(--vnite-border) !important;
    .el-dialog__header { border-bottom: 1px solid var(--vnite-border); }
    .el-dialog__title { color: var(--vnite-text); }
  }
  :deep(.el-dialog__body) {
    padding-top: 10px;
  }
}

.scraper-header {
  margin-bottom: 20px;
}

.scraper-results-grid {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 4px;
}

.scraper-card {
  display: flex;
  gap: 16px;
  padding: 12px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s;

  &:hover {
    background: rgba(255, 255, 255, 0.06);
    border-color: rgba(255, 255, 255, 0.15);
  }

  &.active {
    background: rgba(var(--el-color-primary-rgb), 0.1);
    border-color: var(--el-color-primary);
    box-shadow: 0 0 0 1px var(--el-color-primary);
  }
}

.card-cover {
  width: 90px;
  height: 126px;
  flex-shrink: 0;
  border-radius: 6px;
  overflow: hidden;
  position: relative;
  background: #222;

  .el-image {
    width: 100%;
    height: 100%;
  }

  .image-placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #444;
    font-size: 24px;
  }

  .source-badge {
    position: absolute;
    top: 4px;
    right: 4px;
    background: rgba(0, 0, 0, 0.6);
    color: #fff;
    font-size: 10px;
    padding: 2px 6px;
    border-radius: 4px;
    backdrop-filter: blur(4px);
  }
}

.card-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  justify-content: center;

  .card-title {
    font-size: 16px;
    font-weight: 600;
    margin-bottom: 4px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .card-subtitle {
    font-size: 12px;
    color: rgba(255, 255, 255, 0.4);
    margin-bottom: 8px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .card-meta {
    font-size: 12px;
    color: rgba(255, 255, 255, 0.5);
    margin-bottom: 8px;
  }

  .card-desc {
    font-size: 12px;
    color: rgba(255, 255, 255, 0.4);
    line-height: 1.5;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}
</style>
