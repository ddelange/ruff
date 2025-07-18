"""
Token constants (from "token.h").
"""

from typing import Final

ENDMARKER: Final[int]
NAME: Final[int]
NUMBER: Final[int]
STRING: Final[int]
NEWLINE: Final[int]
INDENT: Final[int]
DEDENT: Final[int]
LPAR: Final[int]
RPAR: Final[int]
LSQB: Final[int]
RSQB: Final[int]
COLON: Final[int]
COMMA: Final[int]
SEMI: Final[int]
PLUS: Final[int]
MINUS: Final[int]
STAR: Final[int]
SLASH: Final[int]
VBAR: Final[int]
AMPER: Final[int]
LESS: Final[int]
GREATER: Final[int]
EQUAL: Final[int]
DOT: Final[int]
PERCENT: Final[int]
BACKQUOTE: Final[int]
LBRACE: Final[int]
RBRACE: Final[int]
EQEQUAL: Final[int]
NOTEQUAL: Final[int]
LESSEQUAL: Final[int]
GREATEREQUAL: Final[int]
TILDE: Final[int]
CIRCUMFLEX: Final[int]
LEFTSHIFT: Final[int]
RIGHTSHIFT: Final[int]
DOUBLESTAR: Final[int]
PLUSEQUAL: Final[int]
MINEQUAL: Final[int]
STAREQUAL: Final[int]
SLASHEQUAL: Final[int]
PERCENTEQUAL: Final[int]
AMPEREQUAL: Final[int]
VBAREQUAL: Final[int]
CIRCUMFLEXEQUAL: Final[int]
LEFTSHIFTEQUAL: Final[int]
RIGHTSHIFTEQUAL: Final[int]
DOUBLESTAREQUAL: Final[int]
DOUBLESLASH: Final[int]
DOUBLESLASHEQUAL: Final[int]
OP: Final[int]
COMMENT: Final[int]
NL: Final[int]
RARROW: Final[int]
AT: Final[int]
ATEQUAL: Final[int]
AWAIT: Final[int]
ASYNC: Final[int]
ERRORTOKEN: Final[int]
COLONEQUAL: Final[int]
N_TOKENS: Final[int]
NT_OFFSET: Final[int]
tok_name: dict[int, str]

def ISTERMINAL(x: int) -> bool: ...
def ISNONTERMINAL(x: int) -> bool: ...
def ISEOF(x: int) -> bool: ...
