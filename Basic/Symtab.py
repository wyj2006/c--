from typing import TYPE_CHECKING
from Basic.Location import Location

if TYPE_CHECKING:
    from AST import (
        Expr,
        AttributeSpecifier,
        StorageClass,
        FunctionSpecifier,
        AlignSpecifier,
    )
    from Types import FunctionType, Type, EnumType

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
            "": {
                "deprecated": DeprecatedAttr,
                "fallthrough": FallthroughAttr,
                "maybe_unused": MaybeUnusedAttr,
                "nodiscard": NodiscardAttr,
                "noreturn": NoReturnAttr,
                "_Noreturn": NoReturnAttr,
                "unsequenced": UnsequencedAttr,
                "reproducible": ReproducibleAttr,
            }
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

    def removeSymbol(self, name: str, namespace_name=ORDINARY_NAMES):
        """
        移除符号
        成功返回True, 失败返回False
        """
        namespace = getattr(self, namespace_name, {})
        if name not in namespace:
            return False
        namespace.pop(name)
        return True

    def print(self, indent=0):
        print(" " * 4 * indent, self)
        for namespace_name in (LABEL_NAMES, TAG_NAMES, ORDINARY_NAMES):
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

    @property
    def init_value(self):
        return self.initializer.value

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
        value_expr: "Expr",
        attribute_specifiers=None,
    ):
        super().__init__(attribute_specifiers)
        self.name = name
        self.enum_type = enum_type
        self.value_expr = value_expr

    @property
    def value(self):
        return self.value_expr.value

    def __str__(self):
        from AST import UnParseVisitor

        value_str = (
            self.value_expr.accept(UnParseVisitor())
            if self.value_expr != None
            else "Auto"
        )

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
