/** Fallback when API omits or breaks frontend configuration */
export const DEFAULT_SERVER_FE_CONFIG = {
  backgroundColor: '#0b090a',
  messageIconsSquare: true,
  messageFont: 'Inter, Avenir, Helvetica, Arial, sans-serif',
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

  const bg =
    parsed.backgroundColor ?? parsed.background ?? parsed.color ?? parsed.bg
  if (bg != null && String(bg).trim()) {
    config.backgroundColor = String(bg).trim()
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
  return {
    '--bg-chat': c.backgroundColor,
    backgroundColor: c.backgroundColor,
  }
}

if (import.meta.env.DEV) {
  const p = parseServerFeConfig('{"color":"red","messageIcons":"circle","font":"Georgia"}')
  console.assert(
    p.backgroundColor === 'red' && p.messageIconsSquare === false && p.messageFont === 'Georgia',
    'serverFeConfig parse self-check'
  )
}
