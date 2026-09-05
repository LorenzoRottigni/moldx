<template>
  <div class="home">
    <h1>MoldX Playground</h1>
    <p>Nuxt frontend served by MoldX.</p>
    <span class="status">online</span>
  </div>
</template>

<script setup>
const features = [
  'Module discovery',
  'Profile matching',
  'Command resolution',
  'Glob targets'
];
</script>

<style scoped>
.home { font-family: system-ui; max-width: 480px; margin: 3rem auto; }
.status { color: green; font-weight: bold; }
</style>
