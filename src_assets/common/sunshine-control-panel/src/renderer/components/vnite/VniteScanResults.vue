<script setup>
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Folder, CircleCheck, Check } from '@element-plus/icons-vue'

const props = defineProps({
  modelValue: {
    type: Boolean,
    default: false
  },
  candidates: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits(['update:modelValue', 'quick-add', 'batch-add'])

const selectedPaths = ref([])

const handleQuickAdd = (candidate) => {
  emit('quick-add', candidate)
}

const handleBatchAdd = () => {
  const selected = props.candidates.filter(c => selectedPaths.value.includes(c.path))
  emit('batch-add', selected)
  selectedPaths.value = []
}

const toggleSelection = (path) => {
  const idx = selectedPaths.value.indexOf(path)
  if (idx > -1) {
    selectedPaths.value.splice(idx, 1)
  } else {
    selectedPaths.value.push(path)
  }
}

const allSelected = computed({
  get: () => props.candidates.length > 0 && selectedPaths.value.length === props.candidates.length,
  set: (val) => {
    if (val) {
      selectedPaths.value = props.candidates.map(c => c.path)
    } else {
      selectedPaths.value = []
    }
  }
})

const getConfidenceColor = (confidence) => {
  if (confidence >= 0.7) return '#67c23a'
  if (confidence >= 0.5) return '#e6a23c'
  return '#909399'
}
</script>

<template>
  <el-dialog 
    :model-value="modelValue" 
    @update:model-value="emit('update:modelValue', $event)"
    title="扫描结果" 
    width="680px"
    class="vnite-scan-dialog"
    append-to-body
  >
    <div class="scan-hint">
      以下是程序检测到的疑似游戏存档路径，您可以点击快速添加到库中。
    </div>

    <el-scrollbar height="420px">
      <div class="scan-grid">
        <div class="scan-toolbar" v-if="candidates.length > 0">
          <el-checkbox v-model="allSelected">全选 ({{ selectedPaths.length }}/{{ candidates.length }})</el-checkbox>
        </div>
        <div 
          v-for="candidate in candidates" 
          :key="candidate.path" 
          class="scan-item" 
          :class="{ selected: selectedPaths.includes(candidate.path) }"
          @click="toggleSelection(candidate.path)"
        >
          <div class="item-checkbox">
            <el-checkbox 
              :model-value="selectedPaths.includes(candidate.path)" 
              @click.stop
              @change="toggleSelection(candidate.path)"
            />
          </div>
          <div class="item-icon">
            <el-icon :size="24"><Folder /></el-icon>
          </div>
          <div class="item-info">
            <div class="item-name">{{ candidate.game_name }}</div>
            <div class="item-path">{{ candidate.path }}</div>
          </div>
          <div class="item-confidence">
            <div class="confidence-label">匹配度 {{ Math.round(candidate.confidence * 100) }}%</div>
            <el-progress 
              :percentage="Math.round(candidate.confidence * 100)" 
              :color="getConfidenceColor(candidate.confidence)"
              :show-text="false"
              :stroke-width="6"
              style="width: 80px"
            />
          </div>
        </div>
        <el-empty v-if="candidates.length === 0" description="未发现可疑的存档目录" />
      </div>
    </el-scrollbar>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="emit('update:modelValue', false)">取消</el-button>
        <el-button 
          v-if="candidates.length > 0"
          type="primary" 
          :disabled="selectedPaths.length === 0"
          @click="handleBatchAdd"
        >
          批量添加已选 ({{ selectedPaths.length }})
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<style scoped lang="less">
.scan-hint {
  font-size: 13px;
  color: var(--vnite-text-muted);
  margin-bottom: 20px;
}

.scan-grid {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 4px;
}

.scan-item {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px;
  background: var(--vnite-card-bg);
  border: 1px solid var(--vnite-border);
  border-radius: 12px;
  cursor: pointer;
  transition: all @transition-fast;
  backdrop-filter: var(--vnite-blur);
  position: relative;

  &::before {
    content: '';
    position: absolute;
    inset: 0;
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 12px;
    z-index: 1;
    pointer-events: none;
  }

  &:hover {
    background: var(--vnite-accent);
    border-color: rgba(var(--el-color-primary-rgb), 0.3);
    transform: translateX(4px);
  }

  &.selected {
    border-color: var(--el-color-primary);
    background: rgba(var(--el-color-primary-rgb), 0.05);
  }
}

.item-icon {
  width: 48px;
  height: 48px;
  background: rgba(var(--el-color-primary-rgb), 0.1);
  color: var(--el-color-primary);
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.item-info {
  flex: 1;
  min-width: 0;

  .item-name {
    font-size: 15px;
    font-weight: 600;
    margin-bottom: 4px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--vnite-text);
  }

  .item-path {
    font-size: 12px;
    color: var(--vnite-text-muted);
    opacity: 0.8;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
}

.item-confidence {
  text-align: right;
  flex-shrink: 0;

  .confidence-label {
    font-size: 11px;
    color: var(--vnite-text-muted);
    margin-bottom: 4px;
  }
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
}
</style>
