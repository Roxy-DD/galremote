<template>
  <div class="vnite-image" :style="containerStyle">
    <el-image
      ref="imageRef"
      :src="displayUrl"
      :fit="fit"
      v-bind="$attrs"
      class="vnite-image-inner"
      :preview-src-list="displayUrl && preview ? [displayUrl] : []"
      :loading="loading ? 'lazy' : 'eager'"
      @error="handleError"
    >
      <template #placeholder>
        <div class="image-slot placeholder">
          <el-icon class="is-loading"><Loading /></el-icon>
        </div>
      </template>
      <template #error>
        <div class="image-slot error">
          <el-icon><Picture /></el-icon>
        </div>
      </template>
    </el-image>
  </div>
</template>

<script setup>
import { ref, watch, onMounted } from 'vue'
import { Picture, Loading } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps({
  src: {
    type: String,
    default: ''
  },
  fit: {
    type: String,
    default: 'cover'
  },
  preview: {
    type: Boolean,
    default: false
  },
  loading: {
    type: Boolean,
    default: true
  },
  radius: {
    type: String,
    default: '8px'
  }
})

const displayUrl = ref('')
const imageRef = ref(null)

const containerStyle = {
  borderRadius: props.radius,
  overflow: 'hidden',
  display: 'block',
  width: '100%',
  height: '100%'
}

const loadImageUrl = async (path) => {
  if (!path || !path.trim()) {
    displayUrl.value = ''
    return
  }

  // If it's already a URL or base64, use it directly
  if (path.startsWith('http') || path.startsWith('data:')) {
    displayUrl.value = path
    return
  }

  try {
    // Call backend to get Data URL for local file
    const dataUrl = await invoke('read_image_as_data_url', { path })
    displayUrl.value = dataUrl
  } catch (error) {
    console.error('Failed to load local image:', error, path)
    displayUrl.value = '' // Fallback to error slot
  }
}

watch(() => props.src, (newPath) => {
  loadImageUrl(newPath)
}, { immediate: true })

const handleError = (e) => {
  // console.debug('Image load error:', e)
}

defineExpose({
  imageRef
})
</script>

<style scoped>
.vnite-image {
  position: relative;
  background: var(--vnite-card-bg);
}

.vnite-image-inner {
  width: 100%;
  height: 100%;
}

.image-slot {
  display: flex;
  justify-content: center;
  align-items: center;
  width: 100%;
  height: 100%;
  color: var(--vnite-text-muted);
  font-size: 24px;
}

.image-slot.placeholder {
  background: var(--vnite-card-bg);
}

.image-slot.error {
  background: var(--vnite-card-bg);
  opacity: 0.6;
}
</style>
