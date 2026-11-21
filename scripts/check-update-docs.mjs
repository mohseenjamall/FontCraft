// Checks the update-notice docs. Run from the app folder.
import { existsSync, readFileSync } from "node:fs";

const readme = readFileSync("README.md", "utf8");
const mechanics = readFileSync("MECHANICS.md", "utf8");
const problems = [];
const need = (ok, msg) => ok || problems.push(msg);
need(/\*\*Update notice\.\*\*/.test(readme), "README: no 'Update notice' feature bullet");
need(/^## Publishing an update/m.test(readme), "README: no 'Publishing an update' section");
need(/releases\/latest/.test(readme) && /public/i.test(readme), "README: doesn't say releases must be public");
need(/FONTCRAFT_LATEST/.test(readme), "README: doesn't explain FONTCRAFT_LATEST for testing");
need(/^### Update notice/m.test(mechanics), "MECHANICS.md: no 'Update notice' section");
need(/update::trusted/.test(mechanics) && /update::is_newer/.test(mechanics), "MECHANICS.md: rules not tied to code");
need(existsSync("docs/plans/2026-11-21-update-notice.md"), "docs/plans: update-notice plan missing");
if (problems.length) {
  console.log(problems.join("\n"));
  process.exit(1);
}
console.log("update docs ok");
