import ctypes
from enum import Enum
from typing import TYPE_CHECKING, Union
from Basic.Symtab import Symbol, Member, EnumConst

if TYPE_CHECKING:
    from AST import (
        Expr,
        TypeQualifier,
        AttributeSpecifier,
        SingleDeclration,
        TypeQualifierKind,
    )


class Type(Symbol):
    """类型符号"""

    def __str__(self):
        from AST import SingleDeclration, UnParseVisitor

        declaration = SingleDeclration(specifiers=[], declarator=None)
        self.genDeclaration(declaration)
        return declaration.accept(UnParseVisitor())

    def genDeclaration(self, declaration: "SingleDeclration"):
        pass

    def is_complete(self) -> bool:
        """是否是完整类型"""
        return True


class BasicTypeKind(Enum):
    VOID = "void"
    CHAR = "char"
    SHORT = "short"
    INT = "int"
    LONG = "long"
    LONGLONG = "long long"
    BOOL = "bool"
    UCHAR = "unsigned char"
    USHORT = "unsigned short"
    UINT = "unsigned int"
    ULONG = "unsigned long"
    ULONGLONG = "unsigned long long"
    FLOAT = "float"
    DOUBLE = "double"
    LONGDOUBLE = "long double"
    DECIMAL32 = "_Decimal32"
    DECIMAL64 = "_Decimal64"
    DECIMAL128 = "_Decimal128"
    FLOAT_COMPLEX = "float _Complex"
    DOUBLE_COMPLEX = "double _Complex"
    LONGDOUBLE_COMPLEX = "long double _Complex"
    FLOAT_IMAGINARY = "float _Imaginary"
    DOUBLE_IMAGINARY = "double _Imaginary"
    LONGDOUBLE_IMAGINARY = "long double _Imaginary"


class BasicType(Type):
    combination = {  # 类型组合
        None: {
            "void": BasicTypeKind.VOID,
            "char": BasicTypeKind.CHAR,
            "short": BasicTypeKind.SHORT,
            "int": BasicTypeKind.INT,
            "long": BasicTypeKind.LONG,
            "bool": BasicTypeKind.BOOL,
            "float": BasicTypeKind.FLOAT,
            "double": BasicTypeKind.DOUBLE,
            "_Decimal32": BasicTypeKind.DECIMAL32,
            "_Decimal64": BasicTypeKind.DECIMAL64,
            "_Decimal128": BasicTypeKind.DECIMAL128,
            "signed": BasicTypeKind.INT,
            "unsigned": BasicTypeKind.UINT,
        },
        BasicTypeKind.CHAR: {
            "signed": BasicTypeKind.CHAR,
            "unsigned": BasicTypeKind.UCHAR,
        },
        BasicTypeKind.SHORT: {
            "signed": BasicTypeKind.SHORT,
            "unsigned": BasicTypeKind.USHORT,
        },
        BasicTypeKind.INT: {
            "signed": BasicTypeKind.INT,
            "unsigned": BasicTypeKind.UINT,
            "short": BasicTypeKind.SHORT,
            "long": BasicTypeKind.LONG,
        },
        BasicTypeKind.LONG: {
            "signed": BasicTypeKind.LONG,
            "unsigned": BasicTypeKind.ULONG,
            "long": BasicTypeKind.LONGLONG,
            "double": BasicTypeKind.LONGDOUBLE,
        },
        BasicTypeKind.LONGLONG: {
            "signed": BasicTypeKind.LONGLONG,
            "unsigned": BasicTypeKind.ULONGLONG,
        },
        BasicTypeKind.FLOAT: {
            "_Complex": BasicTypeKind.FLOAT_COMPLEX,
            "_Imaginary": BasicTypeKind.FLOAT_IMAGINARY,
        },
        BasicTypeKind.DOUBLE: {
            "_Complex": BasicTypeKind.DOUBLE_COMPLEX,
            "_Imaginary": BasicTypeKind.DOUBLE_IMAGINARY,
        },
        BasicTypeKind.LONGDOUBLE: {
            "_Complex": BasicTypeKind.LONGDOUBLE_COMPLEX,
            "_Imaginary": BasicTypeKind.LONGDOUBLE_IMAGINARY,
        },
    }

    def __init__(self, kind: BasicTypeKind):
        super().__init__()
        self.kind = kind

    def genDeclaration(self, declaration):
        from AST import BasicTypeSpecifier

        declaration.specifiers.append(
            BasicTypeSpecifier(specifier_name=self.kind.value)
        )

    def __eq__(self, other):
        return isinstance(other, BasicType) and self.kind == other.kind

    def is_complete(self):
        return self.kind != BasicTypeKind.VOID


