from enum import Enum
import string

# Here goes the specification. We need to support one of the following arguments:
# /p:handle -- show the preview into given handle
# /s        -- show the main event fullscreen
# /s:handle -- show the main event into a given handle
# /c        -- show the configuration window
# /c:handle -- no idea
#
# Now, the trouble part. The arguments may be either in a lower or upper case.
# The handle may be separated by either ":" or space.
# Another day with the great standartisation by Micro$oft.


class CliCommandMode(Enum):
    SHOW = 1
    CONFIG = 2
    PREVIEW = 3


class CliCommand:
    def __init__(self, command: CliCommandMode, handle: int | None) -> None:
        self.command = command
        if command == CliCommandMode.PREVIEW and handle == None:
            raise ValueError("CliCommand PREVIEW requires handle value")
        self.handle = handle


def parse_cli_arguments(args: list[str]):
    del args[0]  # First arg is EXE name
    if len(args) > 2:
        raise ValueError("Wrong number of arguments")
    if args
    # Now flatten args and clean
    arg = " ".join(args).casefold().replace(":", " ")
    # Clean multiple whitespaces
    newarglist = []
    for i in range(1, len(arg)):
        c = arg[i]
        if c in string.whitespace:
            c = " "
            if arg[i - 1] in string.whitespace:
                continue
        newarglist.append(c)
    arg = "".join(newarglist)
