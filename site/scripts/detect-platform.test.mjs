import { test } from 'node:test';
import assert from 'node:assert/strict';
import { detectPlatform } from '../src/lib/detect-platform.mjs';

const d = (o) => detectPlatform(o);
test('windows', () => {
  assert.equal(d({ uaPlatform: 'Windows' }), 'windows');
  assert.equal(d({ userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120' }), 'windows');
});
test('mac', () => assert.equal(d({ userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) Safari/605', maxTouchPoints: 0 }), 'mac'));
test('linux', () => assert.equal(d({ userAgent: 'Mozilla/5.0 (X11; Linux x86_64) Firefox/120' }), 'linux'));
test('iPhone -> other', () => assert.equal(d({ userAgent: 'Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) Safari', uaPlatform: 'iOS' }), 'other'));
test('iPad as Mac -> other', () => assert.equal(d({ userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) Safari', maxTouchPoints: 5 }), 'other'));
test('Android -> other', () => assert.equal(d({ userAgent: 'Mozilla/5.0 (Linux; Android 14; Pixel 8) Chrome/120 Mobile' }), 'other'));
test('CrOS -> other', () => {
  assert.equal(d({ userAgent: 'Mozilla/5.0 (X11; CrOS x86_64 14541.0.0) Chrome/120' }), 'other');
  assert.equal(d({ uaPlatform: 'Chrome OS', userAgent: 'Mozilla/5.0 (X11; Linux x86_64) Chrome' }), 'other');
});
