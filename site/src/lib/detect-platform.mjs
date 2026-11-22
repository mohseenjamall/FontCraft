// platform: 'windows'|'mac'|'linux'|'other'. iOS/iPadOS, Android and ChromeOS are 'other'.
export function detectPlatform({ uaPlatform = '', userAgent = '', maxTouchPoints = 0 } = {}) {
  const ua = String(userAgent).toLowerCase();
  const p = String(uaPlatform || userAgent).toLowerCase();
  if (/iphone|ipad|ipod|android|cros/.test(ua) || /cros|chrome os/.test(p)) return 'other';
  if (/win/.test(p)) return 'windows';
  if (/mac/.test(p)) return maxTouchPoints > 1 ? 'other' : 'mac'; // iPadOS reports as Macintosh
  if (/linux|x11/.test(p)) return 'linux';
  return 'other';
}
