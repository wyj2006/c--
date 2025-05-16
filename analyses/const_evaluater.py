from basic import (
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
)
from basic import (
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
    VoidType,
)
from basic import Object, Function, EnumConst, Error
from analyses.analyzer import Analyzer


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


class ConstEvaluater(Transformer, Analyzer):
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

    def visit_Reference(self, node: Reference):
        if isinstance(node.symbol, Object):
            node.type = node.symbol.type
            if isinstance(node.type, QualifiedType) and node.type.has_const():
                node.value = node.symbol.init_value
        elif isinstance(node.symbol, Function):
            node.type = node.symbol.type
        elif isinstance(node.symbol, EnumConst):
            node.type = node.symbol.enum_type.underlying_type
            node.value = node.symbol.value
        else:
            raise Error(f"无法引用{node.name}", node.location)

        if isinstance(node.symbol, (Function, EnumConst)):
            node.is_lvalue = False
        else:
            node.is_lvalue = True

        return node

    def visit_BoolLiteral(self, node: BoolLiteral):
        node.type = BoolType()
        return node

    def visit_NullPtrLiteral(self, node: NullPtrLiteral):
        node.type = NullPtrType()
        return node

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

        # TODO: 不依赖float
        if node.prefix == "0x":
            node.value = float.fromhex(node.prefix + node.value)
        else:
            node.value = float(node.prefix + node.value)
        return node

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
        node.is_value = True
        return node

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

    def visit_BinaryOperator(self, node: BinaryOperator):
        self.generic_visit(node)

        if not (hasattr(node.left, "value") and hasattr(node.right, "value")):
            return node

        try:
            node.value = self.operator[node.op](node.left.value, node.right.value)
            node.type = node.value.type
        except:
            raise Error(
                f"无法在{node.left.type}和{node.right.type}之间进行{node.op.value}运算",
                node.location,
            )

        return node

    def visit_UnaryOperator(self, node: UnaryOperator):
        self.generic_visit(node)

        if not hasattr(node.operand, "value"):
            return node

        if node.op == UnaryOpKind.SIZEOF:
            return node  # TODO

        try:
            node.value = self.operator[node.op](node.operand.value)
        except Error:
            raise Error(
                f"无法对{node.operand.type}进行{node.op.value}运算",
                node.location,
            )
        if node.op == UnaryOpKind.NOT:
            node.type = IntType()
        else:
            node.type = node.value.type

        return node

    def visit_ConditionalOperator(self, node: ConditionalOperator):
        self.generic_visit(node)

        if not (
            hasattr(node.condition_expr, "value")
            and hasattr(node.true_expr, "value")
            and hasattr(node.false_expr, "value")
        ):
            return node

        if bool(node.condition_expr.value):
            node.type = node.true_expr.type
            node.value = node.true_expr.value
        else:
            node.type = node.false_expr.type
            node.value = node.false_expr.value

        return node

    def visit_ExplicitCast(self, node: ExplicitCast):
        self.generic_visit(node)

        if hasattr(node.expr, "value"):
            node.value = node.expr.value

        node.type = node.type_name.type
        if not (node.type.is_scalar_type or node.type == VoidType()):
            raise Error(f"不能转换成{node.type}", node.location)
        if not (node.expr.type.is_scalar_type or node.expr.type == VoidType()):
            raise Error(f"{node.expr.type}不能进行转换", node.location)

        return node

    # TODO: Enum
    # TODO: Array Def
    # TODO: InitList