class BitIntType(Type):
    def __init__(self, size: "Expr", signed=True):
        super().__init__()
        self.size = size
        self.signed = signed

    def genDeclaration(self, declaration):
        from AST import BasicTypeSpecifier, BitIntSpecifier

        if not self.signed:
            declaration.specifiers.append(BasicTypeSpecifier(specifier_name="unsigned"))
        declaration.specifiers.append(BitIntSpecifier(size=self.size))

    def __eq__(self, other):
        return (
            isinstance(other, BitIntType)
            and self.size == other.size
            and self.signed == other.signed
        )


class PointerType(Type):
    def __init__(
        self,
        pointee_type: Type,
        attribute_specifiers: list["AttributeSpecifier"] = None,
    ):
        super().__init__(attribute_specifiers)
        self.pointee_type = pointee_type

    def genDeclaration(self, declaration):
        from AST import PointerDeclarator

        declarator = PointerDeclarator(qualifiers=[], declarator=declaration.declarator)
        declaration.declarator = declarator
        self.pointee_type.genDeclaration(declaration)

    def __eq__(self, other):
        return (
            isinstance(other, PointerType) and self.pointee_type == other.pointee_type
        )

    def is_complete(self):
        return self.pointee_type.is_complete()


class ArrayType(Type):
    def __init__(
        self,
        element_type: Type,
        size: "Expr",
        is_star_modified: bool = False,
        is_static: bool = False,
        attribute_specifiers: list["AttributeSpecifier"] = None,
    ):
        super().__init__(attribute_specifiers)
        self.element_type = element_type
        self.size = size
        self.is_star_modified = is_star_modified
        self.is_static = is_static

    def __eq__(self, other):
        return (
            isinstance(other, ArrayType)
            and self.element_type == other.element_type
            and self.size == other.size
            and self.is_star_modified == other.is_star_modified
            and self.is_static == other.is_static
        )

    def genDeclaration(self, declaration):
        from AST import ArrayDeclarator

        declarator = ArrayDeclarator(
            size=self.size,
            is_star_modified=self.is_star_modified,
            is_static=self.is_static,
            qualifiers=[],
            declarator=declaration.declarator,
        )
        declaration.declarator = declarator
        self.element_type.genDeclaration(declaration)

    def is_complete(self):
        return self.element_type.is_complete() and self.size != None


class ArrayPtrType(PointerType):
    """函数参数声明中由数组转换过来的指针类型"""

    def __init__(self, array_type: ArrayType):
        super().__init__(array_type.element_type)
        self.array_type = array_type


class FunctionType(Type):
    def __init__(
        self,
        parameters_type: list[Type],
        return_type: Type,
        has_varparam: bool = False,
        attribute_specifiers: list["AttributeSpecifier"] = None,
    ):
        super().__init__(attribute_specifiers)
        self.parameters_type = parameters_type
        self.return_type = return_type
        self.has_varparam = has_varparam

    def genDeclaration(self, declaration):
        from AST import FunctionDeclarator, ParamDecl

        declarator = FunctionDeclarator(
            parameters=[],
            has_varparam=self.has_varparam,
            qualifiers=[],
            declarator=declaration.declarator,
        )

        for i in self.parameters_type:
            param_decl = ParamDecl(specifiers=[], declarator=None)
            i.genDeclaration(param_decl)
            declarator.parameters.append(param_decl)

        declaration.declarator = declarator

        self.return_type.genDeclaration(declaration)

    def __eq__(self, other):
        return (
            isinstance(other, FunctionType)
            and self.parameters_type == other.parameters_type
            and self.return_type == other.return_type
            and self.has_varparam == other.has_varparam
        )


class RecordType(Type):
    def __init__(
        self,
        struct_or_union: str,
        name: str,
        attribute_specifiers: list["AttributeSpecifier"] = None,
    ):
        super().__init__(attribute_specifiers)
        self.struct_or_union = struct_or_union
        self.name = name
        self.members: dict[str, Member] = {}

    def genDeclaration(self, declaration):
        from AST import RecordDecl

        declaration.specifiers.append(
            RecordDecl(struct_or_union=self.struct_or_union, name=self.name)
        )

    def __eq__(self, other):
        return (
            isinstance(other, RecordType)
            and self.struct_or_union == other.struct_or_union
            and self.name == other.name
            and self.members == other.members
        )

    def is_complete(self):
        return len(self.members) > 0


class TypedefType(Type):
    def __init__(self, name: str, type: Type):
        super().__init__()
        self.name = name
        self.type = type

    def __str__(self):
        return f"{self.name}:{self.type}"

    def genDeclaration(self, declaration):
        from AST import TypedefSpecifier

        declaration.specifiers.append(TypedefSpecifier(specifier_name=self.name))

    def __eq__(self, other):
        return (
            isinstance(other, TypedefType)
            and self.name == other.name
            and self.type == other.type
        )

    def is_complete(self):
        return self.type.is_complete()


