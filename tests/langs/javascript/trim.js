console.log("Whitespace characters for String.prototype.trim:");
for (let code = 0; code <= 0x10FFFF; code++) {
  const ch = String.fromCodePoint(code);
  if (ch.trim() !== ch) {
    console.log(`U+${code.toString(16).toUpperCase().padStart(4, "0")}`);
  }
}
