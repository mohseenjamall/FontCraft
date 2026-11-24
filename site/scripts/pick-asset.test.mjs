import { test } from 'node:test';
import assert from 'node:assert/strict';
import { pickAsset } from '../src/lib/pick-asset.mjs';

const assets = [
  'FontCraft-1.2.0-windows-x64.zip',
  'FontCraft-1.2.0-windows-x64.exe',
  'FontCraft-1.2.0-linux-x64',
  'FontCraft-1.2.0-macos.dmg',
  'SHA256SUMS.txt',
].map((name) => ({ name }));

test('windows -> .exe, not .zip', () => assert.equal(pickAsset(assets, 'windows').name, 'FontCraft-1.2.0-windows-x64.exe'));
test('linux -> linux-x64', () => assert.equal(pickAsset(assets, 'linux').name, 'FontCraft-1.2.0-linux-x64'));
test('mac -> the one universal .dmg', () => assert.equal(pickAsset(assets, 'mac').name, 'FontCraft-1.2.0-macos.dmg'));
test('mac ignores old per-arch binaries', () => {
  const old = ['FontCraft-1.1.0-macos-arm64', 'FontCraft-1.1.0-macos-x64', 'FontCraft-1.1.0-macos.dmg.sig'].map((name) => ({ name }));
  assert.equal(pickAsset(old, 'mac'), undefined);
});
test('other -> undefined', () => assert.equal(pickAsset(assets, 'other'), undefined));
test('missing asset -> undefined', () => assert.equal(pickAsset(assets.filter((a) => !a.name.endsWith('.exe')), 'windows'), undefined));

test('null/undefined assets -> undefined', () => {
  assert.equal(pickAsset(undefined, 'windows'), undefined);
  assert.equal(pickAsset(null, 'linux'), undefined);
});

const withSetup = [
  'FontCraft-1.4.0-windows-x64-setup.exe',
  'FontCraft-1.4.0-windows-x64.exe',
  'FontCraft-1.4.0-windows-x64.zip',
  'SHA256SUMS.txt',
].map((name) => ({ name }));

test('windows prefers the setup file when both exist', () =>
  assert.equal(pickAsset(withSetup, 'windows').name, 'FontCraft-1.4.0-windows-x64-setup.exe'));

test('windows falls back to the portable exe when there is no setup file', () =>
  assert.equal(pickAsset(assets, 'windows').name, 'FontCraft-1.2.0-windows-x64.exe'));

test('setup file is never picked for linux or mac', () => {
  const only = [{ name: 'FontCraft-1.4.0-windows-x64-setup.exe' }];
  assert.equal(pickAsset(only, 'linux'), undefined);
  assert.equal(pickAsset(only, 'mac'), undefined);
});
