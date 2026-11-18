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
  TRAIL_FADE_RATE,
  createTrailState,
  ensureTrail,
  buildGlow,
  drawGrid,
  layoutCaption,
  setupCanvas,
  loadingColors,
} from './loadingCanvas.js'

const GOL_STEP_S = 0.11

const canvasEl = ref(null)
const captionStyle = ref({ visibility: 'hidden' })
let frameId = 0
let ro = null

function countNeighbors(grid, cols, rows, c, r) {
  let n = 0
  for (let dr = -1; dr <= 1; dr++) {
    for (let dc = -1; dc <= 1; dc++) {
      if (dr === 0 && dc === 0) continue
      const cc = c + dc
      const rr = r + dr
      if (cc >= 0 && cc < cols && rr >= 0 && rr < rows && grid[rr * cols + cc]) n++
    }
  }
  return n
}

function stepLife(grid, next, cols, rows) {
  for (let r = 0; r < rows; r++) {
    for (let c = 0; c < cols; c++) {
      const i = r * cols + c
      const neighbors = countNeighbors(grid, cols, rows, c, r)
      const alive = grid[i]
      next[i] =
        (alive && (neighbors === 2 || neighbors === 3)) || (!alive && neighbors === 3) ? 1 : 0
    }
  }
  let pop = 0
  for (let i = 0; i < next.length; i++) pop += next[i]
  return pop
}

/** Small live cluster centered on the viewport */
function seedCenterBurst(grid, cols, rows) {
  const cx = (cols * 0.5) | 0
  const cy = (rows * 0.5) | 0
  for (let r = cy - 4; r <= cy + 4; r++) {
    for (let c = cx - 5; c <= cx + 5; c++) {
      if (c < 0 || c >= cols || r < 0 || r >= rows) continue
      if (Math.random() < 0.38) grid[r * cols + c] = 1
    }
  }
}

function seedLife(grid, cols, rows) {
  grid.fill(0)
  seedCenterBurst(grid, cols, rows)
}

function seedPatch(grid, cols, rows) {
  const cx = (Math.random() * cols) | 0
  const cy = (Math.random() * rows) | 0
  const rad = 4 + ((Math.random() * 6) | 0)
  for (let r = cy - rad; r <= cy + rad; r++) {
    for (let c = cx - rad; c <= cx + rad; c++) {
      if (c < 0 || c >= cols || r < 0 || r >= rows) continue
      if (Math.random() < 0.4) grid[r * cols + c] = 1
    }
  }
}

function fadeLifeTrail(trailState, grid, dt) {
  const { trail, fadeRates } = trailState
  for (let i = 0; i < grid.length; i++) {
    if (grid[i]) trail[i] = 1
    else trail[i] *= Math.exp(-dt * TRAIL_FADE_RATE * fadeRates[i])
  }
}

if (import.meta.env.DEV) {
  const g = Uint8Array.from([0, 1, 0, 0, 1, 1, 0, 1, 0])
  const n = new Uint8Array(g.length)
  stepLife(g, n, 3, 3)
  console.assert(n[4] === 1, 'loading life blinker self-check')
}

onMounted(() => {
  const canvas = canvasEl.value
  if (!canvas) return
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const trailState = createTrailState()
  let grid = null
  let nextGrid = null
  let lifeCols = 0
  let golAcc = 0
  let lastTs = 0
  const { resize, getMetrics } = setupCanvas(canvas, ctx)

  const ensureLifeGrid = (cols, cell) => {
    if (cols <= 0) return false
    ensureTrail(trailState, cols)
    if (cols !== lifeCols || !grid || grid.length !== cols * GRID_ROWS) {
      lifeCols = cols
      grid = new Uint8Array(cols * GRID_ROWS)
      nextGrid = new Uint8Array(cols * GRID_ROWS)
      seedLife(grid, cols, GRID_ROWS)
      for (let i = 0; i < grid.length; i++) {
        if (grid[i]) trailState.trail[i] = 1
      }
    }
    layoutCaption(ctx, captionStyle, { cols, cell })
    return true
  }

  const onLayout = ({ cols, cell }) => {
    ensureLifeGrid(cols, cell)
  }

  const draw = (ts) => {
    if (!lastTs) lastTs = ts
    const dt = Math.min((ts - lastTs) / 1000, 0.05)
    lastTs = ts

    const { width, height, cell, cols } = getMetrics()
    if (!ensureLifeGrid(cols, cell)) {
      frameId = requestAnimationFrame(draw)
      return
    }

    golAcc += dt
    if (golAcc >= GOL_STEP_S) {
      golAcc = 0
      const pop = stepLife(grid, nextGrid, cols, GRID_ROWS)
      const tmp = grid
      grid = nextGrid
      nextGrid = tmp
      if (pop < 12) {
        seedPatch(grid, cols, GRID_ROWS)
        if (pop < 4) seedCenterBurst(grid, cols, GRID_ROWS)
      }
    }

    fadeLifeTrail(trailState, grid, dt)
    buildGlow(trailState)
    drawGrid(ctx, trailState, { width, height, cell, colors: loadingColors() })

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
