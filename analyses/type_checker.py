from typing import Callable, TypeVar, TypedDict, Generator, Union
from analyses.analyzer import Analyzer
from cast import (
    Transformer,
    Reference,
    StringLiteral,
    CompoundLiteral,
    ArraySubscript,
    UnaryOperator,
    UnaryOpKind,
    BinaryOperator,
    BinOpKind,
    TypeOrVarDecl,
    FunctionCall,
    ReturnStmt,
    FunctionDef,
    MemberRef,
    ExplicitCast,
    ConditionalOperator,
    GenericSelection,
    Expr,
    Designation,
    BoolLiteral,
    IntegerLiteral,
    NullPtrLiteral,
    FloatLiteral,
    ImaginaryUnit,
    CharLiteral,
    InitList,
    EnumDecl,
    ImplicitCast,
    Node,
    TypeOfSpecifier,
    TypeQualifier,
    TypeQualifierKind,
    Designator,
)
from basic import Error, Object, Function, EnumConst
from typesystem import (
    NullPtrType,
    PointerType,
    FunctionType,
    remove_qualifier,
    is_compatible_type,
    RecordType,
    QualifiedType,
    AtomicType,
    EnumType,
    ArrayPtrType,
    composite_type,
    IntType,
    VoidType,
    FloatType,
    DoubleType,
    SizeType,
    BoolType,
    BitIntType,
    LongDoubleType,
    LongLongType,
    LongType,
    UIntType,
    ULongLongType,
    ULongType,
    Decimal128Type,
    Decimal32Type,
    Decimal64Type,
    CharType,
    Char8Type,
    Char16Type,
    Char32Type,
    WCharType,
    ArrayType,
    FloatImaginaryType,
    Type,
    remove_atomic,
    integer_promotion,
    DecimalType,
    ComplexType,
    ImaginaryType,
    BinaryFloatType,
    LongDoubleComplexType,
    LongDoubleImaginaryType,
    IntegerType,
    RealFloatingType,
    FloatComplexType,
    DoubleComplexType,
    DoubleImaginaryType,
)
from values import Imaginary


def encode_units(string: str, encoding: str):
    """
    返回string在encoding编码下各编码单元的值
    """
    value = []
    byte_array = string.encode(encoding)
    byte_order = "le"  # 字节序
    unit_len = 1  # 编码单元长度
    i = 0
    if encoding == "utf-16":
        if byte_array.startswith(b"\xff\xfe"):
            byte_order = "le"
        elif byte_array.startswith(b"\xfe\xff"):
            byte_order = "be"
        i = 2
        unit_len = 2
    elif encoding == "utf-32":
        if byte_array.startswith(b"\xff\xfe\x00\x00"):
            byte_order = "le"
        elif byte_array.startswith(b"\x00\x00\xfe\xff"):
            byte_order = "be"
        i = 4
        unit_len = 4
    elif encoding == "utf-8":
        pass  # 默认utf-8
    else:
        raise Exception(f"不支持的编码{encoding}")
    while i < len(byte_array):
        t = byte_array[i : i + unit_len]
        v = 0

        if byte_order == "le":
            for j in t[::-1]:
                v = v << 8 | j
        else:
            for j in t:
                v = v << 8 | j
        value.append(v)

        i += unit_len
    return value


_T = TypeVar("_T")


def is_lvalue(node: Expr):
    """判断node是否是左值表达式"""
    if hasattr(node, "is_lvalue"):
        return node.is_lvalue

    if isinstance(node, Reference):
        if isinstance(node.symbol, (Function, EnumConst)):
            return False
        return True
    if isinstance(node, (StringLiteral, CompoundLiteral)):
        return True
    if isinstance(node, MemberRef):
        return (
            node.is_arrow == False and is_lvalue(node.target) or node.is_arrow == True
        )
    if isinstance(node, UnaryOperator) and node.op == UnaryOpKind.DEREFERENCE:
        return True
    if isinstance(node, ArraySubscript):
        return True
    return False


def is_modifiable_lvalue(node: Expr):
    """判断是否是可修改左值"""
    if isinstance(node.type, QualifiedType) and node.type.has_const():
        return False
    if isinstance(node.type, ArrayType):
        return False
    if not node.type.is_complete:
        return False
    if isinstance(node.type, RecordType):
        for name, member in node.type.members.items():
            if isinstance(member.type, QualifiedType) and member.type.has_const():
                return False
    return True


def generic_implicit_cast(func: Callable[["TypeChecker", Node], _T]):
    """
    对访问函数的返回值进行通用的隐式转换
    同时会维护TypeChecker.path
    """

    def wrapper(self: "TypeChecker", node: Node) -> _T:
        self.path.append(node)
        ret = func(self, node)
        self.path.pop()

        if not isinstance(node, Expr):
            return ret
        if not hasattr(node, "type"):
            return ret

        if isinstance(node.type, ArrayType):  # 数组到指针转换
            match self.path:
                case [*_, UnaryOperator() as a] if a.op in (
                    UnaryOpKind.ADDRESS,  # 作为取址运算符的操作数
                    UnaryOpKind.SIZEOF,  # 作为 sizeof 的操作数
                ):
                    return ret
                case [*_, TypeOfSpecifier()]:  # 作为 typeof 和 typeof_unqual 的操作数
                    return ret
                # 作为用于数组初始化的字符串字面量
                case [*_, TypeOrVarDecl()] if isinstance(node, StringLiteral):
                    return ret
            return self.implicit_cast(node, PointerType(node.type.element_type))

        if isinstance(node.type, FunctionType):  # 函数到指针转换
            match self.path:
                case [*_, UnaryOperator() as a] if a.op in (
                    UnaryOpKind.ADDRESS,  # 作为取址运算符的操作数
                    UnaryOpKind.SIZEOF,  # 作为 sizeof 的操作数
                ):
                    return ret
                case [*_, TypeOfSpecifier()]:
                    return ret
            return self.implicit_cast(node, PointerType(node.type))

        if is_lvalue(node) and not isinstance(node.type, ArrayType):  # 左值转换
            match self.path:
                case [*_, UnaryOperator() as a] if a.op in (
                    UnaryOpKind.ADDRESS,  # 作为取址运算符的操作数
                    UnaryOpKind.SIZEOF,  # 作为 sizeof 的操作数
                    # 作为前/后自增减运算符的操作数
                    UnaryOpKind.POSTFIX_DEC,
                    UnaryOpKind.PREFIX_DEC,
                    UnaryOpKind.POSTFIX_INC,
                    UnaryOpKind.PREFIX_INC,
                ):
                    return ret
                # 作为赋值与复合赋值运算符的左操作数
                case [*_, BinaryOperator() as a] if (
                    a.op == BinOpKind.ASSIGN and a.left == node
                ):
                    return ret
                # 作为成员访问（点）运算符的左操作数
                case [*_, MemberRef() as a] if a.target == node and a.is_arrow == False:
                    return ret
            return self.implicit_cast(node, remove_atomic(remove_qualifier(node.type)))

        return ret

    return wrapper


