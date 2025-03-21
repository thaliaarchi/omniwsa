import platform

print(f"Python {platform.python_version()}")

def expect_exception(f):
    try: f()
    except: pass
    else: raise Exception("expected exception")

expect_exception(lambda: chr(-1))
expect_exception(lambda: chr(0x110000))
print("split:", [chr(c) for c in range(0, 0x110000) if len(chr(c).split()) == 0])
print("strip:", [chr(c) for c in range(0, 0x110000) if chr(c).strip() == ""])
