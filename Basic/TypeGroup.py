"""类型组别的判断"""

from Basic.Type import (
    Type,
    TypedefType,
    BasicType,
    BasicTypeKind,
    BitIntType,
    PointerType,
    NullPtrType,
)


def is_integer_type(type: Type):
    """判断是否是整数类型"""
    if isinstance(type, TypedefType):
        type = type.type
    if not isinstance(type, BasicType):
        return False
    if type.kind in (
        BasicTypeKind.CHAR,
        BasicTypeKind.UCHAR,
        BasicTypeKind.INT,
        BasicTypeKind.UINT,
        BasicTypeKind.LONG,
        BasicTypeKind.ULONG,
        BasicTypeKind.SHORT,
        BasicTypeKind.USHORT,
        BasicTypeKind.LONGLONG,
        BasicTypeKind.ULONGLONG,
    ):
        return True
    if isinstance(type, BitIntType):
        return True
    return False


def is_arithmetic_type(type: Type):
    """判断是否为算术类型"""
    if is_integer_type(type):
        return True
    if isinstance(type, BasicType):
        if type.kind in (
            BasicTypeKind.FLOAT,
            BasicTypeKind.DOUBLE,
            BasicTypeKind.LONGDOUBLE,
            BasicTypeKind.DECIMAL32,
            BasicTypeKind.DECIMAL64,
            BasicTypeKind.DECIMAL128,
            BasicTypeKind.FLOAT_COMPLEX,
            BasicTypeKind.DOUBLE_COMPLEX,
            BasicTypeKind.LONGDOUBLE_COMPLEX,
            BasicTypeKind.FLOAT_IMAGINARY,
            BasicTypeKind.DOUBLE_IMAGINARY,
            BasicTypeKind.LONGDOUBLE_IMAGINARY,
        ):
            return True
    return False


def is_decimal_floating_type(type: Type):
    """判断是否为十进制实浮点类型"""
    if isinstance(type, BasicType):
        if type.kind in (
            BasicTypeKind.DECIMAL32,
            BasicTypeKind.DECIMAL64,
            BasicTypeKind.DECIMAL128,
        ):
            return True
    return False


def is_real_floating_type(type: Type):
    """判断是否为实浮点类型"""
    if isinstance(type, BasicType):
        if type.kind in (
            BasicTypeKind.FLOAT,
            BasicTypeKind.DOUBLE,
            BasicTypeKind.LONGDOUBLE,
            BasicTypeKind.DECIMAL32,
            BasicTypeKind.DECIMAL64,
            BasicTypeKind.DECIMAL128,
        ):
            return True
    return False


def is_real_type(type: Type):
    """判断是否为实数类型"""
    return is_integer_type(type) or is_real_floating_type(type)


def is_scalar_type(type: Type):
    """判断是否为标量类型"""
    return (
        is_arithmetic_type(type)
        or isinstance(type, PointerType)
        or isinstance(type, NullPtrType)
    )
