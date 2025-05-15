"""通用模块"""

from Basic.Token import TokenKind, Token, TokenGen
from Basic.Location import Location
from Basic.FileReader import FileReader, MergeReader
from Basic.Diagnostic import (
    Diagnostic,
    Error,
    DiagnosticKind,
    Diagnostics,
    Note,
    Warn,
)
from Basic.Symtab import (
    Symtab,
    Symbol,
    TAG_NAMES,
    LABEL_NAMES,
    ORDINARY_NAMES,
    MEMBER_NAMES,
    ATTRIBUTE_NAMES,
    Object,
    Member,
    EnumConst,
    Parameter,
    Function,
)
from Basic.FlagManager import FlagManager
