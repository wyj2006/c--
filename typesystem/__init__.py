"""
各种类型
"""

# TODO 如何使用基类和派生类(比如IntegerType和IntType)
from .ctype import (
    Type,
    VoidType,
    TypeofType,
    AutoType,
    EnumType,
    ArrayType,
    AtomicType,
    RecordType,
    NullPtrType,
    PointerType,
    ArrayPtrType,
    FunctionType,
    QualifiedType,
)
from .typedef_type import (
    TypedefType,
    Char16Type,
    Char32Type,
    Char8Type,
    SizeType,
    WCharType,
)
from .char_type import CharType, SCharType, UCharType
from .integer_type import (
    ShortType,
    IntType,
    LongType,
    UIntType,
    ULongLongType,
    ULongType,
    UShortType,
    BitIntType,
    IntegerType,
    LongLongType,
    BoolType,
)
from .real_floating_type import (
    FloatType,
    Decimal128Type,
    Decimal32Type,
    Decimal64Type,
    DoubleType,
    LongDoubleType,
    DecimalType,
    BinaryFloatType,
    RealFloatingType,
)
from .complex_type import (
    FloatComplexType,
    DoubleComplexType,
    LongDoubleComplexType,
    ComplexType,
)
from .imaginary_type import (
    FloatImaginaryType,
    DoubleImaginaryType,
    LongDoubleImaginaryType,
    ImaginaryType,
)


def remove_qualifier(type: Type):
    """去除type的限定符"""
    if isinstance(type, QualifiedType):
        return type.type
    return type


def remove_atomic(type: Type):
    """去除原子属性"""
    if isinstance(type, AtomicType):
        return type.type
    return type


def integer_promotion(type: Type):
    """整数提升"""
    if isinstance(type, TypedefType):
        type = type.type
    if not isinstance(type, IntegerType):
        return type
    if type.size >= IntType.size:
        return type
    limit = type.range
    int_limit = IntType().range
    if int_limit[0] <= limit[0] and limit[1] <= int_limit[1]:
        return IntType()
    return UIntType()


def is_compatible_type(a: Type, b: Type):
    """判断a和b是否是兼容类型"""
    if a == b:
        return True
    match a, b:
        case TypedefType(), _:
            return is_compatible_type(a.type, b)
        case _, TypedefType():
            return is_compatible_type(a, b.type)
        case QualifiedType(), QualifiedType():
            return set(a.qualifiers) == set(b.qualifiers) and is_compatible_type(
                a.type, b.type
            )
        case PointerType(), PointerType():
            return is_compatible_type(a.pointee_type, b.pointee_type)
        case ArrayType(), ArrayType():
            return is_compatible_type(a.element_type, b.element_type) and (
                True
                if (a.len_expr == None or b.len_expr == None)
                else a.len_expr.value == b.len_expr.value
            )
        case EnumType(), EnumType():
            if a.is_complete and b.is_complete:
                if len(a.enumerators) != len(b.enumerators):
                    return False
                if not is_compatible_type(a.underlying_type, b.underlying_type):
                    return False
                for x, y in zip(a.enumerators, b.enumerators):
                    m = a.enumerators[x]
                    n = b.enumerators[y]
                    if m.value_expr.value != n.value_expr.value:
                        return False
            return True
        case RecordType(), RecordType():
            if a.is_complete and b.is_complete:
                if len(a.members) != len(b.members):
                    return False
                for x, y in zip(a.members, b.members):
                    m = a.members[x]
                    n = b.members[y]
                    if a.struct_or_union == "struct" and m.name != n.name:
                        return False
                    if m.bit_field.value != n.bit_field.value:
                        return False
                    if not is_compatible_type(m.type, n.type):
                        return False
            return True
        case EnumType(), _ if a.underlying_type == b:
            return True
        case _, EnumType() if b.underlying_type == a:
            return True
        case FunctionType(), FunctionType():
            return (  # TODO 完善
                is_compatible_type(a.return_type, b.return_type)
                and a.has_varparam == b.has_varparam
                and a.parameters_type == b.parameters_type
            )
    return False


def composite_type(a: Type, b: Type) -> Type:
    """合成类型"""
    # TODO 完善
    if isinstance(a, ArrayType) and isinstance(b, ArrayType):
        if a.len_expr == None == b.len_expr:
            return ArrayType(composite_type(a.element_type, b.element_type), None)
        elif hasattr(a.len_expr, "value"):
            return ArrayType(composite_type(a.element_type, b.element_type), a.len_expr)
        elif hasattr(b.len_expr, "value"):
            return ArrayType(composite_type(a.element_type, b.element_type), b.len_expr)
        return ArrayType(composite_type(a.element_type, b.element_type), b.len_expr)
    elif isinstance(a, FunctionType) and isinstance(b, FunctionType):
        parameters_type = []
        for x, y in zip(a.parameters_type, b.parameters_type):
            parameters_type.append(composite_type(x, y))
        return FunctionType(
            parameters_type, a.return_type, a.has_varparam or b.has_varparam
        )
    elif isinstance(a, PointerType) and isinstance(b, PointerType):
        return PointerType(composite_type(a.pointee_type, b.pointee_type))
    if is_compatible_type(a, b):
        return a
    raise Exception(f"无法合成{a}和{b}")
