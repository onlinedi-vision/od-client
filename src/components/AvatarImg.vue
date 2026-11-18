<template>
  <span
    class="avatar-img"
    :class="[imgClass, { 'avatar-img--square': square }]"
    v-bind="$attrs"
    :style="boxStyle"
  >
    <img
      v-if="!useFallback"
      :src="normalizedSrc"
      class="avatar-img-el"
      alt=""
      @error="failed = true"
    />
    <span v-else class="avatar-img-fallback" aria-hidden="true"><span class="avatar-img-q">?</span></span>
  </span>
</template>

<script setup>
import { computed, ref, watch } from 'vue'

defineOptions({ inheritAttrs: false })

const props = defineProps({
  src: { type: String, default: '' },
  width: { type: [Number, String], default: 40 },
  height: { type: [Number, String], default: 40 },
  imgClass: { type: String, default: '' },
  imgStyle: { type: Object, default: () => ({}) },
  square: { type: Boolean, default: false },
})

const failed = ref(false)

const normalizedSrc = computed(() => (props.src || '').trim())

const useFallback = computed(() => !normalizedSrc.value || failed.value)

watch(normalizedSrc, () => {
  failed.value = false
})

function toPx(value) {
  return typeof value === 'number' ? `${value}px` : value
}

const boxStyle = computed(() => ({
  width: toPx(props.width),
  height: toPx(props.height),
  '--avatar-size': toPx(props.width),
  ...props.imgStyle,
}))
</script>

<style scoped>
.avatar-img {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  vertical-align: top;
  line-height: 0;
  overflow: hidden;
  flex-shrink: 0;
  box-sizing: border-box;
  border-radius: 50%;
  user-select: none;
  -webkit-user-select: none;
  cursor: default;
}

.avatar-img--square {
  border-radius: 0;
}

.avatar-img-el {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: cover;
  user-select: none;
  -webkit-user-drag: none;
  pointer-events: none;
}

.avatar-img-fallback {
  display: grid;
  place-items: center;
  width: 100%;
  height: 100%;
  background: var(--bg-rail);
  color: var(--text-muted);
  font-weight: 600;
  font-size: calc(var(--avatar-size, 40px) * 0.42);
  line-height: 1;
  user-select: none;
  -webkit-user-select: none;
  cursor: inherit;
}

.avatar-img-q {
  display: block;
  line-height: 1;
  transform: translateY(0.09em);
  user-select: none;
  -webkit-user-select: none;
  pointer-events: none;
}
</style>
