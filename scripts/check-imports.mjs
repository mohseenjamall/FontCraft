// Fails if a Windows program imports a C runtime DLL (VCRUNTIME*, MSVCP*, api-ms-win-crt-*),
// i.e. it would need the Visual C++ Redistributable. Usage: node check-imports.mjs <file.exe>
import { readFileSync } from "node:fs";

const file = process.argv[2];
if (!file) {
  console.error("usage: node check-imports.mjs <file.exe>");
  process.exit(2);
}
const b = readFileSync(file);
const pe = b.readUInt32LE(0x3c);
if (b.toString("latin1", pe, pe + 4) !== "PE\0\0") throw new Error(`${file}: not a PE file`);
const sections = b.readUInt16LE(pe + 6);
const optSize = b.readUInt16LE(pe + 20);
const opt = pe + 24;
const magic = b.readUInt16LE(opt);
const dirs = opt + (magic === 0x20b ? 112 : 96);
const importRva = b.readUInt32LE(dirs + 8);
const secTable = opt + optSize;

function offsetOf(rva) {
  for (let i = 0; i < sections; i++) {
    const s = secTable + i * 40;
    const va = b.readUInt32LE(s + 12);
    const size = Math.max(b.readUInt32LE(s + 8), b.readUInt32LE(s + 16));
    if (rva >= va && rva < va + size) return rva - va + b.readUInt32LE(s + 20);
  }
  throw new Error(`rva 0x${rva.toString(16)} not in any section`);
}

const names = [];
for (let d = offsetOf(importRva); b.readUInt32LE(d + 12) !== 0; d += 20) {
  const start = offsetOf(b.readUInt32LE(d + 12));
  names.push(b.toString("latin1", start, b.indexOf(0, start)));
}
if (names.length === 0) throw new Error("no imports found: parser problem");
const crt = names.filter((n) => /^(vcruntime|msvcp|api-ms-win-crt-)/i.test(n));
console.log(`${names.length} DLLs: ${names.join(", ")}`);
if (crt.length) {
  console.log(`needs C runtime: ${crt.join(", ")}`);
  process.exit(1);
}
console.log("imports ok: no C runtime DLL");
