import platform

print(f"Python {platform.python_version()}")

print("Digit values in int(str):")

for c in range(0, 0x110000):
  n = None
  try:
    n = int(f"1{chr(c)}3")
  except:
    continue
  xy, z = divmod(n, 10)
  assert z == 3
  if xy == 1:
    print(f"U+{c:04X}", chr(c), "underscore")
    continue
  x, y = divmod(xy, 10)
  assert x == 1
  print(f"U+{c:04X}", chr(c), y)
