# These remain unchanged
str(1)
str(*a)
str("foo", *a)
str(**k)
str("foo", **k)
str("foo", encoding="UTF-8")
str("foo"
    "bar")
str(b"foo")
bytes("foo", encoding="UTF-8")
bytes(*a)
bytes("foo", *a)
bytes("foo", **a)
bytes(b"foo"
      b"bar")
bytes("foo")
bytes(1)
f"{f'{str()}'}"
int(1.0)
int("1")
int(b"11")
int(10, base=2)
int("10", base=2)
int("10", 2)
float("1.0")
float(b"1.0")
bool(1)
bool(0)
bool("foo")
bool("")
bool(b"")
bool(1.0)
int().denominator

# These become literals
str()
str("foo")
str("""
foo""")
bytes()
bytes(b"foo")
bytes(b"""
foo""")
f"{str()}"
int()
int(1)
float()
float(1.0)
bool()
bool(True)
bool(False)

# These become a literal but retain parentheses
int(1).denominator

# These too are literals in spirit
int(+1)
int(-1)
float(+1.0)
float(-1.0)


# https://github.com/astral-sh/ruff/issues/15859
int(-1) ** 0                 # (-1) ** 0
2 ** int(-1)                 # 2 ** -1

int(-1)[0]                   # (-1)[0]
2[int(-1)]                   # 2[-1]

int(-1)(0)                   # (-1)(0)
2(int(-1))                   # 2(-1)

float(-1.0).foo              # (-1.0).foo

await int(-1)                # await (-1)


int(+1) ** 0
float(+1.0)()


str(
    '''Lorem
    ipsum'''  # Comment
).foo

# https://github.com/astral-sh/ruff/issues/17606
bool(True)and None
int(1)and None
float(1.)and None
bool(True)and()


# t-strings are not native literals
str(t"hey")
