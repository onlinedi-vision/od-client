/** Fallback when parsing partial API config */
export const DEFAULT_SERVER_FE_CONFIG = {
  backgroundColor: '#0b090a',
  backgroundImage: '',
  messageIconsSquare: true,
  messageFont: 'Inter, Avenir, Helvetica, Arial, sans-serif',
}

const DEMO_BACKGROUNDS = [
  '#0b090a',
  '#120a10',
  '#0a0f12',
  '#140c0a',
  '#0d0a14',
  '#101208',
  '#160e12',
  '#0a1210',
]

const DEMO_WALLPAPERS = [
  '',
  '',
  '',
  'https://media1.tenor.com/m/viIU4ICp1N8AAAAd/dance.gif',
]

const DEMO_FONTS = [
  'Inter, Avenir, Helvetica, Arial, sans-serif',
  'Georgia, "Times New Roman", serif',
  '"Palatino Linotype", Palatino, serif',
  'Consolas, "Courier New", monospace',
  'system-ui, sans-serif',
  '"Segoe UI", Tahoma, sans-serif',
]

function hashServerId(serverId) {
  const s = String(serverId ?? '')
  let h = 2166136261
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i)
    h = Math.imul(h, 16777619)
  }
  return h >>> 0
}

function looksLikeBackgroundMediaUrl(value) {
  const v = String(value ?? '').trim()
  if (!v) return false
  if (/^https?:\/\//i.test(v) || /^data:image\//i.test(v)) return true
  return /\.(gif|png|jpe?g|webp|avif|bmp|svg)(\?|#|$)/i.test(v)
}

function cssUrl(value) {
  const escaped = String(value).replace(/\\/g, '\\\\').replace(/"/g, '\\"')
  return `url("${escaped}")`
}

/** Stable demo defaults when the API has no fe_config */
export function defaultFeConfigForServer(serverId) {
  const h = hashServerId(serverId)
  const backgroundImage = DEMO_WALLPAPERS[h % DEMO_WALLPAPERS.length]
  return {
    backgroundColor: DEMO_BACKGROUNDS[h % DEMO_BACKGROUNDS.length],
    backgroundImage,
    messageIconsSquare: (h & 4) === 0,
    messageFont: DEMO_FONTS[h % DEMO_FONTS.length],
  }
}

export function resolveServerFeConfig(feConfigRaw, serverId) {
  if (feConfigRaw == null || !String(feConfigRaw).trim()) {
    return defaultFeConfigForServer(serverId)
  }
  const trimmed = String(feConfigRaw).trim()
  try {
    JSON.parse(trimmed)
  } catch {
    return defaultFeConfigForServer(serverId)
  }
  return parseServerFeConfig(feConfigRaw)
}

export function parseServerFeConfig(raw) {
  const config = { ...DEFAULT_SERVER_FE_CONFIG }
  if (raw == null || raw === '') return config
  let parsed = raw
  if (typeof raw === 'string') {
    const trimmed = raw.trim()
    if (!trimmed) return config
    try {
      parsed = JSON.parse(trimmed)
    } catch {
      return config
    }
  }
  if (typeof parsed !== 'object' || parsed === null) return config

  const explicitImage =
    parsed.backgroundImage ??
    parsed.backgroundUrl ??
    parsed.bgImage ??
    parsed.wallpaper ??
    parsed.background_image
  if (explicitImage != null && looksLikeBackgroundMediaUrl(explicitImage)) {
    config.backgroundImage = String(explicitImage).trim()
  }

  const bg =
    parsed.backgroundColor ?? parsed.background ?? parsed.color ?? parsed.bg
  if (bg != null && String(bg).trim()) {
    const bgStr = String(bg).trim()
    if (looksLikeBackgroundMediaUrl(bgStr)) {
      config.backgroundImage = bgStr
    } else {
      config.backgroundColor = bgStr
    }
  }

  const icons =
    parsed.messageIcons ?? parsed.avatarShape ?? parsed.message_icon_shape
  if (icons === 'circle' || icons === 'circles' || icons === 'round') {
    config.messageIconsSquare = false
  } else if (icons === 'square' || icons === 'squares') {
    config.messageIconsSquare = true
  } else if (typeof parsed.messageIconsSquare === 'boolean') {
    config.messageIconsSquare = parsed.messageIconsSquare
  }

  const font = parsed.messageFont ?? parsed.font ?? parsed.message_font
  if (font != null && String(font).trim()) {
    config.messageFont = String(font).trim()
  }

  return config
}

export function chatColumnStyleFromFeConfig(feConfig) {
  const c = feConfig ?? DEFAULT_SERVER_FE_CONFIG
  const style = {
    backgroundColor: c.backgroundColor,
    '--bg-chat': c.backgroundImage ? 'transparent' : c.backgroundColor,
  }
  if (c.backgroundImage) {
    style.backgroundImage = cssUrl(c.backgroundImage)
    style.backgroundSize = 'cover'
    style.backgroundPosition = 'center'
    style.backgroundRepeat = 'no-repeat'
  }
  return style
}

if (import.meta.env.DEV) {
  const img = parseServerFeConfig(
    '{"backgroundImage":"https://example.com/a.gif","color":"#111"}'
  )
  console.assert(
    img.backgroundImage === 'https://example.com/a.gif' && img.backgroundColor === '#111',
    'serverFeConfig image parse self-check'
  )
}
