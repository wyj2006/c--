import ctypes
from typing import TYPE_CHECKING, Union
from basic import Symbol, Member, EnumConst

if TYPE_CHECKING:
    from cast import (
        Expr,
        TypeQualifier,
        AttributeSpecifier,
        SingleDeclration,
        TypeQualifierKind,
    )


class Type:
    """类型符号"""

    is_integer_type: bool = False  # 整数类型
    is_arithmetic_type: bool = False  # 算术类型
    is_decimal_floating_type: bool = False  # 十进制浮点数类型
    is_real_floating_type: bool = False  # 实浮点类型
    is_real_type: bool = False  # 实数类型
    is_scalar_type: bool = False  # 标量类型

    size: int  # 大小

    def __init__(self, attribute_specifiers: list["AttributeSpecifier"] = None):
        super().__init__()
        self.attribute_specifiers = (
            attribute_specifiers if attribute_specifiers != None else []
        )

    def __str__(self):
        from cast import SingleDeclration, UnParseVisitor

        declaration = SingleDeclration(specifiers=[], declarator=None)
        self.genDeclaration(declaration)
        return declaration.accept(UnParseVisitor())

    def genDeclaration(self, declaration: "SingleDeclration"):
        pass

    def is_complete(self) -> bool:
        """是否是完整类型"""
        return True


class VoidType(Type):
    size = 1

    def genDeclaration(self, declaration):
        from cast import BasicTypeSpecifier

        declaration.specifiers.append(BasicTypeSpecifier(specifier_name="void"))


class BoolType(Type):
    size = 1

    def genDeclaration(self, declaration):
        from cast import BasicTypeSpecifier

        declaration.specifiers.append(BasicTypeSpecifier(specifier_name="bool"))


class PointerType(Type):
    is_scalar_type = True

    size = ctypes.sizeof(ctypes.c_void_p)

    def __init__(
        self,
        pointee_type: Type,
        attribute_specifiers: list["AttributeSpecifier"] = None,
    ):
        super().__init__(attribute_specifiers)
        self.pointee_type = pointee_type

    def genDeclaration(self, declaration):
        from cast import PointerDeclarator

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
        size_expr: "Expr",
        is_star_modified: bool = False,
        is_static: bool = False,
        attribute_specifiers: list["AttributeSpecifier"] = None,
    ):
        super().__init__(attribute_specifiers)
        self.element_type = element_type
        self.size_expr = size_expr
        self.is_star_modified = is_star_modified
        self.is_static = is_static

    def __eq__(self, other):
        return (
            isinstance(other, ArrayType)
            and self.element_type == other.element_type
            and self.size_expr == other.size_expr
            and self.is_star_modified == other.is_star_modified
            and self.is_static == other.is_static
        )

    def genDeclaration(self, declaration):
        from cast import ArrayDeclarator

        declarator = ArrayDeclarator(
            size=self.size_expr,
            is_star_modified=self.is_star_modified,
            is_static=self.is_static,
            qualifiers=[],
            declarator=declaration.declarator,
        )
        declaration.declarator = declarator
        self.element_type.genDeclaration(declaration)

    def is_complete(self):
        return self.element_type.is_complete() and self.size_expr != None

    @property
    def size(self):
        return self.element_type.size * self.size_expr.value


class ArrayPtrType(PointerType):
    """函数参数声明中由数组转换过来的指针类型"""

    def __init__(self, array_type: ArrayType):
        super().__init__(array_type.element_type)
        self.array_type = array_type


class FunctionType(Type):
    size = 1

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
        from cast import FunctionDeclarator, ParamDecl

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


class RecordType(Type, Symbol):
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
        from cast import RecordDecl

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

    @property
    def size(self):
        # TODO: 确定对齐
        _size = 0
        for member in self.members:
            _size += member.type.size
        return _size


class EnumType(Type, Symbol):
    is_integer_type = True
    is_real_type = True
    is_arithmetic_type = True
    is_scalar_type = True

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
        from cast import EnumDecl

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

    @property
    def size(self):
        return self.underlying_type.size


class AtomicType(Type):
    def __init__(
        self, type: Type, attribute_specifiers: list["AttributeSpecifier"] = None
    ):
        super().__init__(attribute_specifiers)
        self.type = type

    def genDeclaration(self, declaration):
        from cast import AtomicSpecifier, TypeName

        type_name = TypeName(specifiers=[], declarator=None)
        self.type.genDeclaration(type_name)

        declaration.specifiers.append(AtomicSpecifier(type_name=type_name))

    def __eq__(self, other):
        return isinstance(other, AtomicType) and self.type == other.type

    def is_complete(self):
        return self.type.is_complete()

    @property
    def size(self):
        return self.type.size


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
        from cast import Expr

        return isinstance(self.type_or_expr, Expr)

    def genDeclaration(self, declaration):
        from cast import TypeName, TypeOfSpecifier

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

    @property
    def size(self):
        if isinstance(self.type_or_expr, Type):
            return self.type_or_expr.size
        return self.type_or_expr.type.size


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
        from cast import TypeQualifierKind

        return self._has_qualifier(TypeQualifierKind.CONST)

    def is_complete(self):
        return self.type.is_complete()

    @property
    def size(self):
        return self.type.size


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

    @property
    def size(self):
        return self.type.size


class NullPtrType(Type):
    is_scalar_type = True

    size = ctypes.sizeof(ctypes.c_voidp)

    def __str__(self):
        return "nullptr_t"

    def __eq__(self, other):
        return isinstance(other, NullPtrType)
