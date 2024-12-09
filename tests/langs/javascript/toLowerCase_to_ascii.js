console.log("Characters which String.prototype.toLowerCase maps to ASCII:");
for (let code = 0x80; code <= 0x10FFFF; code++) {
  const ch = String.fromCodePoint(code);
  const lower = [...ch.toLowerCase()];
  if (lower.every(ch => ch.codePointAt(0) <= 0x7f)) {
    console.log(`${ch} -> ${lower}`);
  }
}
