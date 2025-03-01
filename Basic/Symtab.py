from enum import Enum
from typing import TYPE_CHECKING, Union
from Basic.Location import Location

if TYPE_CHECKING:
    from AST import (
        Expr,
        TypeQualifier,
        AttributeSpecifier,
        StorageClass,
        FunctionSpecifier,
        AlignSpecifier,
        SingleDeclration,
    )

# 提供给符号表的命名空间名
LABEL_NAMES = "label_names"
TAG_NAMES = "tag_names"
MEMBER_NAMES = "member_names"
ORDINARY_NAMES = "ordinary_names"
ATTRIBUTE_NAMES = "attribute_names"


class Symtab:
    """
    符号表
    一个作用域对应一个符号表
    """

    def __init__(self, begin_location: Location = None, parent: "Symtab" = None):
        self.parent: Symtab = parent
        self.children: list[Symtab] = []  # 嵌套的作用域
        self.begin_location: Location = begin_location  # 作用域开始位置

        self.label_names = {}
        self.tag_names = {}
        self.ordinary_names = {}
        self.member_names = {}

    @property
    def attribute_names(self):
        from AST import (
            NoReturnAttr,
            NodiscardAttr,
            DeprecatedAttr,
            FallthroughAttr,
            MaybeUnusedAttr,
            UnsequencedAttr,
            ReproducibleAttr,
        )

        return {
            "deprecated": DeprecatedAttr,
            "fallthrough": FallthroughAttr,
            "maybe_unused": MaybeUnusedAttr,
            "nodiscard": NodiscardAttr,
            "noreturn": NoReturnAttr,
            "_Noreturn": NoReturnAttr,
            "unsequenced": UnsequencedAttr,
            "reproducible": ReproducibleAttr,
        }

    def enterScope(self, begin_location: Location):
        for child in self.children:
            if child.begin_location == begin_location:
                break
        else:
            child = Symtab(begin_location, self)
            self.children.append(child)
        return child

    def leaveScope(self):
        return self.parent

    def lookup(self, name: str, namespace_name=ORDINARY_NAMES):
        p: Symtab = self
        while p != None:
            try:
                return getattr(p, namespace_name, {})[name]
            except KeyError:
                p = p.parent
        return None

    def addSymbol(self, name: str, symbol: "Symbol", namespace_name=ORDINARY_NAMES):
        """
        添加符号
        如果符号已存在, 返回False, 否则返回True
        """
        namespace = getattr(self, namespace_name, {})
        if name in namespace:
            return False
        namespace[name] = symbol
        return True

    def print(self, indent=0):
        print(" " * 4 * indent, self)
        for namespace_name in (LABEL_NAMES, TAG_NAMES, ORDINARY_NAMES, ORDINARY_NAMES):
            namespace: dict = getattr(self, namespace_name)
            print(
                " " * 4 * (indent + 1),
                namespace_name,
                f"({len(namespace)})",
                ":",
            )
            for name, symbol in namespace.items():
                print(" " * 4 * (indent + 2), name, ":", symbol)
        for child in self.children:
            child.print(indent + 1)

    def __str__(self):
        return f"<Symtab at {id(self)} from {self.begin_location}>"


class Symbol:
    """符号表中的符号"""

    def __init__(self, attribute_specifiers: list["AttributeSpecifier"] = None):
        self.define_location: Location = None
        self.attribute_specifiers = (
            attribute_specifiers if attribute_specifiers != None else []
        )

        self.define_location: Location = None  # 定义的位置
        self.declare_locations: list[Location] = []  # 声明的位置


class Object(Symbol):
    """对象"""

    def __init__(
        self,
        name: str,
        type: "Type",
        storage_classes: list["StorageClass"] = None,
        align_specifier: "AlignSpecifier" = None,
        attribute_specifiers=None,
    ):
        super().__init__(attribute_specifiers)
        self.storage_classes = storage_classes if storage_classes != None else []
        self.name = name
        self.type = type
        self.align_specifier = align_specifier
        self.initializer: "Expr" = None

    def __str__(self):
        return f"{self.__class__.__name__}({self.name}, {self.type})"


