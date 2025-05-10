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
from Basic.Type import (
    Type,
    BasicType,
    BasicTypeKind,
    BitIntType,
    PointerType,
    ArrayType,
    RecordType,
    TypedefType,
    AtomicType,
    TypeofType,
    QualifiedType,
    ArrayPtrType,
    AutoType,
    NullPtrType,
    FunctionType,
    EnumType,
    Char16Type,
    Char32Type,
    Char8Type,
    WCharType,
    remove_qualifier,
    integer_promotion,
    remove_atomic,
    is_compatible_type,
    composite_type,
)
from Basic.FlagManager import FlagManager
from Basic.TypeGroup import (
    is_integer_type,
    is_arithmetic_type,
    is_scalar_type,
    is_real_type,
    is_real_floating_type,
)