class EnumType(Type):
    def __init__(
        self,
        name: str,
        underlying_type: Type = None,
        attribute_specifiers: list["AttributeSpecifier"] = None,
    ):
        super().__init__(attribute_specifiers)
        self.name = name
        self.underlying_type = underlying_type
        self.enumerators: dict[str, EnumConst] = {}

    def genDeclaration(self, declaration):
        from AST import EnumDecl

        declaration.specifiers.append(EnumDecl(name=self.name))

    def __eq__(self, other):
        return (
            isinstance(other, EnumType)
            and self.name == other.name
            and self.underlying_type == other.underlying_type
            and self.enumerators == other.enumerators
        )

    def is_complete(self):
        return len(self.enumerators) > 0


class AtomicType(Type):
    def __init__(
        self, type: Type, attribute_specifiers: list["AttributeSpecifier"] = None
    ):
        super().__init__(attribute_specifiers)
        self.type = type

    def genDeclaration(self, declaration):
        from AST import AtomicSpecifier, TypeName

        type_name = TypeName(specifiers=[], declarator=None)
        self.type.genDeclaration(type_name)

        declaration.specifiers.append(AtomicSpecifier(type_name=type_name))

    def __eq__(self, other):
        return isinstance(other, AtomicType) and self.type == other.type

    def is_complete(self):
        return self.type.is_complete()


class TypeofType(Type):
    def __init__(
        self,
        type_or_expr: Union[Type, "Expr"],
        is_unqual: bool = False,
        attribute_specifiers: list["AttributeSpecifier"] = None,
    ):
        super().__init__(attribute_specifiers)
        self.is_unqual = is_unqual
        self.type_or_expr = type_or_expr

    @property
    def is_typeof_expr(self):
        from AST import Expr

        return isinstance(self.type_or_expr, Expr)

    def genDeclaration(self, declaration):
        from AST import TypeName, TypeOfSpecifier

        if isinstance(self.type_or_expr, Type):
            arg = TypeName(specifiers=[], declarator=None)
            self.type_or_expr.genDeclaration(arg)
        else:
            arg = self.type_or_expr

        declaration.specifiers.append(
            TypeOfSpecifier(arg=arg, is_unqual=self.is_unqual)
        )

    def __eq__(self, other):
        return (
            isinstance(other, TypeofType)
            and self.is_unqual == other.is_unqual
            and self.type_or_expr == other.type_or_expr
        )

    def is_complete(self):
        if isinstance(self.type_or_expr, Type):
            return self.type_or_expr.is_complete()
        if not hasattr(self.type_or_expr, "type"):
            return False
        return self.type_or_expr.type.is_complete()


class QualifiedType(Type):
    def __init__(
        self, qualifiers: list["TypeQualifier"], type: Type, attribute_specifiers=None
    ):
        super().__init__(attribute_specifiers)
        self.qualifiers = qualifiers
        self.type = type

    def genDeclaration(self, declaration):
        self.type.genDeclaration(declaration)
        if isinstance(self.type, (PointerType, ArrayType, FunctionType)):
            declaration.declarator.qualifiers.extend(self.qualifiers)
        else:
            declaration.specifiers = self.qualifiers + declaration.specifiers

    def __eq__(self, other):
        return (
            isinstance(other, QualifiedType)
            and self.qualifiers == other.qualifiers
            and self.type == other.type
        )

    def _has_qualifier(self, kind: "TypeQualifierKind"):
        for i in self.qualifiers:
            if i.qualifier == kind:
                return True
        return False

    def has_const(self):
        """拥有const"""
        from AST import TypeQualifierKind

        return self._has_qualifier(TypeQualifierKind.CONST)

    def is_complete(self):
        return self.type.is_complete()


class AutoType(Type):
    def __init__(self, type: Type, attribute_specifiers=None):
        super().__init__(attribute_specifiers)
        self.type = type

    def __str__(self):
        if self.type == None:
            return "auto"
        return f"auto:{self.type}"

    def __eq__(self, other):
        return isinstance(other, AutoType) and self.type == other.type

    def is_complete(self):
        return self.type != None and self.type.is_complete()


class NullPtrType(Type):
    def __str__(self):
        return "nullptr_t"

    def __eq__(self, other):
        return isinstance(other, NullPtrType)


class Char8Type(TypedefType):
    def __init__(self):
        super().__init__("char8_t", BasicType(BasicTypeKind.CHAR))


