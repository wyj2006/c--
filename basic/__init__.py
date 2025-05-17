"""通用模块"""

from .location import Location
from .token import TokenKind, Token, TokenGen
from .diagnostic import (
    Diagnostic,
    Error,
    DiagnosticKind,
    Diagnostics,
    Note,
    Warn,
)
from .file_reader import FileReader, MergeReader
from .symtab import (
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
from .flag_manager import FlagManager
