import colorama

from typing import Union
from colorama import Fore

from basic import Location
from enum import Enum

colorama.init(autoreset=True)


class DiagnosticKind(Enum):
    ERROR = "error"
    WARNING = "warning"
    NOTE = "note"


class Diagnostic(Exception):
    def __init__(self, msg: str, location: Location, kind: DiagnosticKind):
        self.msg = msg
        self.location = location
        self.kind = kind

    def dump(self):
        """输出信息"""
        if self.kind == DiagnosticKind.ERROR:
            print(Fore.RED + "错误", end=": ")
        elif self.kind == DiagnosticKind.WARNING:
            print(Fore.YELLOW + "警告", end=": ")
        elif self.kind == DiagnosticKind.NOTE:
            print(Fore.BLUE + "附注", end=": ")
        print(self.msg)
        indent = " " * 4
        for loc in self.location:
            filename = loc["filename"]
            lines = Location.lines[filename]
            row = loc["lineno"] - 1
            col = loc["col"] - 1
            if row >= len(lines):
                continue
            prefix = indent * 2 + f"{loc['lineno']}|"
            print(indent + f"{self.location}:")
            if isinstance(lines[row], bytes):
                print(prefix, " ".join(map(lambda a: hex(a)[2:].upper(), lines[row])))
                print(" " * len(prefix), " " * (col - 1) * 3, end="")
                for _ in range(loc["span_col"]):
                    print("^^ ", end="")
                print()
            else:
                print(prefix, lines[row].rstrip())
                print(" " * len(prefix), " " * col + "^" * loc["span_col"])


class Error(Diagnostic):
    def __init__(self, msg: str, location: Location):
        super().__init__(msg, location, DiagnosticKind.ERROR)


class Note(Diagnostic):
    def __init__(self, msg: str, location: Location):
        super().__init__(msg, location, DiagnosticKind.NOTE)


class Warn(Diagnostic):
    def __init__(self, msg: str, location: Location):
        super().__init__(msg, location, DiagnosticKind.WARNING)


class Diagnostics(Exception):
    def __init__(self, diagnostic_list: list[Diagnostic]):
        self.list = diagnostic_list

    def __add__(self, other: Union[list[Diagnostic], "Diagnostics"]):
        diagnostic_list = self.list
        if isinstance(other, Diagnostics):
            diagnostic_list += other.list
        else:
            diagnostic_list += other
        return Diagnostics(diagnostic_list)

    def __bool__(self):
        return bool(self.list)

    def dump(self):
        for diagnostic in self.list:
            diagnostic.dump()
