// Fails when the Cargo.toml version has no "## {version}" section with 1 to 6 bullets in assets/whats-new.md.
// Usage: node scripts/check-whats-new.mjs [folder]   (default: the current folder)
import { readFileSync } from "node:fs";
import { join } from "node:path";

const dir = process.argv[2] ?? ".";
const version = /^version\s*=\s*"([^"]+)"/m.exec(readFileSync(join(dir, "Cargo.toml"), "utf8"))?.[1];
if (!version) {
  console.log("whats-new: no version in Cargo.toml");
  process.exit(1);
}
const lines = readFileSync(join(dir, "assets/whats-new.md"), "utf8").split(/\r?\n/);
const start = lines.findIndex((l) => l.trim() === `## ${version}`);
if (start < 0) {
  console.log(`whats-new: assets/whats-new.md has no "## ${version}" section. Add it with the release notes.`);
  process.exit(1);
}
const rest = lines.slice(start + 1);
const end = rest.findIndex((l) => l.trim().startsWith("## "));
const bullets = (end < 0 ? rest : rest.slice(0, end)).filter((l) => l.trim().startsWith("- "));
if (bullets.length < 1 || bullets.length > 6) {
  console.log(`whats-new: "## ${version}" has ${bullets.length} bullets; it needs 1 to 6.`);
  process.exit(1);
}
console.log("whats-new ok");