class Char16Type(TypedefType):
    def __init__(self):
        super().__init__("char16_t", BasicType(BasicTypeKind.SHORT))


class Char32Type(TypedefType):
    def __init__(self):
        super().__init__("char32_t", BasicType(BasicTypeKind.LONG))


class WCharType(TypedefType):
    def __init__(self):
        super().__init__("wchar_t", BasicType(BasicTypeKind.INT))


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
    if not isinstance(type, BasicType):
        return type
    signed = True
    # TODO: 不依赖ctypes
    match type.kind:
        case BasicTypeKind.CHAR:
            size = ctypes.sizeof(ctypes.c_char)
        case BasicTypeKind.SHORT:
            size = ctypes.sizeof(ctypes.c_short)
        case BasicTypeKind.INT:
            size = ctypes.sizeof(ctypes.c_char)
        case BasicTypeKind.UCHAR:
            size = ctypes.sizeof(ctypes.c_ubyte)
            signed = False
        case BasicTypeKind.USHORT:
            size = ctypes.sizeof(ctypes.c_ushort)
            signed = False
        case BasicTypeKind.UINT:
            size = ctypes.sizeof(ctypes.c_ubyte)
            signed = False
        case BasicTypeKind.BOOL:
            size = ctypes.sizeof(ctypes.c_bool)
            signed = False
        case _:
            return type
    if signed:
        interval = -(2 ** (size * 8)), 2 ** (size * 8) - 1
    else:
        interval = 0, 2 ** (size * 8) - 1
    int_size = ctypes.sizeof(ctypes.c_int)
    int_interval = -(2 ** (int_size * 8)), 2 ** (int_size * 8) - 1
    if int_interval[0] <= interval[0] and interval[1] <= int_interval[1]:
        return BasicType(BasicTypeKind.INT)
    return BasicType(BasicTypeKind.UINT)


def is_compatible_type(a: Type, b: Type):
    """判断a和b是否是兼容类型"""
    if a == b:
        return True
    if isinstance(a, TypedefType):
        return is_compatible_type(a.type, b)
    if isinstance(b, TypedefType):
        return is_compatible_type(a, b.type)
    if isinstance(a, QualifiedType) and isinstance(b, QualifiedType):
        return set(a.qualifiers) == set(b.qualifiers) and is_compatible_type(
            a.type, b.type
        )
    if isinstance(a, PointerType) and isinstance(b, PointerType):
        return is_compatible_type(a.pointee_type, b.pointee_type)
    if isinstance(a, ArrayType) and isinstance(b, ArrayType):
        return is_compatible_type(a.element_type, b.element_type) and (
            True if (a.size == None or b.size == None) else a.size == b.size
        )
    if isinstance(a, EnumType) and isinstance(b, EnumType):
        if a.is_complete() and b.is_complete():
            if len(a.enumerators) != len(b.enumerators):
                return False
            if not is_compatible_type(a.underlying_type, b.underlying_type):
                return False
            for x, y in zip(a.enumerators, b.enumerators):
                m = a.enumerators[x]
                n = b.enumerators[y]
                if m.value.value != n.value.value:
                    return False
        return True
    if isinstance(a, RecordType) and isinstance(b, RecordType):
        if a.is_complete() and b.is_complete():
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
    if isinstance(a, EnumType) and a.underlying_type == b:
        return True
    if isinstance(b, EnumType) and b.underlying_type == a:
        return True
    if isinstance(a, FunctionType) and isinstance(b, FunctionType):
        return (
            is_compatible_type(a.return_type, b.return_type)
            and a.has_varparam == b.has_varparam
            and a.parameters_type == b.parameters_type
        )
    return False


def composite_type(a: Type, b: Type) -> Type:
    """合成类型"""
    if isinstance(a, ArrayType) and isinstance(b, ArrayType):
        if a.size == None == b.size:
            return ArrayType(composite_type(a.element_type, b.element_type), None)
        elif hasattr(a.size, "value"):
            return ArrayType(composite_type(a.element_type, b.element_type), a.size)
        elif hasattr(b.size, "value"):
            return ArrayType(composite_type(a.element_type, b.element_type), b.size)
        return ArrayType(composite_type(a.element_type, b.element_type), b.size)
    elif isinstance(a, FunctionType) and isinstance(b, FunctionType):
        parameters_type = []
        for x, y in zip(a.parameters_type, b.parameters_type):
            parameters_type.append(composite_type(x, y))
        return FunctionType(
            parameters_type, a.return_type, a.has_varparam or b.has_varparam
        )
    elif isinstance(a, PointerType) and isinstance(b, PointerType):
        return PointerType(composite_type(a.pointee_type, b.pointee_type))
    raise Exception(f"无法合成{a}和{b}")
