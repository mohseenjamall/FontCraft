// Checks the font on/off docs match the current rules. Run from the app folder.
import { existsSync, readFileSync } from "node:fs";

const readme = readFileSync("README.md", "utf8");
const mechanics = readFileSync("MECHANICS.md", "utf8");
const notIn = readme.split(/^## Not in version 1/m)[1] ?? "";
const problems = [];
const need = (ok, msg) => ok || problems.push(msg);
need(/^## Not in version 1/m.test(readme), "README: 'Not in version 1' section missing");
need(!/turning fonts on and off/i.test(notIn), "README still lists turning fonts on and off as missing");
need(/\*\*Turn fonts on and off\.\*\*/.test(readme), "README: no 'Turn fonts on and off' feature bullet");
need(/installed for (you|your user)/i.test(readme) && /FontBase/.test(readme), "README: doesn't say user-installed and FontBase fonts can be turned off");
need(/needs administrator/i.test(readme), "README: doesn't explain system fonts need administrator");
need(/\(FontCraft\)/.test(mechanics), "MECHANICS.md: no (FontCraft) marker row");
need(/disabled\.tsv/.test(mechanics), "MECHANICS.md: saved list disabled.tsv not documented");
need(/byte for byte/i.test(mechanics), "MECHANICS.md: doesn't say values come back byte for byte");
need(/never (writes|touches) HKLM/i.test(mechanics), "MECHANICS.md: doesn't say HKLM is never written");
need(/never delete[sd]? (a |any )?font file/i.test(mechanics), "MECHANICS.md: doesn't say font files are never deleted");
need(existsSync("docs/plans/2026-11-21-toggle-installed-fonts.md"), "docs/plans: plan for this change missing");
if (problems.length) {
  console.log(problems.join("\n"));
  process.exit(1);
}
console.log("docs ok");
