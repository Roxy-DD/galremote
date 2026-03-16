<template>
  <div id="app" class="vnite-theme">
    <router-view />
  </div>
</template>

<script setup>
</script>

<style>
/* App specific global styles - using standard CSS to avoid LESS conflicts */
#app {
  width: 100vw;
  height: 100vh;
  margin: 0;
  padding: 0;
  overflow: hidden;
  background-color: transparent;
}

#app.vnite-theme {
  background-color: var(--vnite-bg);
}
</style>