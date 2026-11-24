// Total downloads across GitHub releases (the API's `download_count`), counting only real downloads.
const NOT_A_DOWNLOAD = /^SHA256SUMS|\.sig$/;

export function sumDownloads(releases) {
  if (!Array.isArray(releases)) return 0;
  let total = 0;
  for (const r of releases) {
    if (!Array.isArray(r?.assets)) continue;
    for (const a of r.assets) {
      const n = a?.download_count;
      if (!NOT_A_DOWNLOAD.test(a?.name ?? '') && Number.isFinite(n) && n >= 0) total += n;
    }
  }
  return total;
}

// 999 -> "999", 1234 -> "1.2K", 1250000 -> "1.3M"
export function formatCount(n) {
  if (n < 1000) return String(n);
  const [v, unit] = n < 1e6 ? [n / 1e3, 'K'] : [n / 1e6, 'M'];
  return v.toFixed(1).replace(/\.0$/, '') + unit;
}
