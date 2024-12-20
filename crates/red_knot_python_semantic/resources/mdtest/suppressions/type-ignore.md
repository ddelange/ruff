# Suppressing errors with `type: ignore`

Type check errors can be suppressed by a `type: ignore` comment on the same line as the violation.

## Simple `type: ignore`

```py
a = 4 + test  # type: ignore
```

## Multiline ranges

A diagnostic with a multiline range can be suppressed by a comment on the same line as the
diagnostic's start or end. This is the same behavior as Mypy's.

```py
# fmt: off
y = (
    4 / 0  # type: ignore
)

y = (
    4 /  # type: ignore
    0
)

y = (
    4 /
    0  # type: ignore
)
```

Pyright diverges from this behavior and instead applies a suppression if its range intersects with
the diagnostic range. This can be problematic for nested expressions because a suppression in a
child expression now suppresses errors in the outer expression.

For example, the `type: ignore` comment in this example suppresses the error of adding `2` to
`"test"` and adding `"other"` to the result of the cast.

```py path=nested.py
# fmt: off
from typing import cast

y = (
    cast(int, "test" +
            2 # type: ignore
    )
    + "other"  # TODO: expected-error[invalid-operator]
)
```

Mypy flags the second usage.

## Before opening parenthesis

A suppression that applies to all errors before the opening parenthesis.

```py
a: Test = (  # type: ignore
  Test()  # error: [unresolved-reference]
)  # fmt: skip
```

## Multiline string

```py
a: int = 4
a = """
  This is a multiline string and the suppression is at its end
"""  # type: ignore
```

## Line continuations

Suppressions after a line continuation apply to all previous lines.

```py
# fmt: off
a = test \
  + 2  # type: ignore

a = test \
  + a \
  + 2  # type: ignore
```

## Codes

Mypy supports `type: ignore[code]`. Red Knot doesn't understand mypy's rule names. Therefore, ignore
the codes and suppress all errors.

```py
a = test  # type: ignore[name-defined]
```

## Nested comments

TODO: We should support this for better interopability with other suppression comments.

```py
# fmt: off
# TODO this error should be suppressed
# error: [unresolved-reference]
a = test \
  + 2  # fmt: skip # type: ignore

a = test \
  + 2  # type: ignore # fmt: skip
```

## Misspelled `type: ignore`

```py
# error: [unresolved-reference]
a = test + 2  # type: ignoree
```

## Invalid - ignore on opening parentheses

`type: ignore` comments after an opening parentheses suppress any type errors inside the parentheses
in Pyright. Neither Ruff, nor mypy support this and neither does Red Knot.

```py
# fmt: off
a = (  # type: ignore
    test + 4  # error: [unresolved-reference]
)
```
