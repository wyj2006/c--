"""
各种类型
"""

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
    if a == b:  # 相同类型
        return True
    match a, b:
        case TypedefType(), _:  # 由typedef引入的别名
            return is_compatible_type(a.type, b)
        case _, TypedefType():
            return is_compatible_type(a, b.type)
        case (
            QualifiedType(),
            QualifiedType(),
        ):  # 兼容的非限定类型的完全相同的 cvr 限定版本
            return set(a.qualifiers) == set(b.qualifiers) and is_compatible_type(
                a.type, b.type
            )
        case PointerType(), PointerType():  # 都是指针类型，需要指向兼容的类型
            return is_compatible_type(a.pointee_type, b.pointee_type)
        case ArrayType(), ArrayType():  # 数组类型
            return is_compatible_type(
                a.element_type, b.element_type  # 元素类型是兼容的
            ) and (  # 如果两者都具有恒定大小，则大小应该相同
                True
                if (a.len_expr == None or b.len_expr == None)
                else a.len_expr.value == b.len_expr.value
            )
        case EnumType(), EnumType():
            if a.is_complete and b.is_complete:
                if len(a.enumerators) != len(b.enumerators):  # 在数量上完全对应
                    return False
                if not is_compatible_type(a.underlying_type, b.underlying_type):
                    return False
                for x, y in zip(a.enumerators, b.enumerators):
                    m = a.enumerators[x]
                    n = b.enumerators[y]
                    if (
                        m.value_expr.value != n.value_expr.value
                    ):  # 对应的成员具有相同的值
                        return False
            return True
        case RecordType(), RecordType():
            if a.is_complete and b.is_complete:
                if len(a.members) != len(b.members):  # 在数量上完全对应
                    return False
                for x, y in zip(a.members, b.members):
                    m = a.members[x]
                    n = b.members[y]
                    if m.name != n.name:  # 对应的成员需要以相同的顺序声明
                        return False
                    if (
                        m.bit_field.value != n.bit_field.value
                    ):  # 对应的位域应该具有相同的宽度
                        return False
                    if not is_compatible_type(m.type, n.type):  # 成员用兼容的类型声明
                        return False
            return True
        case EnumType(), _ if (
            a.underlying_type == b
        ):  # 枚举类型，另一个是该枚举的底层类型
            return True
        case _, EnumType() if (
            b.underlying_type == a
        ):  # 枚举类型，另一个是该枚举的底层类型
            return True
        case FunctionType(), FunctionType():
            return (
                is_compatible_type(a.return_type, b.return_type)  # 返回类型兼容
                and a.has_varparam == b.has_varparam  # 参数数量相同(包括省略号)
                and [remove_qualifier(i) for i in a.parameters_type]
                == [remove_qualifier(i) for i in b.parameters_type]  # 对应参数类型兼容
            )
    return False


def composite_type(a: Type, b: Type) -> Type:
    """合成类型"""
    if a == b:
        return a
    elif isinstance(a, ArrayType) and isinstance(b, ArrayType):
        if a.len_expr == None == b.len_expr:  # 两者都是未知大小的数组
            return ArrayType(composite_type(a.element_type, b.element_type), None)
        elif hasattr(a.len_expr, "value"):  # a为恒定大小的数组
            return ArrayType(composite_type(a.element_type, b.element_type), a.len_expr)
        elif hasattr(b.len_expr, "value"):  # b为恒定大小的数组
            return ArrayType(composite_type(a.element_type, b.element_type), b.len_expr)
        return ArrayType(composite_type(a.element_type, b.element_type), b.len_expr)
    elif isinstance(a, FunctionType) and isinstance(b, FunctionType):
        parameters_type = []
        for i in range(max(len(a.parameters_type), len(b.parameters_type))):
            x = a.parameters_type[i]
            y = b.parameters_type[i]
            if x == None:
                parameters_type.append(y)
            elif y == None:
                parameters_type.append(x)
            else:
                parameters_type.append(composite_type(x, y))
        return FunctionType(
            parameters_type, a.return_type, a.has_varparam or b.has_varparam
        )
    elif isinstance(a, EnumType) and a.underlying_type == b:
        return b
    elif isinstance(b, EnumType) and b.underlying_type == a:
        return a
    raise Exception(f"无法合成{a}和{b}")
