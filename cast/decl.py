from enum import Enum
from typing import TYPE_CHECKING, Union
from typesystem import Type, EnumType, RecordType, FunctionType
from .node import Node, AttributeSpecifier
from .stmt import Stmt, CompoundStmt

if TYPE_CHECKING:
    from .expr import Expr


class Declaration(Node):
    # 虽然叫specifiers, 却还包含了storageclass和qualifier
    specifiers: list["SpecifierOrQualifier"]
    specifier_attributes: list[AttributeSpecifier]  # 跟在说明符后面的可选的属性列表
    declarators: list["Declarator"]

    _fields = Node._fields + (
        "specifiers",
        "specifier_attributes",
        "declarators",
    )


class SingleDeclration(Declaration):
    declarator: "Declarator"

    @property
    def declarators(self):
        if self.declarator == None:
            return []
        return [self.declarator]

    _fields = list(Declaration._fields)
    _fields.remove("declarators")
    _fields = tuple(_fields) + ("declarator",)


class DeclStmt(Stmt, Declaration):
    """声明语句"""

    _fields = Stmt._fields + Declaration._fields


class SpecifierOrQualifier(Node):
    pass


class StorageClassSpecifier(Enum):
    AUTO = "auto"
    CONSTEXPR = "contexpr"
    EXTERN = "extern"
    REGISTER = "register"
    STATIC = "static"
    THREAD_LOCAL = "thread_local"
    TYPEDEF = "typedef"


class StorageClass(SpecifierOrQualifier):
    specifier: StorageClassSpecifier

    _attributes = SpecifierOrQualifier._attributes + ("specifier",)


class Declarator(Node):
    declarator: "Declarator"

    _fields = Node._fields + ("declarator",)


class TypeOrVarDecl(Declarator):
    name: str
    type: Type

    declaration: Declaration  # 相当于父节点
    storage_classes: list[StorageClass]  # 由DeclAnalyzer设置

    @property
    def function_specifiers(self) -> list["FunctionSpecifier"]:
        if hasattr(self, "declaration"):
            return [
                i
                for i in self.declaration.specifiers
                if isinstance(i, FunctionSpecifier)
            ]
        return []

    @property
    def align_specifier(self) -> "AlignSpecifier":
        if hasattr(self, "declaration"):
            for i in self.declaration.specifiers:
                if isinstance(i, AlignSpecifier):
                    return i
        return None

    @property
    def is_typedef(self):
        if not hasattr(self, "storage_classes"):
            return False
        for storage_class in self.storage_classes:
            if storage_class.specifier == StorageClassSpecifier.TYPEDEF:
                return True
        return False

    @property
    def attribute_specifiers(self):
        return self.declaration.attribute_specifiers

    initializer: "Expr"

    _attributes = Declarator._attributes + ("name", "type", "is_typedef")
    _fields = Declarator._fields + ("initializer",)


class PointerDeclarator(Declarator):
    qualifiers: list["TypeQualifier"]

    _fields = Declarator._fields + ("qualifiers",)


class ArrayDeclarator(Declarator):
    qualifiers: list["TypeQualifier"]
    size: "Expr"
    is_star_modified: bool
    is_static: bool

    attribute = Declarator._attributes + ("is_star_modified", "is_static")
    _fields = Declarator._fields + ("qualifiers", "size")


class FunctionDeclarator(Declarator):
    parameters: list["ParamDecl"]
    has_varparam: bool

    _attributes = Declarator._attributes + ("has_varparam",)
    _fields = Declarator._fields + ("parameters",)


class NameDeclarator(Declarator):
    name: str

    _attributes = Declarator._attributes + ("name",)


class BasicTypeSpecifier(SpecifierOrQualifier):
    specifier_name: str

    _attributes = SpecifierOrQualifier._attributes + ("specifier_name",)


class BitIntSpecifier(SpecifierOrQualifier):
    size: "Expr"

    _fields = SpecifierOrQualifier._fields + ("size",)


class AtomicSpecifier(SpecifierOrQualifier):
    type_name: "TypeName"

    _fields = SpecifierOrQualifier._fields + ("type_name",)


class TypeOfSpecifier(SpecifierOrQualifier):
    arg: Union["Expr", "TypeName"]
    is_unqual: bool

    _attributes = SpecifierOrQualifier._attributes + ("is_unqual",)
    _fields = SpecifierOrQualifier._fields + ("arg",)


class TypeQualifierKind(Enum):
    CONST = "const"
    RESTRICT = "restrict"
    VOLATILE = "volatile"
    _ATOMIC = "_Atomic"


class TypeQualifier(SpecifierOrQualifier):
    qualifier: TypeQualifierKind

    _attributes = SpecifierOrQualifier._attributes + ("qualifier",)


class FunctionSpecifier(SpecifierOrQualifier):
    specifier_name: str

    _attributes = SpecifierOrQualifier._attributes + ("specifier_name",)


class TypedefSpecifier(SpecifierOrQualifier):
    specifier_name: str

    _attributes = SpecifierOrQualifier._attributes + ("specifier_name",)


class AlignSpecifier(SpecifierOrQualifier):
    type_or_expr: Node

    _attributes = SpecifierOrQualifier._attributes + ("type_or_expr",)


class RecordDecl(SpecifierOrQualifier):
    struct_or_union: str
    name: str
    type: RecordType
    members_declaration: list[Node]

    _attributes = SpecifierOrQualifier._attributes + ("struct_or_union", "name", "type")
    _fields = SpecifierOrQualifier._fields + ("members_declaration",)


class FieldDecl(Declaration):
    pass


class MemberDecl(TypeOrVarDecl):
    initializer = None

    bit_field: "Expr"

    _fields = Declarator._fields + ("bit_field",)


class Enumerator(Node):
    name: str
    value: "Expr"
    enum_type: EnumType

    _attributes = Node._attributes + ("name", "enum_type")
    _fields = Node._fields + ("value",)


class EnumDecl(SpecifierOrQualifier):
    name: str
    type: EnumType
    underlying_type: Type
    specifiers: list[SpecifierOrQualifier]
    enumerators: list[Enumerator]

    _attributes = SpecifierOrQualifier._attributes + ("name", "underlying_type")
    _fields = SpecifierOrQualifier._fields + (
        "specifiers",
        "enumerators",
    )


class ParamDecl(SingleDeclration):
    name: str
    type: Type

    _attributes = SingleDeclration._attributes + ("name", "type")


class TypeName(SingleDeclration):
    type: Type

    _attributes = SingleDeclration._attributes + ("type",)


class FunctionDef(SingleDeclration):
    func_name: str
    func_type: FunctionType
    body: CompoundStmt

    _attributes = SingleDeclration._attributes + ("func_name", "func_type")

    _fields = SingleDeclration._fields + ("body",)
