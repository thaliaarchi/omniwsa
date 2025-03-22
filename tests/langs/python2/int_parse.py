import platform

print "Python", platform.python_version()

def value_before_space(c):
  try:
    n = int("%s 1" % chr(c))
  except:
    return None
  if n == -1:
    return "negative"
  assert n == 1
  return "space"

def value_before_digit(c):
  try:
    n = int("%s2" % chr(c))
  except:
    return None
  if n == -2:
    return "negative"
  x, y = divmod(n, 10)
  assert y == 2
  return x

def value_between_digits(c):
  try:
    n = int("1%s3" % chr(c))
  except:
    return None
  xy, z = divmod(n, 10)
  assert z == 3
  if xy == 1:
    return "underscore"
  x, y = divmod(xy, 10)
  assert x == 1
  return y

def value_after_digit(c):
  try:
    n = int("1%s" % chr(c))
  except:
    return None
  if n == 1:
    return "space"
  x, y = divmod(n, 10)
  assert x == 1
  return y

def value_after_space(c):
  try:
    n = int("1 %s" % chr(c))
  except:
    return None
  assert n == 1
  return "space"

for c in range(0, 0x110000):
  before_space = value_before_space(c)
  before_digit = value_before_digit(c)
  between_digits = value_between_digits(c)
  after_digit = value_after_digit(c)
  after_space = value_after_space(c)
  if before_digit == None and between_digits == None and after_digit == None \
      and before_space == None and after_space == None:
    continue
  if before_digit == between_digits and before_digit == after_digit \
      and before_space == None and after_space == None:
    value = before_digit
  elif before_digit == 0 and between_digits == None and after_digit == "space" \
      and before_space == "space" and after_space == "space":
    value = "space"
  else:
    value = (before_space, before_digit, between_digits, after_digit, after_space)
  print "U+%04X %s %s" % (c, chr(c), value)
