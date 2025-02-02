from enum import Enum
from typing import TYPE_CHECKING, TypedDict, Union
from Basic.Location import Location

if TYPE_CHECKING:
    from AST import Expr, TypeQualifier, AttributeSpecifier

# 提供给符号表的命名空间名
LABEL_NAMES = "label names"
TAG_NAMES = "tag names"
MEMBER_NAMES = "member names"
ORDINARY_NAMES = "ordinary identifiers"
# TODO: 属性命名空间


class Symtab:
    """
    符号表
    一个作用域对应一个符号表
    """

    def __init__(self, begin_location: Location = None):
        self.parent: Symtab = None
        self.children: list[Symtab] = []  # 嵌套的作用域
        self.begin_location: Location = begin_location  # 作用域开始位置
        self.namespaces: dict[str, Namespace] = {  # TODO: 属性命名空间
            LABEL_NAMES: Namespace(LABEL_NAMES),
            TAG_NAMES: Namespace(TAG_NAMES),
            ORDINARY_NAMES: Namespace(ORDINARY_NAMES),
        }

    def enterScope(self, begin_location: Location):
        for child in self.children:
            if child.begin_location == begin_location:
                break
        else:
            child = Symtab(begin_location)
            child.parent = self
            self.children.append(child)
        return child

    def leaveScope(self):
        return self.parent

    def lookup(self, name: str, namespace_name=ORDINARY_NAMES):
        p = self
        while p != None:
            try:
                return p.namespaces[namespace_name].lookup(name)
            except KeyError:
                p = p.parent
        return None

    def addSymbol(self, name: str, symbol: "Symbol", namespace_name=ORDINARY_NAMES):
        return self.namespaces[namespace_name].addSymbol(name, symbol)

    def print(self, indent=0):
        print(" " * 4 * indent, self)
        for namespace_name in self.namespaces:
            print(" " * 4 * (indent + 1), namespace_name, ":")
            self.namespaces[namespace_name].print(indent + 2)
        for child in self.children:
            child.print(indent + 1)

    def __str__(self):
        return f"<Symtab at {id(self)} from {self.begin_location}>"


class Symbol:
    """符号表中的符号"""

    def __init__(self, attribute_specifiers: list["AttributeSpecifier"] = None):
        # TODO: 位置信息
        self.attribute_specifiers = (
            attribute_specifiers if attribute_specifiers != None else []
        )


class Namespace(Symbol):
    """命名空间"""

    def __init__(self, name: str):
        super().__init__()
        self.name = name
        self.symbols: dict[str, Symbol] = {}

    def lookup(self, name):
        return self.symbols[name]

    def addSymbol(self, name: str, symbol: Symbol):
        """返回False说明重定义"""
        if name in self.symbols:
            return False
        self.symbols[name] = symbol
        return True

    def print(self, indent=0):
        for name, symbol in self.symbols.items():
            print(" " * 4 * indent, name, ":", symbol)


class Variable(Symbol):
    """变量符号"""


class TypeStrDict(TypedDict):
    """
    生成类型字符串的中间体
    比如对于类型 int*(*[1])[2] 该字典为
    {
        "specifier":"int",
        "declarators":[
            {
                "pointer":"*",
                "direct":"[1]",
            },
            {
                "pointer:"*",
                "direct":"[2]"
            }
        ]
    }
    """

    specifier: str
    declarators: list


class Type(Symbol):
    """类型符号"""

    def __str__(self):
        typestrdict = {"specifier": "", "declarators": [{"pointer": "", "direct": ""}]}
        self.genString(typestrdict)
        declarators = ""
        for i, declarator in enumerate(typestrdict["declarators"]):
            declarators = declarator["pointer"] + declarators
            declarators += declarator["direct"]
            if i < len(typestrdict["declarators"]) - 1:
                declarators = "(" + declarators + ")"  # 体现优先级
        return typestrdict["specifier"] + declarators

    def genString(self, typestrdict: TypeStrDict):
        assert False, "你应该自己实现这个方法"


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
    UNSIGNEDLONGLONG = "unsigned long long "
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

    def genString(self, typestrdict):
        typestrdict["specifier"] = self.kind.value


