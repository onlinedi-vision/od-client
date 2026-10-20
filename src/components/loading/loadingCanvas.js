export const GRID_ROWS = 62
export const GAP_RATIO = 0.12
export const TRAIL_FADE_RATE = 3.15
export const TRAIL_FADE_JITTER = 0.72
export const GLOW_RADIUS_CELLS = 2.75
export const GLOW_STRENGTH = 0.54
export const CAPTION_ROWS = 3
export const CAPTION_GAP_ROWS = 2
export const CAPTION_LABEL = 'give us a sec :o'

export function cssVar(name) {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim()
}

export function createTrailState() {
  return {
    trail: null,
    glow: null,
    fadeRates: null,
    trailCols: 0,
  }
}

export function ensureTrail(state, cols, rows = GRID_ROWS) {
  if (state.trail && state.trailCols === cols) return
  state.trailCols = cols
  const n = cols * rows
  state.trail = new Float32Array(n)
  state.glow = new Float32Array(n)
  state.fadeRates = new Float32Array(n)
  for (let i = 0; i < n; i++) {
    state.fadeRates[i] = 1 + (Math.random() * 2 - 1) * TRAIL_FADE_JITTER
  }
}

export function buildGlow(state, rows = GRID_ROWS) {
  const { trail, glow, trailCols: cols } = state
  glow.fill(0)
  const rMax = Math.ceil(GLOW_RADIUS_CELLS)
  for (let row = 0; row < rows; row++) {
    for (let col = 0; col < cols; col++) {
      const t = trail[row * cols + col]
      if (t < 0.015) continue
      const r0 = Math.max(0, row - rMax)
      const r1 = Math.min(rows - 1, row + rMax)
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

export function fadeTrail(state, dt, rows = GRID_ROWS, keepBright) {
  const { trail, fadeRates, trailCols: cols } = state
  for (let row = 0; row < rows; row++) {
    for (let col = 0; col < cols; col++) {
      const idx = row * cols + col
      if (keepBright?.(idx, col, row)) trail[idx] = 1
      else trail[idx] *= Math.exp(-dt * TRAIL_FADE_RATE * fadeRates[idx])
    }
  }
}

export function drawGrid(ctx, state, { width, height, cell, rows = GRID_ROWS, colors }) {
  const { trail, glow, trailCols: cols } = state
  const { bg, fill, glowFill } = colors
  const gap = cell * GAP_RATIO
  const inset = gap / 2
  const tileSize = cell - gap

  ctx.fillStyle = bg
  ctx.fillRect(0, 0, width, height)

  for (let row = 0; row < rows; row++) {
    for (let col = 0; col < cols; col++) {
      const idx = row * cols + col
      const sx = col * cell + inset
      const sy = row * cell + inset

      ctx.fillStyle = bg
      ctx.fillRect(sx, sy, tileSize, tileSize)

      const halo = glow[idx]
      if (halo > 0.015) {
        ctx.globalAlpha = Math.min(0.72, halo * 1.1)
        ctx.fillStyle = glowFill
        ctx.fillRect(sx, sy, tileSize, tileSize)
        ctx.globalAlpha = 1
      }

      const tail = trail[idx]
      if (tail > 0.02) {
        ctx.globalAlpha = Math.min(1, tail * 1.05)
        ctx.fillStyle = fill
        ctx.fillRect(sx, sy, tileSize, tileSize)
        ctx.globalAlpha = 1
      }
    }
  }
}

export function layoutCaption(ctx, captionStyle, { cols, cell, label = CAPTION_LABEL }) {
  const fontSize = Math.min(cell * 1.14, cell * CAPTION_ROWS * 0.52)
  ctx.font = `400 ${fontSize}px Consolas, Avenir, Helvetica, Arial, sans-serif`
  const textW = ctx.measureText(label).width
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

export function setupCanvas(canvas, ctx) {
  let width = 0
  let height = 0
  let cell = 0
  let cols = 0

  const resize = (onLayout) => {
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
    onLayout?.({ width, height, cell, cols })
    return { width, height, cell, cols }
  }

  return { resize, getMetrics: () => ({ width, height, cell, cols }) }
}

export function loadingColors() {
  const accent = cssVar('--accent') || '#c48a92'
  const accentHover = cssVar('--accent-hover') || '#d49aa2'
  return {
    bg: cssVar('--bg-app') || '#080607',
    fill: accent,
    glowFill: accentHover,
  }
}
