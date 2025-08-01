import sys
from collections.abc import Callable
from typing import Any, ClassVar, Final, final
from typing_extensions import TypeAlias

# _tkinter is meant to be only used internally by tkinter, but some tkinter
# functions e.g. return _tkinter.Tcl_Obj objects. Tcl_Obj represents a Tcl
# object that hasn't been converted to a string.
#
# There are not many ways to get Tcl_Objs from tkinter, and I'm not sure if the
# only existing ways are supposed to return Tcl_Objs as opposed to returning
# strings. Here's one of these things that return Tcl_Objs:
#
#    >>> import tkinter
#    >>> text = tkinter.Text()
#    >>> text.tag_add('foo', '1.0', 'end')
#    >>> text.tag_ranges('foo')
#    (<textindex object: '1.0'>, <textindex object: '2.0'>)
@final
class Tcl_Obj:
    @property
    def string(self) -> str:
        """the string representation of this object, either as str or bytes"""

    @property
    def typename(self) -> str:
        """name of the Tcl type"""
    __hash__: ClassVar[None]  # type: ignore[assignment]
    def __eq__(self, value, /): ...
    def __ge__(self, value, /): ...
    def __gt__(self, value, /): ...
    def __le__(self, value, /): ...
    def __lt__(self, value, /): ...
    def __ne__(self, value, /): ...

class TclError(Exception): ...

_TkinterTraceFunc: TypeAlias = Callable[[tuple[str, ...]], object]

# This class allows running Tcl code. Tkinter uses it internally a lot, and
# it's often handy to drop a piece of Tcl code into a tkinter program. Example:
#
#    >>> import tkinter, _tkinter
#    >>> tkapp = tkinter.Tk().tk
#    >>> isinstance(tkapp, _tkinter.TkappType)
#    True
#    >>> tkapp.call('set', 'foo', (1,2,3))
#    (1, 2, 3)
#    >>> tkapp.eval('return $foo')
#    '1 2 3'
#    >>>
#
# call args can be pretty much anything. Also, call(some_tuple) is same as call(*some_tuple).
#
# eval always returns str because _tkinter_tkapp_eval_impl in _tkinter.c calls
# Tkapp_UnicodeResult, and it returns a string when it succeeds.
@final
class TkappType:
    # Please keep in sync with tkinter.Tk
    def adderrorinfo(self, msg, /): ...
    def call(self, command: Any, /, *args: Any) -> Any: ...
    def createcommand(self, name, func, /): ...
    if sys.platform != "win32":
        def createfilehandler(self, file, mask, func, /): ...
        def deletefilehandler(self, file, /): ...

    def createtimerhandler(self, milliseconds, func, /): ...
    def deletecommand(self, name, /): ...
    def dooneevent(self, flags: int = 0, /): ...
    def eval(self, script: str, /) -> str: ...
    def evalfile(self, fileName, /): ...
    def exprboolean(self, s, /): ...
    def exprdouble(self, s, /): ...
    def exprlong(self, s, /): ...
    def exprstring(self, s, /): ...
    def getboolean(self, arg, /): ...
    def getdouble(self, arg, /): ...
    def getint(self, arg, /): ...
    def getvar(self, *args, **kwargs): ...
    def globalgetvar(self, *args, **kwargs): ...
    def globalsetvar(self, *args, **kwargs): ...
    def globalunsetvar(self, *args, **kwargs): ...
    def interpaddr(self) -> int: ...
    def loadtk(self) -> None: ...
    def mainloop(self, threshold: int = 0, /): ...
    def quit(self): ...
    def record(self, script, /): ...
    def setvar(self, *ags, **kwargs): ...
    if sys.version_info < (3, 11):
        def split(self, arg, /): ...

    def splitlist(self, arg, /): ...
    def unsetvar(self, *args, **kwargs): ...
    def wantobjects(self, *args, **kwargs): ...
    def willdispatch(self): ...
    if sys.version_info >= (3, 12):
        def gettrace(self, /) -> _TkinterTraceFunc | None:
            """Get the tracing function."""

        def settrace(self, func: _TkinterTraceFunc | None, /) -> None:
            """Set the tracing function."""

# These should be kept in sync with tkinter.tix constants, except ALL_EVENTS which doesn't match TCL_ALL_EVENTS
ALL_EVENTS: Final = -3
FILE_EVENTS: Final = 8
IDLE_EVENTS: Final = 32
TIMER_EVENTS: Final = 16
WINDOW_EVENTS: Final = 4

DONT_WAIT: Final = 2
EXCEPTION: Final = 8
READABLE: Final = 2
WRITABLE: Final = 4

TCL_VERSION: Final[str]
TK_VERSION: Final[str]

@final
class TkttType:
    def deletetimerhandler(self): ...

if sys.version_info >= (3, 13):
    def create(
        screenName: str | None = None,
        baseName: str = "",
        className: str = "Tk",
        interactive: bool = False,
        wantobjects: int = 0,
        wantTk: bool = True,
        sync: bool = False,
        use: str | None = None,
        /,
    ):
        """

        wantTk
          if false, then Tk_Init() doesn't get called
        sync
          if true, then pass -sync to wish
        use
          if not None, then pass -use to wish
        """

else:
    def create(
        screenName: str | None = None,
        baseName: str = "",
        className: str = "Tk",
        interactive: bool = False,
        wantobjects: bool = False,
        wantTk: bool = True,
        sync: bool = False,
        use: str | None = None,
        /,
    ):
        """

        wantTk
          if false, then Tk_Init() doesn't get called
        sync
          if true, then pass -sync to wish
        use
          if not None, then pass -use to wish
        """

def getbusywaitinterval():
    """Return the current busy-wait interval between successive calls to Tcl_DoOneEvent in a threaded Python interpreter."""

def setbusywaitinterval(new_val, /):
    """Set the busy-wait interval in milliseconds between successive calls to Tcl_DoOneEvent in a threaded Python interpreter.

    It should be set to a divisor of the maximum time between frames in an animation.
    """