class BitIntType(Type):
    def __init__(self, size: "Expr", signed=True):
        super().__init__()
        self.size = size
        self.signed = signed

    def genString(self, typestrdict):
        from AST import FormatVisitor

        typestrdict["specifier"] = (
            f"{'unsigned ' if not self.signed else ''}_BitInt({self.size.accept(FormatVisitor())})"
        )


class PointerType(Type):
    def __init__(
        self,
        pointee_type: Type,
        attribute_specifiers: list["AttributeSpecifier"] = None,
    ):
        super().__init__(attribute_specifiers)
        self.pointee_type = pointee_type

    def genString(self, typestrdict):
        typestrdict["declarators"][-1]["pointer"] += "*"
        if isinstance(self.pointee_type, (ArrayType, FunctionType)):
            # 优先级
            typestrdict["declarators"].append({"pointer": "", "direct": ""})
        self.pointee_type.genString(typestrdict)


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

    def genString(self, typestrdict):
        from AST import FormatVisitor

        typestrdict["declarators"][-1][
            "direct"
        ] += f"[{self.size.accept(FormatVisitor()) if self.size!=None else ''}]"
        self.element_type.genString(typestrdict)


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

    def genString(self, typestrdict):
        self.return_type.genString(typestrdict)
        direct = typestrdict["declarators"][-1]["direct"]
        direct += "("
        for i, parameter_type in enumerate(self.parameters_type):
            direct += str(parameter_type)
            if i < len(self.parameters_type) - 1:
                direct += ", "
        if self.has_varparam:
            direct += ",..."
        direct += ")"
        typestrdict["declarators"][-1]["direct"] = direct


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
        self.member: Namespace = None  # Namespace(MEMBER_NAMES)

    def genString(self, typestrdict):
        typestrdict["specifier"] = f"{self.struct_or_union} {self.name}"


class TypedefType(Type):
    def __init__(self, name: str, type: Type):
        super().__init__()
        self.name = name
        self.type = type

    def genString(self, typestrdict):
        typestrdict["specifier"] = f"{self.name}:{self.type}"


class EnumType(Type):
    def __init__(
        self, name: str, attribute_specifiers: list["AttributeSpecifier"] = None
    ):
        super().__init__(attribute_specifiers)
        self.name = name
        self.enumerator: Symtab = None

    def genString(self, typestrdict):
        typestrdict["specifier"] = f"enum {self.name}"


class AtomicType(Type):
    def __init__(self, type: Type):
        super().__init__()
        self.type = type

    def genString(self, typestrdict):
        typestrdict["specifier"] = f"_Atomic({self.type})"


class TypeofType(Type):
    def __init__(self, type_or_expr: Union[Type, "Expr"], is_unqual: bool = False):
        super().__init__()
        self.is_unqual = is_unqual
        self.type_or_expr = type_or_expr

    @property
    def isTypeofExpr(self):
        from AST import Expr

        return isinstance(self.type_or_expr, Expr)

    def genString(self, typestrdict):
        from AST import FormatVisitor

        typestrdict["specifier"] = f"typeof{'_unqual' if self.is_unqual else ''}("
        typestrdict["specifier"] += (
            str(self.type_or_expr)
            if not self.isTypeofExpr
            else self.type_or_expr.accept(FormatVisitor())
        )
        typestrdict["specifier"] += ")"


class QualifiedType(Type):
    def __init__(self, qualifiers: list["TypeQualifier"], type: Type):
        super().__init__()
        self.qualifiers = qualifiers
        self.type = type

    def genString(self, typestrdict):
        self.type.genString(typestrdict)
        qualifier_str = " ".join([i.qualifier.value for i in self.qualifiers])
        if isinstance(self.type, PointerType):
            typestrdict["declarators"][-1]["pointer"] += qualifier_str
        else:
            typestrdict["specifier"] = qualifier_str + " " + typestrdict["specifier"]
