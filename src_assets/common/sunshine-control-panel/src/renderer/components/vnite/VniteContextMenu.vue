<script setup>
import { ref, onMounted, onUnmounted, nextTick, watch } from 'vue'

const props = defineProps({
  visible: Boolean,
  x: Number,
  y: Number,
  items: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits(['update:visible', 'command'])

const menuRef = ref(null)
const adjustedX = ref(0)
const adjustedY = ref(0)

const closeMenu = () => {
  emit('update:visible', false)
}

const handleCommand = (item) => {
  if (item.disabled) return
  emit('command', item.command)
  closeMenu()
}

const updatePosition = async () => {
  if (!props.visible) return
  
  await nextTick()
  if (!menuRef.value) return

  const menuWidth = menuRef.value.offsetWidth
  const menuHeight = menuRef.value.offsetHeight
  const screenWidth = window.innerWidth
  const screenHeight = window.innerHeight

  let posX = props.x
  let posY = props.y

  // Boundary checks
  if (posX + menuWidth > screenWidth) {
    posX = screenWidth - menuWidth - 10
  }
  if (posY + menuHeight > screenHeight) {
    posY = screenHeight - menuHeight - 10
  }

  adjustedX.value = Math.max(10, posX)
  adjustedY.value = Math.max(10, posY)
}

watch(() => props.visible, (newVal) => {
  if (newVal) {
    updatePosition()
    // Add click-outside listener
    setTimeout(() => {
      window.addEventListener('click', closeMenu)
      window.addEventListener('contextmenu', closeMenu)
    }, 10)
  } else {
    window.removeEventListener('click', closeMenu)
    window.removeEventListener('contextmenu', closeMenu)
  }
})

onUnmounted(() => {
  window.removeEventListener('click', closeMenu)
  window.removeEventListener('contextmenu', closeMenu)
})
</script>

<template>
  <teleport to="body">
    <transition name="menu-fade">
      <div 
        v-if="visible"
        ref="menuRef"
        class="vnite-context-menu"
        :style="{ 
          left: adjustedX + 'px', 
          top: adjustedY + 'px' 
        }"
        @click.stop
      >
        <div class="menu-inner">
          <div 
            v-for="(item, index) in items" 
            :key="index"
            :class="['menu-item', { divider: item.divider, danger: item.danger, disabled: item.disabled }]"
            @click="handleCommand(item)"
          >
            <template v-if="!item.divider">
              <el-icon v-if="item.icon" class="item-icon">
                <component :is="item.icon" />
              </el-icon>
              <span class="item-label">{{ item.label }}</span>
            </template>
          </div>
        </div>
      </div>
    </transition>
  </teleport>
</template>

<style scoped lang="less">
.vnite-context-menu {
  position: fixed;
  z-index: 9999;
  min-width: 180px;
  pointer-events: auto;
}

.menu-inner {
  background: rgba(var(--vnite-bg-rgb, 20, 20, 22), 0.7);
  backdrop-filter: blur(20px) saturate(180%);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 12px;
  padding: 6px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5), 0 0 0 1px rgba(255, 255, 255, 0.05);
  overflow: hidden;
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  color: rgba(255, 255, 255, 0.85);
  font-size: 13px;
  user-select: none;

  &:hover:not(.disabled) {
    background: rgba(255, 255, 255, 0.08);
    color: #fff;
    transform: translateX(2px);
  }

  &.danger {
    color: #f56c6c;
    &:hover { background: rgba(245, 108, 108, 0.15); }
  }

  &.disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  &.divider {
    height: 1px;
    background: rgba(255, 255, 255, 0.1);
    margin: 4px 6px;
    padding: 0;
    cursor: default;
    &:hover { transform: none; background: rgba(255, 255, 255, 0.1); }
  }

  .item-icon {
    font-size: 16px;
    opacity: 0.9;
  }

  .item-label {
    flex: 1;
    font-weight: 500;
  }
}

/* Animations */
.menu-fade-enter-active, .menu-fade-leave-active {
  transition: opacity 0.15s ease, transform 0.15s cubic-bezier(0.4, 0, 0.2, 1);
}

.menu-fade-enter-from, .menu-fade-leave-to {
  opacity: 0;
  transform: scale(0.95) translateY(-10px);
}

/* Light mode support */
:global(.light-mode) .menu-inner {
  background: rgba(255, 255, 255, 0.75);
  border-color: rgba(0, 0, 0, 0.1);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
  
  .menu-item {
    color: #333;
    &:hover:not(.disabled) { background: rgba(0, 0, 0, 0.05); }
    &.divider { background: rgba(0, 0, 0, 0.05); }
  }
}
</style>
