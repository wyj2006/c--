from cast import (
    Transformer,
    Reference,
    BoolLiteral,
    NullPtrLiteral,
    IntegerLiteral,
    FloatLiteral,
    StringLiteral,
    CharLiteral,
    BinaryOperator,
    BinOpKind,
    UnaryOperator,
    UnaryOpKind,
    ConditionalOperator,
    ExplicitCast,
    EnumDecl,
    InitList,
    Designation,
    TypeOrVarDecl,
    Expr,
    ImaginaryLiteral,
)
from typesystem import (
    BoolType,
    NullPtrType,
    BitIntType,
    LongDoubleType,
    LongLongType,
    LongType,
    UIntType,
    ULongLongType,
    ULongType,
    IntType,
    Decimal128Type,
    Decimal32Type,
    Decimal64Type,
    DoubleType,
    FloatType,
    CharType,
    Char8Type,
    Char16Type,
    Char32Type,
    WCharType,
    ArrayType,
    QualifiedType,
    SizeType,
    FloatImaginaryType,
)
from basic import Object, Function, EnumConst, Error, Warn
from values import Int, Imaginary
from analyses.analyzer import Analyzer
from .cast_operation import generic_implicit_cast, implicit_cast, remove_qualifier


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


def get_designation_index(expr: Expr):
    """
    获取指派符的索引
    没有就返回None
    """
    if not isinstance(expr, Designation):
        return None
    assert len(expr.designators) == 1  # 已经拆分过了
    designator = expr.designators[0]
    if hasattr(designator, "index"):
        return designator.index.value
    return None


