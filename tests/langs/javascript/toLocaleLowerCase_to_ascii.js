const locale = new Intl.Collator().resolvedOptions().locale;
console.log(`Characters which String.prototype.toLocaleLowerCase maps to ASCII with locale ${locale}:`);
for (let code = 0x80; code <= 0x10FFFF; code++) {
  const ch = String.fromCodePoint(code);
  const lower = [...ch.toLocaleLowerCase()];
  if (lower.every(ch => ch.codePointAt(0) <= 0x7f)) {
    console.log(`${ch} -> ${lower}`);
  }
}
