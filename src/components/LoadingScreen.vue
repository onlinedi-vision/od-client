<template>
  <div class="loading-screen" aria-hidden="true">
    <canvas ref="canvasEl" />
    <p class="loading-caption" :style="captionStyle">{{ CAPTION_LABEL }}</p>
  </div>
</template>

<script setup>
import { onMounted, onUnmounted, ref } from 'vue'

const GRID_ROWS = 50
const BALL_RADIUS_CELLS = 1.8
const SPEED_PX_S = 800
const GAP_RATIO = 0.12
/** Higher = faster fade; lower = longer tail */
const TRAIL_FADE_RATE = 3.15
/** Per-tile fade multiplier spread: 1 ± this value */
const TRAIL_FADE_JITTER = 0.72
const GLOW_RADIUS_CELLS = 2.75
const GLOW_STRENGTH = 0.5
const CAPTION_ROWS = 3
const CAPTION_GAP_ROWS = 2
const CAPTION_LABEL = 'give us a sec :o'

const canvasEl = ref(null)
const captionStyle = ref({ visibility: 'hidden' })
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
  let trail = null
  let glow = null
  let fadeRates = null
  let trailCols = 0

  const ensureTrail = () => {
    if (trail && trailCols === cols) return
    trailCols = cols
    const n = cols * GRID_ROWS
    trail = new Float32Array(n)
    glow = new Float32Array(n)
    fadeRates = new Float32Array(n)
    for (let i = 0; i < n; i++) {
      fadeRates[i] = 1 + (Math.random() * 2 - 1) * TRAIL_FADE_JITTER
    }
  }

  const buildGlow = () => {
    glow.fill(0)
    const rMax = Math.ceil(GLOW_RADIUS_CELLS)
    for (let row = 0; row < GRID_ROWS; row++) {
      for (let col = 0; col < cols; col++) {
        const t = trail[row * cols + col]
        if (t < 0.015) continue
        const r0 = Math.max(0, row - rMax)
        const r1 = Math.min(GRID_ROWS - 1, row + rMax)
        const c0 = Math.max(0, col - rMax)
        const c1 = Math.min(cols - 1, col + rMax)
        for (let r = r0; r <= r1; r++) {
          for (let c = c0; c <= c1; c++) {
            const cellsDist = Math.hypot(c - col, r - row)
            if (cellsDist > GLOW_RADIUS_CELLS) continue
            const falloff = 1 - cellsDist / GLOW_RADIUS_CELLS
            const g = t * falloff * GLOW_STRENGTH
            const idx = r * cols + c
            if (g > glow[idx]) glow[idx] = g
          }
        }
      }
    }
  }

  const stampBall = (bx, by, radius) => {
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
        if (dx * dx + dy * dy <= r2) {
          trail[row * cols + col] = 1
        }
      }
    }
  }

  const stampSegment = (x0, y0, x1, y1, radius) => {
    const dist = Math.hypot(x1 - x0, y1 - y0)
    const step = Math.max(cell * 0.35, 1)
    const n = Math.max(1, Math.ceil(dist / step))
    for (let i = 0; i <= n; i++) {
      const t = i / n
      stampBall(x0 + (x1 - x0) * t, y0 + (y1 - y0) * t, radius)
    }
  }

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
    const fontSize = Math.min(cell * 1.14, cell * CAPTION_ROWS * 0.52)
    ctx.font = `400 ${fontSize}px Consolas, Avenir, Helvetica, Arial, sans-serif`
    const textW = ctx.measureText(CAPTION_LABEL).width
    const padCols = 2
    let colsWide = Math.ceil((textW + padCols * cell) / cell)
    colsWide = Math.min(Math.max(colsWide, 6), cols)
    let startCol = Math.floor((cols - colsWide) / 2)
    if (startCol < 0) startCol = 0

    captionStyle.value = {
      visibility: 'visible',
      left: `${startCol * cell}px`,
      width: `${colsWide * cell}px`,
      height: `${cell * CAPTION_ROWS}px`,
      bottom: `${cell * CAPTION_GAP_ROWS}px`,
      fontSize: `${fontSize}px`,
    }
  }

  const draw = (ts) => {
    if (!lastTs) lastTs = ts
    const dt = Math.min((ts - lastTs) / 1000, 0.05)
    lastTs = ts

    const bg = cssVar('--bg-app') || '#080607'
    const ballFill = cssVar('--accent') || '#c48a92'
    const glowFill = cssVar('--link-hover') || cssVar('--accent-hover') || '#e0b8be'
    const radius = cell * BALL_RADIUS_CELLS
    const gap = cell * GAP_RATIO
    const inset = gap / 2
    const tileSize = cell - gap

    const prevX = ball.x
    const prevY = ball.y
    ball = stepBounce(ball, dt, { left: 0, top: 0, right: width, bottom: height }, radius)

    ensureTrail()
    stampSegment(prevX, prevY, ball.x, ball.y, radius)

    for (let row = 0; row < GRID_ROWS; row++) {
      for (let col = 0; col < cols; col++) {
        const idx = row * cols + col
        const cx = col * cell + cell / 2
        const cy = row * cell + cell / 2
        const dx = cx - ball.x
        const dy = cy - ball.y
        const inside = dx * dx + dy * dy <= radius * radius
        if (inside) trail[idx] = 1
        else {
          trail[idx] *= Math.exp(-dt * TRAIL_FADE_RATE * fadeRates[idx])
        }
      }
    }
    buildGlow()

    ctx.fillStyle = bg
    ctx.fillRect(0, 0, width, height)

    for (let row = 0; row < GRID_ROWS; row++) {
      for (let col = 0; col < cols; col++) {
        const idx = row * cols + col
        const sx = col * cell + inset
        const sy = row * cell + inset

        ctx.fillStyle = bg
        ctx.fillRect(sx, sy, tileSize, tileSize)

        const halo = glow[idx]
        if (halo > 0.015) {
          ctx.globalAlpha = Math.min(0.65, halo)
          ctx.fillStyle = glowFill
          ctx.fillRect(sx, sy, tileSize, tileSize)
          ctx.globalAlpha = 1
        }

        const tail = trail[idx]
        if (tail > 0.02) {
          ctx.globalAlpha = tail
          ctx.fillStyle = ballFill
          ctx.fillRect(sx, sy, tileSize, tileSize)
          ctx.globalAlpha = 1
        }
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

.loading-caption {
  position: absolute;
  margin: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.35);
  backdrop-filter: blur(18px) saturate(140%);
  -webkit-backdrop-filter: blur(18px) saturate(140%);
  color: var(--text-muted);
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-weight: 400;
  line-height: 1;
  white-space: nowrap;
  z-index: 1;
  pointer-events: none;
  box-sizing: border-box;
}

canvas {
  display: block;
  width: 100%;
  height: 100%;
}
</style>
