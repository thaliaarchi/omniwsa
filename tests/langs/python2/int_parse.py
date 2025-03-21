import platform

print "Python", platform.python_version()

print "Digit values in int(str):"

for c in range(0, 256):
  n = None
  try:
    n = int("1%s3" % chr(c))
  except:
    continue
  xy, z = divmod(n, 10)
  assert z == 3
  if xy == 1:
    print "U+%04X" % c, chr(c), "underscore"
    continue
  x, y = divmod(xy, 10)
  assert x == 1
  print "U+%04X" % c, chr(c), y