class TypeChecker(Analyzer, Transformer):
    """
    完成的任务:
    1. 确定各表达式的类型, 必要时进行隐式类型转换
    2. 计算常量表达式的值
    3. 确定枚举值及枚举类型的底层类型
    4. 确定不完整数组类型的大小
    """

    operator = {
        BinOpKind.ADD: lambda a, b: a + b,
        BinOpKind.SUB: lambda a, b: a - b,
        BinOpKind.MUL: lambda a, b: a * b,
        BinOpKind.DIV: lambda a, b: a / b,
        BinOpKind.MOD: lambda a, b: a % b,
        BinOpKind.LSHIFT: lambda a, b: a << b,
        BinOpKind.RSHIFT: lambda a, b: a >> b,
        BinOpKind.EQ: lambda a, b: a == b,
        BinOpKind.NEQ: lambda a, b: a != b,
        BinOpKind.GT: lambda a, b: a > b,
        BinOpKind.GTE: lambda a, b: a >= b,
        BinOpKind.LT: lambda a, b: a < b,
        BinOpKind.LTE: lambda a, b: a <= b,
        BinOpKind.BITAND: lambda a, b: a & b,
        BinOpKind.BITOR: lambda a, b: a | b,
        BinOpKind.BITXOR: lambda a, b: a ^ b,
        BinOpKind.AND: lambda a, b: a and b,
        BinOpKind.OR: lambda a, b: a or b,
        UnaryOpKind.POSITIVE: lambda a: +a,
        UnaryOpKind.NEGATIVE: lambda a: -a,
        UnaryOpKind.INVERT: lambda a: ~a,
        UnaryOpKind.NOT: lambda a: a.__not__(),
    }

    def __init__(self, symtab):
        super().__init__(symtab)
        self.path = []  # 调用路径

    def implicit_cast(self, node: Expr, type: Type):
        """
        将node的类型隐式转换为type, 并返回转换后的节点
        transformer被用于InitList进行隐式转换
        """
        if hasattr(node, "type") and is_compatible_type(node.type, type):
            return node
        if isinstance(node, InitList):
            node.type = type
            return node.accept(self)
        elif isinstance(node, Designation):
            node.initializer = self.implicit_cast(node.initializer, type)
            return node
        if (
            isinstance(type, ArrayType)
            and not type.is_complete
            and isinstance(node.type, ArrayType)
            and node.type.is_complete
        ):
            type.len_expr = node.type.len_expr
        ret = ImplicitCast(
            type=type,
            expr=node,
            location=node.location,
        )
        try:
            ret.value = type(node.value)
        except:
            pass
        return ret

    def integer_promotion_cast(self, node, type):
        """整数提升转换"""
        return self.implicit_cast(node, integer_promotion(type))

    def usual_arithmetic_cast(self, a: Expr, b: Expr):
        """一般算术转换"""
        if isinstance(a.type, DecimalType):
            if isinstance(b, (BinaryFloatType, ComplexType, ImaginaryType)):
                raise Error(f"不能为{b.type}", b.location)
            if isinstance(a.type, Decimal128Type):
                return a, self.implicit_cast(b, Decimal128Type())
            if isinstance(a.type, Decimal64Type):
                if not isinstance(b.type, Decimal128Type):
                    return a, self.implicit_cast(b, Decimal64Type())
                return self.implicit_cast(a, Decimal128Type()), b
            if isinstance(a.type, Decimal32Type):
                if not isinstance(b.type, (Decimal64Type, Decimal128Type)):
                    return a, self.implicit_cast(b, Decimal32Type())
                return self.implicit_cast(a, b.type), b
        if isinstance(b.type, DecimalType):
            return tuple(list(self.usual_arithmetic_cast(b, a))[::-1])

        if isinstance(
            a.type, (LongDoubleType, LongDoubleImaginaryType, LongDoubleComplexType)
        ):
            if isinstance(b.type, (IntegerType, RealFloatingType)):
                return a, self.implicit_cast(b, LongDoubleType())
            if isinstance(b.type, ComplexType):
                return a, self.implicit_cast(b, LongDoubleComplexType())
            if isinstance(b.type, ImaginaryType):
                return a, self.implicit_cast(b, LongDoubleImaginaryType())
        if isinstance(
            b.type, (LongDoubleType, LongDoubleComplexType, LongDoubleImaginaryType)
        ):
            return tuple(list(self.usual_arithmetic_cast(b, a))[::-1])

        if isinstance(a.type, (DoubleType, DoubleImaginaryType, DoubleComplexType)):
            if isinstance(b.type, (IntegerType, RealFloatingType)):
                return a, self.implicit_cast(b, DoubleType())
            if isinstance(b.type, ComplexType):
                return a, self.implicit_cast(b, DoubleComplexType())
            if isinstance(b.type, ImaginaryType):
                return a, self.implicit_cast(b, DoubleImaginaryType())
        if isinstance(b.type, (DoubleType, DoubleComplexType, DoubleImaginaryType)):
            return tuple(list(self.usual_arithmetic_cast(b, a))[::-1])

        if isinstance(a.type, (FloatType, FloatImaginaryType, FloatComplexType)):
            if isinstance(b.type, (IntegerType, RealFloatingType)):
                return a, self.implicit_cast(b, FloatType())
            if isinstance(b.type, ComplexType):
                return a, self.implicit_cast(b, FloatComplexType())
            if isinstance(b.type, ImaginaryType):
                return a, self.implicit_cast(b, FloatImaginaryType())
        if isinstance(b.type, (FloatType, FloatImaginaryType, FloatComplexType)):
            return tuple(list(self.usual_arithmetic_cast(b, a))[::-1])

        if isinstance(a.type, IntegerType) and isinstance(b.type, IntegerType):
            x = integer_promotion(a.type)
            y = integer_promotion(b.type)
            if x != y:
                xrange = x.range
                yrange = y.range
                if xrange[0] <= yrange[0] and yrange[1] <= xrange[1]:  # 选择范围更大的
                    y = x
                elif yrange[0] <= xrange[0] and xrange[1] <= yrange[1]:
                    x = y
                elif x.signed == True:
                    z = IntegerType()
                    z.signed = False
                    z.size = x.size
                    x = y = z
                elif y.signed == True:
                    z = IntegerType()
                    z.signed = False
                    z.size = y.size
                    x = y = z
            assert x == y
            return self.implicit_cast(a, x), self.implicit_cast(b, y)

    def result_type(self, a: Type, b: Type, op):
        """确定算术结果类型"""
        return self.operator[op](a(), b()).type

    @generic_implicit_cast
    def visit_Expr(self, node: Expr):
        self.generic_visit(node)
        return node

    @generic_implicit_cast
    def visit_Reference(self, node: Reference):
        if isinstance(node.symbol, Object):
            node.type = node.symbol.type
            if isinstance(node.type, QualifiedType) and node.type.has_const():
                node.value = node.symbol.init_value
        elif isinstance(node.symbol, Function):
            node.type = node.symbol.type
            node.value = node.symbol
        elif isinstance(node.symbol, EnumConst):
            node.type = node.symbol.enum_type.underlying_type
            if not hasattr(node.symbol.value_expr, "value"):
                raise Error("枚举值还没有确定", node.location)
            node.value = node.symbol.value
        else:
            raise Error(f"无法引用{node.name}", node.location)

        if isinstance(node.symbol, (Function, EnumConst)):
            node.is_lvalue = False
        else:
            node.is_lvalue = True

        return node

    @generic_implicit_cast
    def visit_BoolLiteral(self, node: BoolLiteral):
        node.type = BoolType()
        return node

    @generic_implicit_cast
    def visit_NullPtrLiteral(self, node: NullPtrLiteral):
        node.type = NullPtrType()
        return node

    @generic_implicit_cast
    def visit_IntegerLiteral(self, node: IntegerLiteral):
        if not node.value:
            node.value = "0"
        match node.prefix:
            case "":
                node.value = int(node.value)
            case "0b":
                node.value = int(node.value, base=2)
            case "0x":
                node.value = int(node.value, base=16)
            case "0":
                node.value = int(node.value, base=8)

        allow_types = []
        if "u" in node.suffix and "wb" in node.suffix:
            node.type = BitIntType(len(bin(node.value)) - 2, signed=False)
        elif "wb" in node.suffix:  # 这个要单独处理
            node.type = BitIntType(len(bin(node.value)) - 1)
        elif "u" in node.suffix and "ll" in node.suffix:
            allow_types = [ULongLongType]
        elif "ll" in node.suffix:
            allow_types = (
                [LongLongType] if node.prefix == "" else [LongLongType, ULongLongType]
            )
        elif "u" in node.suffix and "l" in node.suffix:
            allow_types = [ULongType, LongLongType]
        elif "l" in node.suffix:
            allow_types = (
                [LongType, ULongType, LongLongType]
                if node.prefix == ""
                else [LongType, ULongType, LongLongType, ULongLongType]
            )
        elif "u" in node.suffix:
            allow_types = [UIntType, ULongType, ULongLongType]
        else:
            allow_types = (
                [IntType, LongType, ULongType, LongLongType]
                if node.prefix == ""
                else [
                    IntType,
                    UIntType,
                    LongType,
                    ULongType,
                    LongLongType,
                    ULongLongType,
                ]
            )

        if allow_types:
            for int_type in allow_types:
                type = int_type()
                value = type(node.value)
                if value == node.value:
                    node.value = value
                    node.type = int_type()
                    break
            else:
                raise Error("整数太大了", node.location)

        return node

    @generic_implicit_cast
    def visit_FloatLiteral(self, node: FloatLiteral):
        if "df" in node.suffix:
            node.type = Decimal32Type()
        elif "dd" in node.suffix:
            node.type = Decimal64Type()
        elif "dl" in node.suffix:
            node.type = Decimal128Type()
        elif "l" in node.suffix:
            node.type = LongDoubleType()
        elif "f" in node.suffix:
            node.type = FloatType()
        else:
            node.type = DoubleType()

        if node.prefix == "0x":
            node.value = node.type(float.fromhex(node.prefix + node.value))
        else:
            node.value = node.type(float(node.prefix + node.value))
        return node

    @generic_implicit_cast
    def visit_StringLiteral(self, node: StringLiteral):
        match node.prefix:
            case "":
                node.value = list(node.value.encode("utf-8"))  # TODO: 可能不是utf-8
                node.type = ArrayType(
                    QualifiedType(
                        [TypeQualifier(qualifier=TypeQualifierKind.CONST)], CharType()
                    ),
                    IntegerLiteral(value=len(node.value)),
                )
            case "u8":
                node.value = encode_units(node.value, "utf-8")
                node.type = ArrayType(
                    QualifiedType(
                        [TypeQualifier(qualifier=TypeQualifierKind.CONST)], Char8Type()
                    ),
                    IntegerLiteral(value=len(node.value)),
                )
            case "u":
                node.value = encode_units(node.value, "utf-16")
                node.type = ArrayType(
                    QualifiedType(
                        [TypeQualifier(qualifier=TypeQualifierKind.CONST)], Char16Type()
                    ),
                    IntegerLiteral(value=len(node.value)),
                )
            case "U":
                node.value = encode_units(node.value, "utf-32")
                node.type = ArrayType(
                    QualifiedType(
                        [TypeQualifier(qualifier=TypeQualifierKind.CONST)], Char32Type()
                    ),
                    IntegerLiteral(value=len(node.value)),
                )
            case "L":
                node.value = encode_units(node.value, "utf-16")  # TODO: 可能不是utf-16
                node.type = ArrayType(
                    QualifiedType(
                        [TypeQualifier(qualifier=TypeQualifierKind.CONST)], WCharType()
                    ),
                    IntegerLiteral(value=len(node.value)),
                )
        node.is_lvalue = True
        return node

    @generic_implicit_cast
    def visit_CharLiteral(self, node: CharLiteral):
        match node.prefix:
            case "":
                node.value = encode_units(node.value, "utf-8")  # TODO: 可能不是utf-8
                node.type = QualifiedType(
                    [TypeQualifier(qualifier=TypeQualifierKind.CONST)], IntType()
                )
            case "u8":
                node.value = encode_units(node.value, "utf-8")
                node.type = QualifiedType(
                    [TypeQualifier(qualifier=TypeQualifierKind.CONST)], Char8Type()
                )
            case "u":
                node.value = encode_units(node.value, "utf-16")
                node.type = QualifiedType(
                    [TypeQualifier(qualifier=TypeQualifierKind.CONST)], Char16Type()
                )
            case "U":
                node.value = encode_units(node.value, "utf-32")
                node.type = QualifiedType(
                    [TypeQualifier(qualifier=TypeQualifierKind.CONST)], Char32Type()
                )
            case "L":
                node.value = encode_units(node.value, "utf-16")  # TODO: 可能不是utf-16
                node.type = QualifiedType(
                    [TypeQualifier(qualifier=TypeQualifierKind.CONST)], WCharType()
                )

        assert isinstance(node.value, list)

        if node.prefix not in "L" and len(node.value) > 1:  # 包括了空字符串
            raise Error("字符常量不能有多个字符", node.location)
        v = 0
        for i in node.value:
            v = v << 8 | i
        node.value = v

        return node

    @generic_implicit_cast
    def visit_ImaginaryUnit(self, node: ImaginaryUnit):
        node.type = FloatImaginaryType()
        node.value = Imaginary(1, node.type)
        return node

    @generic_implicit_cast
    def visit_CompoundLiteral(self, node: CompoundLiteral):
        self.generic_visit(node)
        node.type = node.type_name.type
        if not node.type.is_complete:
            raise Error(f"{node.type}不完整", node.location)
        node.is_lvalue = True
        node.initializer = self.implicit_cast(node.initializer, node.type)
        return node

    @generic_implicit_cast
    def visit_ArraySubscript(self, node: ArraySubscript):
        self.generic_visit(node)

        if isinstance(node.array.type, PointerType):  # 指针表达式 [ 整数表达式 ]
            if not node.index.type.is_integer_type:
                raise Error("数组索引应该是整数", node.index.location)
            node.type = node.array.type.pointee_type
        elif isinstance(node.index.type, PointerType):  # 整数表达式 [ 指针表达式 ]
            if not node.array.type.is_integer_type:
                raise Error("数组索引应该是整数", node.array.location)
            node.type = node.index.type.pointee_type
        else:
            raise Error(
                "下标访问需要数组或指针", node.array.location + node.index.location
            )
        node.is_lvalue = True
        return node

    @generic_implicit_cast
    def visit_UnaryOperator(self, node: UnaryOperator):
        self.generic_visit(node)

        if hasattr(node.operand, "value") and node.op in self.operator:
            try:
                node.value = self.operator[node.op](node.operand.value)
                node.type = node.value.type
            except Error:
                raise Error(
                    f"无法对{node.operand.type}进行{node.op.value}运算",
                    node.location,
                )

        match node.op:
            case UnaryOpKind.DEREFERENCE:
                if not isinstance(node.operand.type, PointerType):
                    raise Error(f"只能对指针类型解引用", node.location)
                node.type = node.operand.type.pointee_type
                if is_lvalue(node.operand):
                    node.is_lvalue = True
            case UnaryOpKind.ADDRESS:
                if not is_lvalue(node.operand) and not isinstance(
                    node.operand.type, FunctionType
                ):
                    raise Error("只能对左值取地址", node.location)
                node.type = PointerType(node.operand.type)
            case UnaryOpKind.NOT:
                if not node.operand.type.is_scalar_type:
                    raise Error("不能对非标量类型进行'!'操作", node.location)
                node.type = IntType()
            case UnaryOpKind.POSITIVE | UnaryOpKind.NEGATIVE:
                if not node.operand.type.is_arithmetic_type:
                    raise Error(
                        f"'{node.op.value}'的操作数应该是算术类型", node.location
                    )
                node.operand = self.integer_promotion_cast(
                    node.operand, node.operand.type
                )
                node.type = node.operand.type
            case UnaryOpKind.INVERT:
                if not node.operand.type.is_integer_type:
                    raise Error(f"'{node.op.value}'需要整数类型", node.location)
                node.operand = self.integer_promotion_cast(
                    node.operand, node.operand.type
                )
                node.type = node.operand.type
            case (
                UnaryOpKind.POSTFIX_DEC
                | UnaryOpKind.PREFIX_DEC
                | UnaryOpKind.POSTFIX_INC
                | UnaryOpKind.PREFIX_INC
            ):
                if not is_modifiable_lvalue(node.operand):
                    raise Error("无法修改", node.location)
                a = node.operand.type
                if isinstance(node.operand.type, (AtomicType, QualifiedType)):
                    a = node.operand.type.type
                if a.is_integer_type or isinstance(a, EnumType):
                    pass
                elif a.is_real_floating_type:
                    pass
                elif isinstance(a, PointerType):
                    pass
                else:
                    Error(
                        f"非法的'{node.op.value}'操作数类型:{node.operand.type}",
                        node.operand.location,
                    )
                node.type = node.operand.type
            case UnaryOpKind.SIZEOF:
                node.type = SizeType()
                node.value = node.operand.type.size
            case UnaryOpKind.ALIGNOF:
                if not node.operand.type.is_complete:
                    raise Error("必须是完整类型", node.operand.location)
                if isinstance(node.operand.type, FunctionType):
                    raise Error("不能是函数类型", node.operand.type)
                node.type = SizeType()
                if isinstance(node.operand.type, ArrayType):
                    node.value = node.operand.type.element_type.alignment
                else:
                    node.value = node.operand.type.alignment
        return node

    @generic_implicit_cast
    def visit_BinaryOperator(self, node: BinaryOperator):
        self.generic_visit(node)

        if (
            hasattr(node.left, "value")
            and hasattr(node.right, "value")
            and node.op in self.operator
        ):
            try:
                node.value = self.operator[node.op](node.left.value, node.right.value)
                node.type = node.value.type
            except Exception as e:
                raise Error(
                    f"无法在{node.left.type}和{node.right.type}之间进行{node.op.value}运算({e})",
                    node.location,
                )

        match node.op:
            case BinOpKind.ASSIGN:
                if not node.left.type.is_complete:
                    raise Error(f"{node.left.type}不完整", node.left.location)
                if not is_modifiable_lvalue(node.left):
                    raise Error("无法修改", node.location)
                node.right = self.implicit_cast(
                    node.right, remove_qualifier(node.left.type)
                )
                node.type = node.left.type
            case (
                BinOpKind.AMUL
                | BinOpKind.ADIV
                | BinOpKind.AMOD
                | BinOpKind.ALSHIFT
                | BinOpKind.ARSHIFT
                | BinOpKind.ABITAND
                | BinOpKind.ABITXOR
                | BinOpKind.ABITOR
            ):
                a = node.left.type
                b = node.right.type
                if isinstance(node.left.type, (AtomicType, QualifiedType)):
                    a = node.left.type.type
                if not (a.is_arithmetic_type and b.is_arithmetic_type):
                    raise Error(f"'{node.op.value}'需要算术类型", node.location)
                node.type = node.left.type
            case BinOpKind.AADD:
                a = node.left.type
                b = node.right.type
                if isinstance(node.left.type, (AtomicType, QualifiedType)):
                    a = node.left.type.type
                if a.is_arithmetic_type and b.is_arithmetic_type:
                    pass
                elif isinstance(a, PointerType) and b.is_integer_type:
                    if not a.is_complete:
                        raise Error(f"{a.pointee_type}不完整", node.left.location)
                elif isinstance(b, PointerType) and a.is_integer_type:
                    if not b.is_complete:
                        raise Error(f"{b.pointee_type}不完整", node.right.location)
                else:
                    raise Error(
                        f"非法的'{node.op.value}'操作数类型: {a}和{b}",
                        node.location,
                    )
                node.type = node.left.type
            case BinOpKind.ASUB:
                a = node.left.type
                b = node.right.type
                if isinstance(node.left.type, (AtomicType, QualifiedType)):
                    a = node.left.type.type
                if a.is_arithmetic_type and b.is_arithmetic_type:
                    pass
                elif isinstance(a, PointerType) and b.is_integer_type:
                    if not a.is_complete:
                        raise Error(f"{a.pointee_type}不完整", node.left.location)
                elif isinstance(a, PointerType) and isinstance(b, PointerType):
                    a = remove_qualifier(a.pointee_type)
                    b = remove_qualifier(b.pointee_type)
                    if not is_compatible_type(a, b):
                        raise Error(f"{a}和{b}不兼容", node.location)
                    if not a.is_complete:
                        raise Error(f"{a}不完整", node.left.location)
                    if not b.is_complete:
                        raise Error(f"{b}不完整", node.right.location)
                else:
                    raise Error(
                        f"非法的'{node.op.value}'操作数类型: {a}和{b}",
                        node.location,
                    )
                node.type = node.left.type
            case BinOpKind.AND | BinOpKind.OR:
                if (
                    not node.left.type.is_scalar_type
                    or not node.right.type.is_scalar_type
                ):
                    raise Error(
                        f"不能对非标量类型进行'{node.op.value}'操作", node.location
                    )
                node.type = IntType()
            case BinOpKind.LT | BinOpKind.LTE | BinOpKind.GT | BinOpKind.GTE:
                node.type = IntType()
                if node.left.type.is_real_type and node.right.type.is_real_type:
                    node.left, node.right = self.usual_arithmetic_cast(
                        node.left, node.right
                    )
                elif isinstance(node.left.type, PointerType) and isinstance(
                    node.right.type, PointerType
                ):
                    a = remove_qualifier(node.left.type.pointee_type)
                    b = remove_qualifier(node.right.type.pointee_type)
                    if not is_compatible_type(a, b):
                        raise Error(f"{a}和{b}不兼容", node.location)
                else:
                    raise Error(
                        f"{node.op.value}需要两个操作数都为实数类型或指针类型",
                        node.left.location + node.right.location,
                    )
            case BinOpKind.EQ | BinOpKind.NEQ:
                node.type = IntType()
                if (
                    node.left.type.is_arithmetic_type
                    and node.right.type.is_arithmetic_type
                ):
                    node.left, node.right = self.usual_arithmetic_cast(
                        node.left, node.right
                    )
                elif isinstance(node.left.type, NullPtrType) and isinstance(
                    node.right.type, NullPtrType
                ):
                    pass
                elif isinstance(node.left.type, NullPtrType) and isinstance(
                    node.right.type, PointerType
                ):
                    node.left = self.implicit_cast(node.left, node.right.type)
                elif isinstance(node.left.type, PointerType) and isinstance(
                    node.right.type, NullPtrType
                ):
                    node.right = self.implicit_cast(node.right, node.left.type)
                elif isinstance(node.left.type, PointerType) and isinstance(
                    node.right.type, PointerType
                ):
                    a = remove_qualifier(node.left.type.pointee_type)
                    b = remove_qualifier(node.right.type.pointee_type)
                    is_a_void = a == VoidType()
                    is_b_void = b == VoidType()
                    if not is_compatible_type(a, b) and not (is_a_void or is_b_void):
                        raise Error(f"{a}和{b}不兼容", node.location)
                    if is_a_void or is_b_void:
                        node.left = self.implicit_cast(
                            node.left, PointerType(VoidType())
                        )
                        node.right = self.implicit_cast(
                            node.right, PointerType(VoidType())
                        )
                else:
                    raise Error(
                        f"非法的'{node.op.value}'操作数类型: {node.left.type}和{node.right.type}",
                        node.location,
                    )
            case BinOpKind.ADD:
                if (
                    node.left.type.is_arithmetic_type
                    and node.right.type.is_arithmetic_type
                ):
                    node.left, node.right = self.usual_arithmetic_cast(
                        node.left, node.right
                    )
                    node.type = self.result_type(
                        node.left.type, node.right.type, node.op
                    )
                elif (
                    isinstance(node.left.type, PointerType)
                    and node.right.type.is_integer_type
                ):
                    if not node.left.type.is_complete:
                        raise Error(
                            f"{node.left.type.pointee_type}不完整", node.left.location
                        )
                    node.type = node.left.type
                elif (
                    isinstance(node.right.type, PointerType)
                    and node.left.type.is_integer_type
                ):
                    if not node.right.type.is_complete:
                        raise Error(
                            f"{node.right.type.pointee_type}不完整", node.right.location
                        )
                    node.type = node.left.type
                else:
                    raise Error(
                        f"非法的'{node.op.value}'操作数类型: {node.left.type}和{node.right.type}",
                        node.location,
                    )
            case BinOpKind.SUB:
                if (
                    node.left.type.is_arithmetic_type
                    and node.right.type.is_arithmetic_type
                ):
                    node.left, node.right = self.usual_arithmetic_cast(
                        node.left, node.right
                    )
                    node.type = self.result_type(
                        node.left.type, node.right.type, node.op
                    )
                elif (
                    isinstance(node.left.type, PointerType)
                    and node.right.type.is_integer_type
                ):
                    if not node.left.type.is_complete:
                        raise Error(
                            f"{node.left.type.pointee_type}不完整", node.left.location
                        )
                    node.type = node.left.type
                elif isinstance(node.left.type, PointerType) and isinstance(
                    node.right.type, PointerType
                ):
                    a = remove_qualifier(node.left.type.pointee_type)
                    b = remove_qualifier(node.right.type.pointee_type)
                    if not is_compatible_type(a, b):
                        raise Error(f"{a}和{b}不兼容", node.location)
                    if not a.is_complete:
                        raise Error(f"{a}不完整", node.left.location)
                    if not b.is_complete:
                        raise Error(f"{b}不完整", node.right.location)
                    node.type = node.left.type
                else:
                    raise Error(
                        f"非法的'{node.op.value}'操作数类型: {node.left.type}和{node.right.type}",
                        node.location,
                    )
            case BinOpKind.MUL | BinOpKind.DIV | BinOpKind.MOD:
                if not (
                    node.left.type.is_arithmetic_type
                    and node.right.type.is_arithmetic_type
                ):
                    raise Error(f"'{node.op.value}'运算需要两个算术类型", node.location)
                node.left, node.right = self.usual_arithmetic_cast(
                    node.left, node.right
                )
                node.type = self.result_type(node.left.type, node.right.type, node.op)
            case BinOpKind.BITAND | BinOpKind.BITOR | BinOpKind.BITXOR:
                if not (
                    node.left.type.is_integer_type and node.right.type.is_integer_type
                ):
                    raise Error(f"'{node.op.value}'需要整数类型", node.location)
                node.left, node.right = self.usual_arithmetic_cast(
                    node.left, node.right
                )
                node.type = node.left.type
            case BinOpKind.LSHIFT | BinOpKind.RSHIFT:
                if not (
                    node.left.type.is_integer_type and node.right.type.is_integer_type
                ):
                    raise Error(f"'{node.op.value}'需要整数类型", node.location)
                node.left = self.integer_promotion_cast(node.left, node.left.type)
                node.right = self.integer_promotion_cast(node.right, node.right.type)
                node.type = node.left.type
            case BinOpKind.COMMA:
                node.type = node.right.type
        return node

    @generic_implicit_cast
    def visit_FunctionCall(self, node: FunctionCall):
        self.generic_visit(node)

        if not isinstance(node.func.type, PointerType) or not isinstance(
            node.func.type.pointee_type, FunctionType
        ):
            raise Error("应该调用一个函数", node.location)
        function_type: FunctionType = node.func.type.pointee_type

        for i, v in enumerate(function_type.parameters_type):
            if isinstance(v, ArrayPtrType) and isinstance(node.args[i].type, ArrayType):
                if not node.args[i].type.is_complete:
                    raise Error("传入数组不完整", node.args[i].type)
                if (
                    v.array_type.len_expr != None
                    and node.args[i].type.len < v.array_type.len
                ):
                    raise Error(
                        f"数组大小不足{v.array_type.len}", node.args[i].location
                    )
            node.args[i] = self.implicit_cast(node.args[i], v)

        if function_type.has_varparam:
            while i < len(node.args):
                if isinstance(node.args[i].type, FloatType):
                    node.args[i] = self.implicit_cast(node.args[i], DoubleType)
                elif node.args[i].type.is_integer_type:
                    node.args[i] = self.integer_promotion_cast(
                        node.args[i], node.args[i].type
                    )
                i += 1
        elif i != len(node.args):
            raise Error(f"形参有{i}个参数, 但给了{len(node.args)}个参数", node.location)

        node.type = function_type.return_type

        return node

    @generic_implicit_cast
    def visit_FunctionDef(self, node: FunctionDef):
        return super().visit_FunctionDef(node)

    @generic_implicit_cast
    def visit_ReturnStmt(self, node: ReturnStmt):
        if node.expr == None:
            return node
        self.generic_visit(node)
        for i in self.path[::-1]:
            if isinstance(i, FunctionDef):
                node.expr = self.implicit_cast(node.expr, i.func_type.return_type)
                break
        return node

    @generic_implicit_cast
    def visit_MemberRef(self, node: MemberRef):
        self.generic_visit(node)

        if node.is_arrow:
            node.is_lvalue = True
            if not isinstance(node.target.type, PointerType):
                raise Error(f"'->'只能用在指针上", node.location)
            record_type = node.target.type.pointee_type
        else:
            node.is_lvalue = is_lvalue(node.target)
            record_type = node.target.type

        if not isinstance(record_type, RecordType):
            raise Error(f"无法访问{record_type}的成员", node.location)
        for member in record_type.members:
            if member == node.member_name:
                break
        else:
            raise Error(f"{record_type}没有成员{node.member_name}", node.location)
        node.type = record_type.members[member].type

        return node

    @generic_implicit_cast
    def visit_ConditionalOperator(self, node: ConditionalOperator):
        self.generic_visit(node)

        if (
            hasattr(node.condition_expr, "value")
            and hasattr(node.true_expr, "value")
            and hasattr(node.false_expr, "value")
        ):
            if bool(node.condition_expr.value):
                node.value = node.true_expr.value
            else:
                node.value = node.false_expr.value
            node.type = node.value.type

        if not node.condition_expr.type.is_scalar_type:
            raise Error("条件得是标量类型", node.condition_expr.location)
        if (
            node.true_expr.type.is_arithmetic_type
            and node.false_expr.type.is_arithmetic_type
        ):
            node.true_expr, node.false_expr = self.usual_arithmetic_cast(
                node.true_expr, node.false_expr
            )
            node.type = node.true_expr.type
        elif (
            isinstance(node.true_expr.type, RecordType)
            and isinstance(node.false_expr.type, RecordType)
            and node.true_expr.type == node.false_expr.type
        ):
            node.type = node.true_expr.type
        elif node.true_expr.type == VoidType() == node.false_expr.type:
            node.type = VoidType()
        elif isinstance(node.true_expr.type, (PointerType, NullPtrType)) and isinstance(
            node.false_expr.type, (PointerType, NullPtrType)
        ):
            if isinstance(node.true_expr.type, NullPtrType) and isinstance(
                node.false_expr.type, NullPtrType
            ):
                node.type = NullPtrType()
            elif isinstance(node.true_expr.type, NullPtrType):
                node.type = node.false_expr.type
            elif isinstance(node.false_expr.type, NullPtrType):
                node.type = node.true_expr.type
            else:
                a = node.true_expr.type.pointee_type
                qa = []
                if isinstance(a, QualifiedType):
                    qa = a.qualifiers
                    a = a.type
                b = node.false_expr.type.pointee_type
                qb = []
                if isinstance(b, QualifiedType):
                    qb = b.qualifiers
                    b = b.type

                if a == VoidType() == b:
                    if qa or qb:
                        pointee_type = QualifiedType(qa + qb, a)
                    node.type = PointerType(pointee_type)
                elif a == VoidType():
                    node.type = node.false_expr.type
                elif b == VoidType():
                    node.type = node.true_expr.type
                elif not is_compatible_type(a, b):
                    raise Error(f"{a}和{b}不兼容", node.location)
                else:
                    if a != b:
                        try:
                            pointee_type = composite_type(
                                a.pointee_type, b.pointee_type
                            )
                        except Exception as e:
                            raise Error(e.args[0], node.location)
                    elif qa or qb:
                        pointee_type = QualifiedType(qa + qb, a)
                    else:
                        pointee_type = a
                    node.type = PointerType(pointee_type)
        else:
            raise Error("非法三目运算符的操作数", node.location)
        return node

    def visit_GenericSelection(self, node: GenericSelection):
        self.generic_visit(node)
        node.select = None
        defulat_assoc = None

        for assoc in node.assoc_list:
            if assoc.is_default:
                if defulat_assoc != None:
                    raise Error("只能有一个default", assoc.location)
                defulat_assoc = assoc
                continue
            if is_compatible_type(node.controling_expr.type, assoc.expr.type):
                node.select = assoc.expr
                break
        else:
            if defulat_assoc != None:
                node.select = defulat_assoc.expr

        if node.select == None:
            raise Error("无法选择", node.location)
        if hasattr(node.select, "value"):
            node.value = node.select.value
        node.type = node.select.type
        node.is_lvalue = node.select.is_lvalue

    @generic_implicit_cast
    def visit_ExplicitCast(self, node: ExplicitCast):
        self.generic_visit(node)

        if hasattr(node.expr, "value"):
            node.value = node.expr.value

        node.type = node.type_name.type

        return node

    @generic_implicit_cast
    def visit_EnumDecl(self, node: EnumDecl):
        candidate = [
            IntType(),
            UIntType(),
            LongType(),
            ULongType(),
            LongLongType(),
            ULongLongType(),
        ]
        candidate_i = 0
        underlying_type = candidate[candidate_i]

        for i, enumerator in enumerate(node.enumerators):
            enum_const = node.type.enumerators[enumerator.name]
            if enumerator.value != None:
                enumerator.accept(self)
                while underlying_type(enum_const.value) != int(
                    enum_const.value
                ):  # 溢出
                    candidate_i += 1
                    if candidate_i >= len(candidate):
                        raise Error(
                            "无法找到合适的类型表示该枚举值", enumerator.location
                        )
                    underlying_type = candidate[candidate_i]
            elif i == 0:  # 第一个枚举值
                enum_const.value_expr = enumerator.value = IntegerLiteral(
                    value="0", location=enumerator.location
                ).accept(self)
            else:
                last_enum_const = node.type.enumerators[node.enumerators[i - 1].name]
                precise_value = int(last_enum_const.value) + 1  # 精确值
                while underlying_type(precise_value) != precise_value:  # 溢出
                    candidate_i += 1
                    if candidate_i >= len(candidate):
                        raise Error(
                            "无法找到合适的类型表示该枚举值", enumerator.location
                        )
                    underlying_type = candidate[candidate_i]
                enum_const.value_expr = enumerator.value = IntegerLiteral(
                    value=underlying_type(precise_value),
                    type=underlying_type,
                    location=enumerator.location,
                )

        for i in range(len(node.enumerators)):
            enumerator = node.enumerators[i]
            enumerator.value = self.implicit_cast(enumerator.value, underlying_type)
            node.type.enumerators[enumerator.name].value_expr = enumerator.value

        node.underlying_type = node.type.underlying_type = underlying_type

        return node

    def visit_InitList(self, node: InitList):
        class NodeDict(TypedDict):
            type: Type
            value: str | int
            parent_type: Type
            union_id: list[int]  # 所属的union的id
            union_index: list[int]  # 属于union的哪一个成员之中, 与union_id一一对应

        def gen_path(
            t: Type,
        ) -> Generator[Union[list[NodeDict], dict[int, list[NodeDict]]]]:
            if isinstance(t, ArrayType):
                i = 0
                if t.len_expr == None:
                    n = float("inf")
                else:
                    n = t.len_expr.value
                while i < n:
                    for k in gen_path(t.element_type):
                        yield [
                            {
                                "type": t.element_type,
                                "value": i,
                                "parent_type": t,
                                "union_id": [],
                                "union_index": [],
                            }
                        ] + k
                    i += 1
            elif isinstance(t, RecordType):
                for i, (name, member) in enumerate(t.members.items()):
                    if (
                        isinstance(member.type, ArrayType)
                        and member.type.len_expr == None
                    ):
                        if i != len(t.members):
                            raise Error(
                                "柔性数组没有位于结构体最后", member.define_location
                            )
                        raise Error(
                            "柔性数组成员不能用初始化列表初始化",
                            member.define_location,
                        )
                    for k in gen_path(member.type):
                        nodes: list[NodeDict] = [
                            {
                                "type": member.type,
                                "value": name,
                                "parent_type": t,
                                "union_id": [],
                                "union_index": [],
                            }
                        ] + k
                        if t.struct_or_union == "union":
                            for node in nodes:
                                node["union_id"].append(id(t))
                                node["union_index"].append(i)
                        yield nodes
            else:
                yield []

        def build_designator(
            initializer: Expr, designators: list[NodeDict]
        ) -> list[Designator]:
            """
            将gen_path生成的路径转换成Designator列表
            initializer是用来提供位置信息的
            """
            a = []
            for i in designators:
                if isinstance(i["parent_type"], ArrayType):
                    a.append(
                        Designator(
                            index=IntegerLiteral(
                                value=i["value"], location=initializer.location
                            ).accept(self),
                            location=initializer.location,
                        )
                    )
                elif isinstance(i["parent_type"], RecordType):
                    a.append(
                        Designator(
                            member=i["value"],
                            location=initializer.location,
                        )
                    )
                # 其它类型在一开始就会被处理
            return a

        if not hasattr(node, "type"):  # InitList的类型要由外部指定
            return node
        self.generic_visit(node)

        if node.type.is_scalar_type:  # 标量初始化
            if len(node.initializers) > 1:
                raise Error("标量初始化只需要0个或1个表达式", node.location)
            if len(node.initializers) == 1:
                node.initializers[0] = self.implicit_cast(
                    node.initializers[0], node.type
                )
            return node

        if not isinstance(node.type, ArrayType) and not isinstance(
            node.type, RecordType
        ):
            raise Error(f"无法初始化{node.type}", node.location)

        path_gen = gen_path(node.type)
        paths: list[NodeDict] = []
        cur = 0
        compare_index = -1  # 获取下一个path的时候跟path的哪一个元素进行比较
        path: NodeDict = None
        select_union_index = {}  # 各个union被选中的成员索引

        def next_path(error_msg):
            nonlocal cur
            while True:
                if cur >= len(paths):
                    try:
                        path = next(path_gen)
                    except StopIteration:
                        raise Error(error_msg, init_expr.location)
                    paths.append(path)
                for node in paths[cur]:
                    # 忽略那些不用于初始化的union成员
                    for i, union_id in enumerate(node["union_id"]):
                        if (
                            union_id in select_union_index
                            and node["union_index"][i] != select_union_index[union_id]
                        ):  # 忽略
                            break
                    else:
                        continue
                    break
                else:
                    path = paths[cur]
                    cur += 1
                    return path
                cur += 1

        for i, init_expr in enumerate(node.initializers):
            if not isinstance(init_expr, Designation):
                # 如果有指派符就无法保证这次获取的是下一个path
                # 所以下面的判断需要放在这个分支下
                if path == None:
                    path = next_path("超出可初始化的数量")
                else:
                    t = path[compare_index]
                    while t["value"] == path[compare_index]["value"]:
                        path = next_path("超出可初始化的数量")
                    cur -= 1
                if isinstance(init_expr, InitList):
                    node.initializers[i] = self.implicit_cast(
                        Designation(
                            designators=build_designator(init_expr, path[:1]),
                            initializer=init_expr,
                            location=init_expr.location,
                        ),
                        path[0]["type"],
                    )
                    compare_index = 0
                else:
                    node.initializers[i] = self.implicit_cast(
                        Designation(
                            designators=build_designator(init_expr, path),
                            initializer=init_expr,
                            location=init_expr.location,
                        ),
                        path[-1]["type"],
                    )
                    compare_index = -1
            else:
                # 查找符合要求的
                cur = 0
                k = len(init_expr.designators)
                while True:
                    path = next_path("索引或成员不存在")
                    for a, b in zip(
                        build_designator(init_expr, path[:k]),
                        init_expr.designators,
                    ):
                        default_member = None
                        default_index = Expr(value=None)
                        if not (
                            getattr(a, "member", default_member)
                            == getattr(b, "member", default_member)
                            and getattr(a, "index", default_index).value
                            == getattr(b, "index", default_index).value
                        ):
                            break
                    else:  # 找到了
                        break
                if isinstance(init_expr.initializer, InitList):
                    node.initializers[i] = self.implicit_cast(
                        node.initializers[i], path[k - 1]["type"]
                    )
                    compare_index = k - 1
                else:
                    node.initializers[i].designators = build_designator(init_expr, path)
                    node.initializers[i] = self.implicit_cast(
                        node.initializers[i], path[-1]["type"]
                    )
                    compare_index = -1
            # 跳过union剩余的部分
            # 保证这时候的path是init_expr.designators或它的一部分
            for node_ in path:
                for i, union_id in enumerate(node_["union_id"]):
                    select_union_index[union_id] = node_["union_index"][i]

        if isinstance(node.type, ArrayType) and not node.type.is_complete:
            max_index = max(
                [j["value"] for i in paths for j in i if j["parent_type"] == node.type]
            )
            node.type.len_expr = IntegerLiteral(
                value=max_index + 1, location=node.location
            ).accept(self)

        return node

    @generic_implicit_cast
    def visit_TypeOrVarDecl(self, node: TypeOrVarDecl):
        self.generic_visit(node)
        if node.initializer != None:
            node.initializer = self.implicit_cast(
                node.initializer, remove_qualifier(node.type)
            )
        if isinstance(node.type, ArrayType) and not node.type.is_complete:
            raise Error("无法确定数组长度", node.location)
        return node
