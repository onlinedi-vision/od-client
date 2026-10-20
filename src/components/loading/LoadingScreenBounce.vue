<template>
  <div class="loading-screen" aria-hidden="true">
    <canvas ref="canvasEl" />
    <p class="loading-caption" :style="captionStyle">{{ CAPTION_LABEL }}</p>
  </div>
</template>

<script setup>
import { onMounted, onUnmounted, ref } from 'vue'
import {
  GRID_ROWS,
  CAPTION_LABEL,
  createTrailState,
  ensureTrail,
  buildGlow,
  fadeTrail,
  drawGrid,
  layoutCaption,
  setupCanvas,
  loadingColors,
} from './loadingCanvas.js'

const BALL_RADIUS_CELLS = 1.8
const SPEED_PX_S = 800

const canvasEl = ref(null)
const captionStyle = ref({ visibility: 'hidden' })
let frameId = 0
let ro = null

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

  const trailState = createTrailState()
  let ball = { x: 0, y: 0, vx: SPEED_PX_S, vy: SPEED_PX_S * 0.85 }
  let lastTs = 0
  const { resize, getMetrics } = setupCanvas(canvas, ctx)

  const stampBall = (bx, by, radius, cols, cell) => {
    const { trail } = trailState
    const minCol = Math.max(0, Math.floor((bx - radius) / cell))
    const maxCol = Math.min(cols - 1, Math.ceil((bx + radius) / cell))
    const minRow = Math.max(0, Math.floor((by - radius) / cell))
    const maxRow = Math.min(GRID_ROWS - 1, Math.ceil((by + radius) / cell))
    const r2 = radius * radius
    for (let row = minRow; row <= maxRow; row++) {
      for (let col = minCol; col <= maxCol; col++) {
        const cx = col * cell + cell / 2
        const cy = row * cell + cell / 2
        const dx = cx - bx
        const dy = cy - by
        if (dx * dx + dy * dy <= r2) trail[row * cols + col] = 1
      }
    }
  }

  const stampSegment = (x0, y0, x1, y1, radius, cols, cell) => {
    const dist = Math.hypot(x1 - x0, y1 - y0)
    const step = Math.max(cell * 0.35, 1)
    const n = Math.max(1, Math.ceil(dist / step))
    for (let i = 0; i <= n; i++) {
      const t = i / n
      stampBall(x0 + (x1 - x0) * t, y0 + (y1 - y0) * t, radius, cols, cell)
    }
  }

  const onLayout = ({ width, height, cell, cols }) => {
    ensureTrail(trailState, cols)
    if (ball.x === 0 && ball.y === 0) {
      ball.x = width * 0.35
      ball.y = height * 0.4
      ball.vx = SPEED_PX_S * (Math.random() > 0.5 ? 1 : -1)
      ball.vy = SPEED_PX_S * 0.9 * (Math.random() > 0.5 ? 1 : -1)
    }
    layoutCaption(ctx, captionStyle, { cols, cell })
  }

  const draw = (ts) => {
    if (!lastTs) lastTs = ts
    const dt = Math.min((ts - lastTs) / 1000, 0.05)
    lastTs = ts

    const { width, height, cell, cols } = getMetrics()
    const colors = loadingColors()
    const radius = cell * BALL_RADIUS_CELLS

    const prevX = ball.x
    const prevY = ball.y
    ball = stepBounce(ball, dt, { left: 0, top: 0, right: width, bottom: height }, radius)

    ensureTrail(trailState, cols)
    stampSegment(prevX, prevY, ball.x, ball.y, radius, cols, cell)

    fadeTrail(trailState, dt, GRID_ROWS, (_idx, col, row) => {
      const cx = col * cell + cell / 2
      const cy = row * cell + cell / 2
      const dx = cx - ball.x
      const dy = cy - ball.y
      return dx * dx + dy * dy <= radius * radius
    })

    buildGlow(trailState)
    drawGrid(ctx, trailState, { width, height, cell, colors: { ...colors, fill: colors.fill } })

    frameId = requestAnimationFrame(draw)
  }

  resize(onLayout)
  ro = new ResizeObserver(() => resize(onLayout))
  ro.observe(document.documentElement)
  frameId = requestAnimationFrame(draw)
})

onUnmounted(() => {
  cancelAnimationFrame(frameId)
  ro?.disconnect()
})
</script>

<style scoped>
@import './loadingScreen.css';
</style>
