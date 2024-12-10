import platform
print "Python %s" % platform.python_version()

def expect_except(func):
    try: func()
    except Exception: pass
    else: raise Exception("expected exception")

expect_except(lambda: chr(-1))
expect_except(lambda: chr(256))
print [chr(c) for c in range(0, 256) if len(chr(c).split()) == 0]
