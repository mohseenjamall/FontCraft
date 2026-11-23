import { test } from 'node:test';
import assert from 'node:assert/strict';
import { sumDownloads, formatCount } from '../src/lib/sum-downloads.mjs';

const rel = (...assets) => ({ assets: assets.map(([name, download_count]) => ({ name, download_count })) });

test('sums every installable asset across releases', () =>
  assert.equal(sumDownloads([rel(['FontCraft-1.2.0-windows-x64.exe', 100], ['FontCraft-1.2.0-macos.dmg', 20]), rel(['FontCraft-1.1.0-windows-x64.zip', 5])]), 125));
test('ignores checksum and signature files', () =>
  assert.equal(sumDownloads([rel(['SHA256SUMS.txt', 900], ['FontCraft-1.2.0-macos.dmg.sig', 900], ['FontCraft-1.2.0-linux-x64', 3])]), 3));
test('bad download_count counts as 0, never NaN', () =>
  assert.equal(sumDownloads([rel(['a.exe', undefined], ['b.exe', '7'], ['c.exe', -4], ['d.exe', NaN], ['e.exe', 2])]), 2));
test('non-array, null, or release without assets -> 0', () => {
  assert.equal(sumDownloads(null), 0);
  assert.equal(sumDownloads({ message: 'rate limited' }), 0);
  assert.equal(sumDownloads([{}, { assets: null }, null]), 0);
});
test('formatCount', () => {
  assert.equal(formatCount(0), '0');
  assert.equal(formatCount(999), '999');
  assert.equal(formatCount(1234), '1.2K');
  assert.equal(formatCount(12500), '12.5K');
  assert.equal(formatCount(1250000), '1.3M');
});
