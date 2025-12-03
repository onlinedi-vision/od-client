<template>
  <App :key="appKey"/>
</template>

<script>
import { ref } from 'vue'
import App from './App.vue'

const appKey = ref(0)

export function reloadApp() {
  appKey.value++
}

export default {
  components: { App },
  setup() {
    return { appKey }
  }
}
</script>