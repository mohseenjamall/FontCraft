// Runs `cargo build --release` and fails on any compiler warning. Run from the app folder.
import { spawnSync } from "node:child_process";

const r = spawnSync("cargo", ["build", "--release"], { encoding: "utf8", shell: true });
const out = `${r.stdout}${r.stderr}`;
const warnings = out.split(/\r?\n/).filter((l) => /^warning[:\[]/.test(l) && !/generated \d+ warning/.test(l));
console.log(out.trim().split(/\r?\n/).slice(-3).join("\n"));
if (r.status !== 0) {
  console.log(`build failed (exit ${r.status})`);
  process.exit(1);
}
if (warnings.length) {
  console.log(`${warnings.length} warning(s):\n${warnings.join("\n")}`);
  process.exit(1);
}
console.log("release build ok, 0 warnings");
