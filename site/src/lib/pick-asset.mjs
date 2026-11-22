// Picks the release asset for a visitor's OS. platform: 'windows'|'mac'|'linux'|'other'.
// macOS has one universal .dmg (Apple Silicon and Intel).
const PATTERNS = {
  windows: /windows-x64\.exe$/,
  linux: /linux-x64$/,
  mac: /-macos\.dmg$/,
};

export function pickAsset(assets, platform) {
  const re = PATTERNS[platform];
  return re ? assets?.find((a) => re.test(a.name)) : undefined;
}
