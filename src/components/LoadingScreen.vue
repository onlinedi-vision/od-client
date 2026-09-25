<template>
  <component :is="screen" :key="loadKey" />
</template>

<script setup>
import { shallowRef, ref, onBeforeMount } from 'vue'
import LoadingScreenBounce from './loading/LoadingScreenBounce.vue'
import LoadingScreenLife from './loading/LoadingScreenLife.vue'

const variants = {
  bounce: LoadingScreenBounce,
  life: LoadingScreenLife,
}

const screen = shallowRef(LoadingScreenBounce)
const loadKey = ref(0)

function pickLoadingScreen() {
  const forced = new URLSearchParams(window.location.search).get('loading')
  if (forced === 'life') return variants.life
  if (forced === 'bounce') return variants.bounce
  return Math.random() < 0.5 ? variants.bounce : variants.life
}

onBeforeMount(() => {
  screen.value = pickLoadingScreen()
  loadKey.value += 1
})
</script>
