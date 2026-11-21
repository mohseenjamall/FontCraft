// Checks the font-activation docs: README feature bullet present and not listed as missing,
// MECHANICS.md documents FontCraft's own registry marker. Run from the app folder.
import { readFileSync } from "node:fs";

const readme = readFileSync("README.md", "utf8");
const mechanics = readFileSync("MECHANICS.md", "utf8");
const notIn = readme.split(/^## Not in version 1/m)[1] ?? "";
const problems = [];
if (!/^## Not in version 1/m.test(readme)) problems.push("README: 'Not in version 1' section missing");
if (/turning fonts on and off/i.test(notIn)) problems.push("README still lists turning fonts on and off as missing");
if (!/\*\*Turn fonts on and off\.\*\*/.test(readme)) problems.push("README: no 'Turn fonts on and off' feature bullet");
if (!/\(FontCraft\)/.test(mechanics)) problems.push("MECHANICS.md: no (FontCraft) marker row");
if (!/never (edits|touches)[^\n]*\(FontBase\)/i.test(mechanics)) problems.push("MECHANICS.md: doesn't say FontBase values are left alone");
if (problems.length) {
  console.log(problems.join("\n"));
  process.exit(1);
}
console.log("docs ok");