class ConstEvaluater(Transformer, Analyzer):
    """
    完成的任务(分摊TypeChecker的部分任务):
    1. 计算常量表达式的值
    2. 确定枚举值及枚举类型的底层类型
    3. 确定不完整数组类型的大小
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
        UnaryOpKind.NOT: lambda a: not a,
    }

    def __init__(self, symtab):
        super().__init__(symtab)
        self.path = []  # 调用路径

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
                [ULongLongType] if node.prefix == "" else [LongLongType, ULongLongType]
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
                    CharType(),
                    IntegerLiteral(value=len(node.value)),
                )
            case "u8":
                node.value = encode_units(node.value, "utf-8")
                node.type = ArrayType(
                    Char8Type(),
                    IntegerLiteral(value=len(node.value)),
                )
            case "u":
                node.value = encode_units(node.value, "utf-16")
                node.type = ArrayType(
                    Char16Type(),
                    IntegerLiteral(value=len(node.value)),
                )
            case "U":
                node.value = encode_units(node.value, "utf-32")
                node.type = ArrayType(
                    Char32Type(),
                    IntegerLiteral(value=len(node.value)),
                )
            case "L":
                node.value = encode_units(node.value, "utf-16")  # TODO: 可能不是utf-16
                node.type = ArrayType(
                    WCharType(),
                    IntegerLiteral(value=len(node.value)),
                )
        node.is_lvalue = True
        return node

    @generic_implicit_cast
    def visit_CharLiteral(self, node: CharLiteral):
        match node.prefix:
            case "":
                node.value = encode_units(node.value, "utf-8")  # TODO: 可能不是utf-8
                node.type = IntType()
            case "u8":
                node.value = encode_units(node.value, "utf-8")
                node.type = Char8Type()
            case "u":
                node.value = encode_units(node.value, "utf-16")
                node.type = Char16Type()
            case "U":
                node.value = encode_units(node.value, "utf-32")
                node.type = Char32Type()
            case "L":
                node.value = encode_units(node.value, "utf-16")  # TODO: 可能不是utf-16
                node.type = WCharType()

        assert isinstance(node.value, list)

        if node.prefix not in "L" and len(node.value) > 1:  # 包括了空字符串
            raise Error("字符常量不能有多个字符", node.location)
        v = 0
        for i in node.value:
            v = v << 8 | i
        node.value = v

        return node

    @generic_implicit_cast
    def visit_ImaginaryLiteral(self, node: ImaginaryLiteral):
        node.type = FloatImaginaryType()
        node.value = Imaginary(1, node.type)
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
            except Exception as e:
                raise Error(
                    f"无法在{node.left.type}和{node.right.type}之间进行{node.op.value}运算({e})",
                    node.location,
                )
            if node.op in (
                BinOpKind.EQ,
                BinOpKind.NEQ,
                BinOpKind.GT,
                BinOpKind.GTE,
                BinOpKind.LT,
                BinOpKind.LTE,
                BinOpKind.AND,
                BinOpKind.OR,
            ):
                node.type = IntType()
                node.value = Int(bool(node.value))
            else:
                node.type = node.value.type

        return node

    @generic_implicit_cast
    def visit_UnaryOperator(self, node: UnaryOperator):
        self.generic_visit(node)

        if hasattr(node.operand, "value") and node.op in self.operator:
            try:
                node.value = self.operator[node.op](node.operand.value)
            except Error:
                raise Error(
                    f"无法对{node.operand.type}进行{node.op.value}运算",
                    node.location,
                )
            if node.op in (UnaryOpKind.NOT,):
                node.type = IntType()
                node.value = Int(bool(node.value))
            else:
                node.type = node.value.type

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

        return node

    @generic_implicit_cast
    def visit_ExplicitCast(self, node: ExplicitCast):
        self.generic_visit(node)

        if hasattr(node.expr, "value"):
            node.value = node.expr.value

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
            enumerator.value = implicit_cast(enumerator.value, underlying_type)
            node.type.enumerators[enumerator.name].value_expr = enumerator.value

        node.underlying_type = node.type.underlying_type = underlying_type

        return node

    def visit_InitList(self, node: InitList):
        if not hasattr(node, "type"):  # InitList的类型要由外部指定
            return node
        self.generic_visit(node)

        if node.type.is_scalar_type:  # 标量初始化
            if len(node.initializers) > 1:
                raise Error("标量初始化只需要0个或1个表达式", node.location)
            if len(node.initializers) == 1:
                node.initializers[0] = implicit_cast(node.initializers[0], node.type)
            return node

        if isinstance(node.type, ArrayType):  # 数组初始化
            if not node.type.element_type.is_complete:
                raise Error(f"{node.type.element_type}不完整", node.location)
            index = -1  # 上一个元素的索引
            max_index = 0
            for i, initializer in enumerate(node.initializers):
                if (designate_index := get_designation_index(initializer)) == None:
                    index += 1
                else:
                    index = int(designate_index)
                max_index = max(max_index, index)
                node.initializers[i] = implicit_cast(  # initializer有可能还是InitList
                    initializer, node.type.element_type, self
                )
            if not node.type.is_complete:
                t = SizeType()
                node.type.size_expr = IntegerLiteral(
                    value=t(max_index + 1), type=t, location=node.location
                )
            return node

        return node

    @generic_implicit_cast
    def visit_Designation(self, node: Designation):
        self.generic_visit(node)

        if len(node.designators) <= 1:  # 不处理
            return node
        # 拆分指派符
        t = node.designators.pop()
        a = InitList(
            initializers=[
                Designation(
                    designators=[t], initializer=node.initializer, location=t.location
                )
            ],
            location=t.location,
        )
        while node.designators:
            t = node.designators.pop()
            a = InitList(
                initializers=[
                    Designation(designators=[t], initializer=a, location=t.location)
                ],
                location=t.location,
            )
        return a.initializers[0]

    @generic_implicit_cast
    def visit_TypeOrVarDecl(self, node: TypeOrVarDecl):
        self.generic_visit(node)
        if node.initializer != None:
            node.initializer = implicit_cast(
                node.initializer, remove_qualifier(node.type), self
            )
        return node
