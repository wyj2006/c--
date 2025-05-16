"""通用模块"""

from basic.location import Location
from basic.token import TokenKind, Token, TokenGen
from basic.diagnostic import (
    Diagnostic,
    Error,
    DiagnosticKind,
    Diagnostics,
    Note,
    Warn,
)
from basic.file_reader import FileReader, MergeReader
from basic.symtab import (
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
from basic.flag_manager import FlagManager

from basic.types import *
from basic.ast import *
