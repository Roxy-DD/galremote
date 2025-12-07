<template>
  <div class="tools-page">
    <div class="tool-container">
      <!-- 串流计时器 - 本地页面 -->
      <iframe 
        v-if="initialTool === 'timer'"
        :src="timerUrl"
        class="tool-iframe"
        frameborder="0"
        allow="fullscreen"
        loading="eager"
      />

      <!-- 延迟测试 - 外部页面 -->
      <iframe 
        v-else-if="initialTool === 'delay'"
        src="https://yangkile.github.io/D-lay/"
        class="tool-iframe"
        frameborder="0"
        allow="fullscreen"
        loading="lazy"
      />

      <!-- 手柄测试 - 外部页面 -->
      <iframe 
        v-else-if="initialTool === 'gamepad'"
        src="https://hardwaretester.com/gamepad"
        class="tool-iframe"
        frameborder="0"
        allow="fullscreen; gamepad"
        loading="lazy"
      />


    </div>
  </div>
</template>

<script setup>
import { defineProps } from 'vue'

const props = defineProps({
  initialTool: {
    type: String,
    default: 'timer'
  }
})

const timerUrl = window.location.origin + '/stop-clock-canvas/index.html'
</script>

<style scoped lang="less">
.tools-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--el-bg-color);
}

.tool-container {
  flex: 1;
  overflow: hidden;
}

.tool-iframe {
  width: 100%;
  height: 100%;
  border: none;
  background: #1a1a2e;
}
</style>
