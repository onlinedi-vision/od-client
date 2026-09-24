<template>
  <div class="loading-screen" aria-hidden="true">
    <canvas ref="canvasEl" />
  </div>
</template>

<script setup>
import { onMounted, onUnmounted, ref } from 'vue'

const GRID_ROWS = 90
const BALL_RADIUS_CELLS = 2.2
const SPEED_PX_S = 800
const GAP_RATIO = 0.12

const canvasEl = ref(null)
let frameId = 0
let ro = null

function cssVar(name) {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim()
}

function stepBounce(state, dt, bounds, radius) {
  let { x, y, vx, vy } = state
  x += vx * dt
  y += vy * dt
  if (x - radius <= bounds.left) {
    x = bounds.left + radius
    vx = Math.abs(vx)
  } else if (x + radius >= bounds.right) {
    x = bounds.right - radius
    vx = -Math.abs(vx)
  }
  if (y - radius <= bounds.top) {
    y = bounds.top + radius
    vy = Math.abs(vy)
  } else if (y + radius >= bounds.bottom) {
    y = bounds.bottom - radius
    vy = -Math.abs(vy)
  }
  return { x, y, vx, vy }
}

if (import.meta.env.DEV) {
  const r = stepBounce(
    { x: 10, y: 10, vx: -50, vy: 40 },
    0.2,
    { left: 0, top: 0, right: 100, bottom: 100 },
    5
  )
  console.assert(r.x === 5 && r.vx > 0, 'loading bounce self-check')
}

onMounted(() => {
  const canvas = canvasEl.value
  if (!canvas) return
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  let width = 0
  let height = 0
  let cell = 0
  let cols = 0
  let ball = { x: 0, y: 0, vx: SPEED_PX_S, vy: SPEED_PX_S * 0.85 }
  let lastTs = 0

  const resize = () => {
    const dpr = Math.min(window.devicePixelRatio || 1, 2)
    width = window.innerWidth
    height = window.innerHeight
    cell = height / GRID_ROWS
    cols = Math.ceil(width / cell)
    canvas.width = Math.floor(width * dpr)
    canvas.height = Math.floor(height * dpr)
    canvas.style.width = `${width}px`
    canvas.style.height = `${height}px`
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
    if (ball.x === 0 && ball.y === 0) {
      ball.x = width * 0.35
      ball.y = height * 0.4
      ball.vx = SPEED_PX_S * (Math.random() > 0.5 ? 1 : -1)
      ball.vy = SPEED_PX_S * 0.9 * (Math.random() > 0.5 ? 1 : -1)
    }
  }

  const draw = (ts) => {
    if (!lastTs) lastTs = ts
    const dt = Math.min((ts - lastTs) / 1000, 0.05)
    lastTs = ts

    const bg = cssVar('--bg-app') || '#080607'
    const tile = cssVar('--bg-sidebar') || '#110f11'
    const ballFill = cssVar('--accent') || '#c48a92'
    const radius = cell * BALL_RADIUS_CELLS
    const gap = cell * GAP_RATIO
    const inset = gap / 2
    const tileSize = cell - gap

    ball = stepBounce(ball, dt, { left: 0, top: 0, right: width, bottom: height }, radius)

    ctx.fillStyle = bg
    ctx.fillRect(0, 0, width, height)

    for (let row = 0; row < GRID_ROWS; row++) {
      for (let col = 0; col < cols; col++) {
        const cx = col * cell + cell / 2
        const cy = row * cell + cell / 2
        const dx = cx - ball.x
        const dy = cy - ball.y
        const inside = dx * dx + dy * dy <= radius * radius
        ctx.fillStyle = inside ? ballFill : tile
        ctx.fillRect(col * cell + inset, row * cell + inset, tileSize, tileSize)
      }
    }

    frameId = requestAnimationFrame(draw)
  }

  resize()
  ro = new ResizeObserver(resize)
  ro.observe(document.documentElement)
  frameId = requestAnimationFrame(draw)
})

onUnmounted(() => {
  cancelAnimationFrame(frameId)
  ro?.disconnect()
})
</script>

<style scoped>
.loading-screen {
  position: fixed;
  inset: 0;
  z-index: 100000;
  background: var(--bg-app);
  overflow: hidden;
}

canvas {
  display: block;
  width: 100%;
  height: 100%;
}
</style>
