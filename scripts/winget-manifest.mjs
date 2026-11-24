// Writes the three winget manifest files for a release.
// Usage: node scripts/winget-manifest.mjs <version> <path to that release's SHA256SUMS.txt>
// Output: packaging/winget/<version>/ ; copy it to manifests/m/mohseenjamall/FontCraft/<version>/ in a
// microsoft/winget-pkgs fork and open a pull request (see README, "Publishing an update").
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";

const [version, sums] = process.argv.slice(2);
if (!/^\d+\.\d+\.\d+$/.test(version ?? "") || !sums) {
  console.log("usage: node scripts/winget-manifest.mjs <version> <SHA256SUMS.txt>");
  process.exit(1);
}
const file = `FontCraft-${version}-windows-x64.exe`;
const line = readFileSync(sums, "utf8").split(/\r?\n/).find((l) => l.trim().endsWith(`  ${file}`));
const sha = line?.split(/\s+/)[0]?.toUpperCase();
if (!sha || !/^[0-9A-F]{64}$/.test(sha)) {
  console.log(`no SHA-256 for ${file} in ${sums}`);
  process.exit(1);
}
const id = "mohseenjamall.FontCraft";
const schema = (kind) => `# yaml-language-server: $schema=https://aka.ms/winget-manifest.${kind}.1.6.0.schema.json\n`;
const head = `PackageIdentifier: ${id}\nPackageVersion: ${version}\n`;
const dir = `packaging/winget/${version}`;
mkdirSync(dir, { recursive: true });
writeFileSync(`${dir}/${id}.yaml`, `${schema("version")}${head}DefaultLocale: en-US\nManifestType: version\nManifestVersion: 1.6.0\n`);
writeFileSync(
  `${dir}/${id}.locale.en-US.yaml`,
  `${schema("defaultLocale")}${head}PackageLocale: en-US
Publisher: mohseenjamall
PublisherUrl: https://github.com/mohseenjamall
PackageName: FontCraft
PackageUrl: https://github.com/mohseenjamall/FontCraft
License: MIT
LicenseUrl: https://github.com/mohseenjamall/FontCraft/blob/main/LICENSE
ShortDescription: A fast font browser. Preview, search and turn fonts on and off.
Tags:
- font
- fonts
- typography
- font-manager
ReleaseNotesUrl: https://github.com/mohseenjamall/FontCraft/releases/tag/v${version}
ManifestType: defaultLocale
ManifestVersion: 1.6.0
`,
);
writeFileSync(
  `${dir}/${id}.installer.yaml`,
  `${schema("installer")}${head}InstallerType: portable
Commands:
- fontcraft
Installers:
- Architecture: x64
  InstallerUrl: https://github.com/mohseenjamall/FontCraft/releases/download/v${version}/${file}
  InstallerSha256: ${sha}
ManifestType: installer
ManifestVersion: 1.6.0
`,
);
console.log(`winget manifest for ${version} written to ${dir}`);
