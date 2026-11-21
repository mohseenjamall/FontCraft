// Runs the live tests on the real per-user font key, then checks with reg.exe that nothing from
// FontCraft or its tests was left behind. Run from the app folder.
import { spawnSync } from "node:child_process";

const run = (cmd, args) => spawnSync(cmd, args, { encoding: "utf8", shell: true });
const t = run("cargo", ["test", "--release", "live_", "--", "--ignored", "--test-threads=1", "--nocapture"]);
const testOut = `${t.stdout}${t.stderr}`;
const problems = [];
if (t.status !== 0) problems.push(`live tests failed (exit ${t.status})`);
for (const marker of ["live activation ok", "live user-installed ok"]) {
  if (!testOut.includes(marker)) problems.push(`missing "${marker}"`);
}
const fonts = run("reg", ["query", '"HKCU\\Software\\Microsoft\\Windows NT\\CurrentVersion\\Fonts"']);
if (fonts.status !== 0) problems.push("reg query of the font key failed");
const leftovers = fonts.stdout.split(/\r?\n/).filter((l) => /fontcraft/i.test(l));
if (leftovers.length) problems.push(`left in the font key:\n${leftovers.join("\n")}`);
const scratch = run("reg", ["query", '"HKCU\\Software\\FontCraftTest"']);
if (scratch.status === 0) problems.push("scratch key HKCU\\Software\\FontCraftTest still exists");
console.log(testOut.split(/\r?\n/).filter((l) => /^test |test result|live .* ok/.test(l)).join("\n"));
if (problems.length) {
  console.log(problems.join("\n"));
  process.exit(1);
}
console.log("live ok, registry clean");