class Member(Object):
    """Record的成员"""

    def __init__(
        self,
        name: str,
        type: "Type",
        bit_field: "Expr" = None,
        storage_classes: list["StorageClass"] = None,
        align_specifier: "AlignSpecifier" = None,
        attribute_specifiers=None,
    ):
        super().__init__(
            name, type, storage_classes, align_specifier, attribute_specifiers
        )
        self.bit_field = bit_field


class Parameter(Object):
    """函数参数"""


class EnumConst(Symbol):
    """枚举常量"""

    def __init__(
        self,
        name,
        enum_type: "EnumType",
        value: "Expr",
        attribute_specifiers=None,
    ):
        super().__init__(attribute_specifiers)
        self.name = name
        self.enum_type = enum_type
        self.value = value

    def __str__(self):
        from AST import FormatVisitor

        value_str = self.value.accept(FormatVisitor()) if self.value != None else "Auto"

        return f"{self.__class__.__name__}({self.name}, {self.enum_type}, {value_str})"


class Function(Symbol):
    """函数"""

    def __init__(
        self,
        name,
        type: "FunctionType",
        function_specifiers: list["FunctionSpecifier"] = None,
        attribute_specifiers=None,
    ):
        super().__init__(attribute_specifiers)
        self.name = name
        self.type = type
        self.function_specifiers = (
            function_specifiers if function_specifiers != None else []
        )

    def __str__(self):
        return f"{self.__class__.__name__}({self.name}, {self.type})"


class Type(Symbol):
    """类型符号"""

    def __str__(self):
        from AST import SingleDeclration, FormatVisitor

        declaration = SingleDeclration(specifiers=[], declarator=None)
        self.genDeclaration(declaration)
        return declaration.accept(FormatVisitor())

    def genDeclaration(self, declaration: "SingleDeclration"):
        pass


class BasicTypeKind(Enum):
    VOID = "void"
    CHAR = "char"
    SHORT = "short"
    INT = "int"
    LONG = "long"
    LONGLONG = "long long"
    _BOOL = "_Bool"
    UNSIGNEDCHAR = "unsigned char"
    UNSIGNEDSHORT = "unsigned short"
    UNSIGNEDINT = "unsigned int"
    UNSIGNEDLONG = "unsigned long"
    UNSIGNEDLONGLONG = "unsigned long long"
    FLOAT = "float"
    DOUBLE = "double"
    LONGDOUBLE = "long double"
    _DECIMAL32 = "_Decimal32"
    _DECIMAL64 = "_Decimal64"
    _DECIMAL128 = "_Decimal128"
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
            "_Bool": BasicTypeKind._BOOL,
            "float": BasicTypeKind.FLOAT,
            "double": BasicTypeKind.DOUBLE,
            "_Decimal32": BasicTypeKind._DECIMAL32,
            "_Decimal64": BasicTypeKind._DECIMAL64,
            "_Decimal128": BasicTypeKind._DECIMAL128,
            "signed": BasicTypeKind.INT,
            "unsigned": BasicTypeKind.UNSIGNEDINT,
        },
        BasicTypeKind.CHAR: {
            "signed": BasicTypeKind.CHAR,
            "unsigned": BasicTypeKind.UNSIGNEDCHAR,
        },
        BasicTypeKind.SHORT: {
            "signed": BasicTypeKind.SHORT,
            "unsigned": BasicTypeKind.UNSIGNEDSHORT,
        },
        BasicTypeKind.INT: {
            "signed": BasicTypeKind.INT,
            "unsigned": BasicTypeKind.UNSIGNEDINT,
            "short": BasicTypeKind.SHORT,
            "long": BasicTypeKind.LONG,
        },
        BasicTypeKind.LONG: {
            "signed": BasicTypeKind.LONG,
            "unsigned": BasicTypeKind.UNSIGNEDLONG,
            "long": BasicTypeKind.LONGLONG,
            "double": BasicTypeKind.LONGDOUBLE,
        },
        BasicTypeKind.LONGLONG: {
            "signed": BasicTypeKind.LONGLONG,
            "unsigned": BasicTypeKind.UNSIGNEDLONGLONG,
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
