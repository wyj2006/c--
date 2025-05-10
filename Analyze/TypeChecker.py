import ctypes

from AST import (
    Transformer,
    Reference,
    BoolLiteral,
    NullPtrLiteral,
    StringLiteral,
    IntegerLiteral,
    FloatLiteral,
    CharLiteral,
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
)
from Basic import (
    Object,
    Function,
    EnumConst,
    Error,
    BasicType,
    BasicTypeKind,
    NullPtrType,
    ArrayType,
    BitIntType,
    Char8Type,
    Char32Type,
    Char16Type,
    WCharType,
    PointerType,
    is_integer_type,
    FunctionType,
    remove_qualifier,
    integer_promotion,
    is_compatible_type,
    RecordType,
    is_scalar_type,
    is_real_type,
    is_arithmetic_type,
    QualifiedType,
    AtomicType,
    EnumType,
    is_real_floating_type,
    ArrayPtrType,
    composite_type,
)
from Analyze.Analyzer import Analyzer
from Analyze.CastOperation import (
    implicit_cast,
    generic_implicit_cast,
    is_lvalue,
    usual_arithmetic_cast,
    integer_promotion_cast,
    is_modifiable_lvalue,
)


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


# TODO: Type中引用的Expr的同步问题
class TypeChecker(Analyzer, Transformer):
    def __init__(self, symtab):
        super().__init__(symtab)
        self.path = []  # 调用路径

    @generic_implicit_cast
    def visit_Reference(self, node: Reference):
        if isinstance(node.symbol, (Object, Function)):
            node.type = node.symbol.type
        elif isinstance(node.symbol, EnumConst):
            node.type = node.symbol.enum_type.underlying_type
        else:
            raise Error(f"无法引用{node.name}", node.location)

        if isinstance(node.symbol, (Function, EnumConst)):
            node.is_lvalue = False
        else:
            node.is_lvalue = True

        return node

    @generic_implicit_cast
    def visit_BoolLiteral(self, node: BoolLiteral):
        node.type = BasicType(BasicTypeKind.BOOL)
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
        # TODO: 不依赖于ctypes
        allow_types = []
        if "u" in node.suffix and "wb" in node.suffix:
            node.type = BitIntType(
                IntegerLiteral(value=len(bin(node.value)) - 2), signed=False
            )
        elif "wb" in node.suffix:  # 这个要单独处理
            node.type = BitIntType(IntegerLiteral(value=len(bin(node.value)) - 1))
        elif "u" in node.suffix and "ll" in node.suffix:
            allow_types = [(ctypes.c_ulonglong, BasicTypeKind.ULONGLONG)]
        elif "ll" in node.suffix:
            allow_types = (
                [(ctypes.c_ulonglong, BasicTypeKind.ULONGLONG)]
                if node.prefix == ""
                else [
                    (ctypes.c_longlong, BasicTypeKind.LONGLONG),
                    (ctypes.c_ulonglong, BasicTypeKind.ULONGLONG),
                ]
            )
        elif "u" in node.suffix and "l" in node.suffix:
            allow_types = [
                (ctypes.c_ulong, BasicTypeKind.ULONG),
                (ctypes.c_longlong, BasicTypeKind.LONGLONG),
            ]
        elif "l" in node.suffix:
            allow_types = (
                [
                    (ctypes.c_long, BasicTypeKind.LONG),
                    (ctypes.c_ulong, BasicTypeKind.ULONG),
                    (ctypes.c_longlong, BasicTypeKind.LONGLONG),
                ]
                if node.prefix == ""
                else [
                    (ctypes.c_long, BasicTypeKind.LONG),
                    (ctypes.c_ulong, BasicTypeKind.ULONG),
                    (ctypes.c_longlong, BasicTypeKind.LONGLONG),
                    (ctypes.c_ulonglong, BasicTypeKind.ULONGLONG),
                ]
            )
        elif "u" in node.suffix:
            allow_types = [
                (ctypes.c_uint, BasicTypeKind.UINT),
                (ctypes.c_ulong, BasicTypeKind.ULONG),
                (ctypes.c_ulonglong, BasicTypeKind.ULONGLONG),
            ]
        else:
            allow_types = (
                [
                    (ctypes.c_int, BasicTypeKind.INT),
                    (ctypes.c_long, BasicTypeKind.LONG),
                    (ctypes.c_ulong, BasicTypeKind.ULONG),
                    (ctypes.c_longlong, BasicTypeKind.LONGLONG),
                ]
                if node.prefix == ""
                else [
                    (ctypes.c_int, BasicTypeKind.INT),
                    (ctypes.c_uint, BasicTypeKind.UINT),
                    (ctypes.c_long, BasicTypeKind.LONG),
                    (ctypes.c_ulong, BasicTypeKind.ULONG),
                    (ctypes.c_longlong, BasicTypeKind.LONGLONG),
                    (ctypes.c_ulonglong, BasicTypeKind.ULONGLONG),
                ]
            )

        if allow_types:
            for int_type, type_kind in allow_types:
                value = int_type(node.value).value
                if value == node.value:
                    node.value = value
                    node.type = BasicType(type_kind)
                    break
            else:
                raise Error("整数太大了", node.location)

        return node

    @generic_implicit_cast
    def visit_FloatLiteral(self, node: FloatLiteral):
        if "df" in node.suffix:
            node.type = BasicType(BasicTypeKind.DECIMAL32)
        elif "dd" in node.suffix:
            node.type = BasicType(BasicTypeKind.DECIMAL64)
        elif "dl" in node.suffix:
            node.type = BasicType(BasicTypeKind.DECIMAL128)
        elif "l" in node.suffix:
            node.type = BasicType(BasicTypeKind.LONGDOUBLE)
        elif "f" in node.suffix:
            node.type = BasicType(BasicTypeKind.FLOAT)
        else:
            node.type = BasicType(BasicTypeKind.DOUBLE)

        # TODO: 不依赖float
        if node.prefix == "0x":
            node.value = float.fromhex(node.prefix + node.value)
        else:
            node.value = float(node.prefix + node.value)
        return node

    @generic_implicit_cast
    def visit_StringLiteral(self, node: StringLiteral):
        match node.prefix:
            case "":
                node.value = list(node.value.encode("utf-8"))  # TODO: 可能不是utf-8
                node.type = ArrayType(
                    BasicType(BasicTypeKind.CHAR),
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

    @generic_implicit_cast
    def visit_CharLiteral(self, node: CharLiteral):
        match node.prefix:
            case "":
                node.value = encode_units(node.value, "utf-8")  # TODO: 可能不是utf-8
                node.type = BasicType(BasicTypeKind.INT)
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
    def visit_CompoundLiteral(self, node: CompoundLiteral):
        self.generic_visit(node)
        node.type = node.type_name.type
        node.is_lvalue = True
        return node

    @generic_implicit_cast
    def visit_ArraySubscript(self, node: ArraySubscript):
        self.generic_visit(node)

        if isinstance(node.array.type, PointerType):  # 指针表达式 [ 整数表达式 ]
            if not is_integer_type(node.index.type):
                raise Error("数组索引应该是整数", node.index.location)
            node.type = node.array.type.pointee_type
        elif isinstance(node.index.type, PointerType):  # 整数表达式 [ 指针表达式 ]
            if not is_integer_type(node.array.type):
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

        node.is_lvalue = False
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
                if not is_scalar_type(node.operand.type):
                    raise Error("不能对非标量类型进行'!'操作", node.location)
                node.type = BasicType(BasicTypeKind.INT)
            case UnaryOpKind.POSITIVE | UnaryOpKind.NEGATIVE:
                if not is_arithmetic_type(node.operand.type):
                    raise Error(
                        f"'{node.op.value}'的操作数应该是算术类型", node.location
                    )
                node.operand = integer_promotion_cast(node.operand, node.operand.type)
                node.type = node.operand.type
            case UnaryOpKind.INVERT:
                if not is_integer_type(node.operand.type):
                    raise Error(f"'{node.op.value}'需要整数类型", node.location)
                node.operand = integer_promotion_cast(node.operand, node.operand.type)
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
                if is_integer_type(a) or isinstance(a, EnumType):
                    pass
                elif is_real_floating_type(a):
                    pass
                elif isinstance(a, PointerType):
                    pass
                else:
                    Error(
                        f"非法的'{node.op.value}'操作数类型:{node.operand.type}",
                        node.operand.location,
                    )
                node.type = node.operand.type
        return node

    @generic_implicit_cast
    def visit_BinaryOperator(self, node: BinaryOperator):
        self.generic_visit(node)

        match node.op:
            case BinOpKind.ASSIGN:
                if not node.left.type.is_complete():
                    raise Error(f"{node.left.type}不完整", node.left.location)
                if not is_modifiable_lvalue(node.left):
                    raise Error("无法修改", node.location)
                node.right = implicit_cast(node.right, remove_qualifier(node.left.type))
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
                if not (is_arithmetic_type(a) and is_arithmetic_type(b)):
                    raise Error(f"'{node.op.value}'需要算术类型", node.location)
                node.type = node.left.type
            case BinOpKind.AADD:
                a = node.left.type
                b = node.right.type
                if isinstance(node.left.type, (AtomicType, QualifiedType)):
                    a = node.left.type.type
                if is_arithmetic_type(a) and is_arithmetic_type(b):
                    pass
                elif isinstance(a, PointerType) and is_integer_type(b):
                    if not a.is_complete():
                        raise Error(f"{a.pointee_type}不完整", node.left.location)
                elif isinstance(b, PointerType) and is_integer_type(a):
                    if not b.is_complete():
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
                if is_arithmetic_type(a) and is_arithmetic_type(b):
                    pass
                elif isinstance(a, PointerType) and is_integer_type(b):
                    if not a.is_complete():
                        raise Error(f"{a.pointee_type}不完整", node.left.location)
                elif isinstance(a, PointerType) and isinstance(b, PointerType):
                    a = remove_qualifier(a.pointee_type)
                    b = remove_qualifier(b.pointee_type)
                    if not is_compatible_type(a, b):
                        raise Error(f"{a}和{b}不兼容", node.location)
                    if not a.is_complete():
                        raise Error(f"{a}不完整", node.left.location)
                    if not b.is_complete():
                        raise Error(f"{b}不完整", node.right.location)
                else:
                    raise Error(
                        f"非法的'{node.op.value}'操作数类型: {a}和{b}",
                        node.location,
                    )
                node.type = node.left.type
            case BinOpKind.AND | BinOpKind.OR:
                if not is_scalar_type(node.left.type) or not is_scalar_type(
                    node.right.type
                ):
                    raise Error(
                        f"不能对非标量类型进行'{node.op.value}'操作", node.location
                    )
                node.type = BasicType(BasicTypeKind.INT)
            case BinOpKind.LT | BinOpKind.LTE | BinOpKind.GT | BinOpKind.GTE:
                node.type = BasicType(BasicTypeKind.INT)
                if is_real_type(node.left.type) and is_real_type(node.right.type):
                    node.left, node.right = usual_arithmetic_cast(node.left, node.right)
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
                node.type = BasicType(BasicTypeKind.INT)
                if is_arithmetic_type(node.left.type) and is_arithmetic_type(
                    node.right.type
                ):
                    node.left, node.right = usual_arithmetic_cast(node.left, node.right)
                elif isinstance(node.left.type, NullPtrType) and isinstance(
                    node.right.type, NullPtrType
                ):
                    pass
                elif isinstance(node.left.type, NullPtrType) and isinstance(
                    node.right.type, PointerType
                ):
                    node.left = implicit_cast(node.left, node.right.type)
                elif isinstance(node.left.type, PointerType) and isinstance(
                    node.right.type, NullPtrType
                ):
                    node.right = implicit_cast(node.right, node.left.type)
                elif isinstance(node.left.type, PointerType) and isinstance(
                    node.right.type, PointerType
                ):
                    a = remove_qualifier(node.left.type.pointee_type)
                    b = remove_qualifier(node.right.type.pointee_type)
                    is_a_void = a == BasicType(BasicTypeKind.VOID)
                    is_b_void = b == BasicType(BasicTypeKind.VOID)
                    if not is_compatible_type(a, b) and not (is_a_void or is_b_void):
                        raise Error(f"{a}和{b}不兼容", node.location)
                    if is_a_void or is_b_void:
                        node.left = implicit_cast(
                            node.left, PointerType(BasicType(BasicTypeKind.VOID))
                        )
                        node.right = implicit_cast(
                            node.right, PointerType(BasicType(BasicTypeKind.VOID))
                        )
                else:
                    raise Error(
                        f"非法的'{node.op.value}'操作数类型: {node.left.type}和{node.right.type}",
                        node.location,
                    )
            case BinOpKind.ADD:
                if is_arithmetic_type(node.left.type) and is_arithmetic_type(
                    node.right.type
                ):
                    node.left, node.right = usual_arithmetic_cast(node.left, node.right)
                elif isinstance(node.left.type, PointerType) and is_integer_type(
                    node.right.type
                ):
                    if not node.left.type.is_complete():
                        raise Error(
                            f"{node.left.type.pointee_type}不完整", node.left.location
                        )
                elif isinstance(node.right.type, PointerType) and is_integer_type(
                    node.left.type
                ):
                    if not node.right.type.is_complete():
                        raise Error(
                            f"{node.right.type.pointee_type}不完整", node.right.location
                        )
                else:
                    raise Error(
                        f"非法的'{node.op.value}'操作数类型: {node.left.type}和{node.right.type}",
                        node.location,
                    )
                node.type = node.left.type
            case BinOpKind.SUB:
                if is_arithmetic_type(node.left.type) and is_arithmetic_type(
                    node.right.type
                ):
                    node.left, node.right = usual_arithmetic_cast(node.left, node.right)
                elif isinstance(node.left.type, PointerType) and is_integer_type(
                    node.right.type
                ):
                    if not node.left.type.is_complete():
                        raise Error(
                            f"{node.left.type.pointee_type}不完整", node.left.location
                        )
                elif isinstance(node.left.type, PointerType) and isinstance(
                    node.right.type, PointerType
                ):
                    a = remove_qualifier(node.left.type.pointee_type)
                    b = remove_qualifier(node.right.type.pointee_type)
                    if not is_compatible_type(a, b):
                        raise Error(f"{a}和{b}不兼容", node.location)
                    if not a.is_complete():
                        raise Error(f"{a}不完整", node.left.location)
                    if not b.is_complete():
                        raise Error(f"{b}不完整", node.right.location)
                else:
                    raise Error(
                        f"非法的'{node.op.value}'操作数类型: {node.left.type}和{node.right.type}",
                        node.location,
                    )
                node.type = node.left.type
            case BinOpKind.MUL | BinOpKind.DIV | BinOpKind.MOD:
                if not (
                    is_arithmetic_type(node.left.type)
                    and is_arithmetic_type(node.right.type)
                ):
                    raise Error(f"'{node.op.value}'运算需要两个算术类型", node.location)
                node.left, node.right = usual_arithmetic_cast(node.left, node.right)
                node.type = node.left.type
            case BinOpKind.BITAND | BinOpKind.BITOR | BinOpKind.BITXOR:
                if not (
                    is_integer_type(node.left.type) and is_integer_type(node.right.type)
                ):
                    raise Error(f"'{node.op.value}'需要整数类型", node.location)
                node.left, node.right = usual_arithmetic_cast(node.left, node.right)
                node.type = node.left.type
            case BinOpKind.LSHIFT | BinOpKind.RSHIFT:
                if not (
                    is_integer_type(node.left.type) and is_integer_type(node.right.type)
                ):
                    raise Error(f"'{node.op.value}'需要整数类型", node.location)
                node.left = integer_promotion_cast(node.left, node.left.type)
                node.right = integer_promotion_cast(node.right, node.right.type)
                node.type = node.left.type
            case BinOpKind.COMMA:
                node.type = node.right.type
        return node

    @generic_implicit_cast
    def visit_TypeOrVarDecl(self, node: TypeOrVarDecl):
        self.generic_visit(node)
        if node.initializer != None:
            node.initializer = implicit_cast(
                node.initializer, remove_qualifier(node.type)
            )
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
            if isinstance(v, ArrayPtrType):
                pass  # TODO: 比较数组大小
            node.args[i] = implicit_cast(node.args[i], v)

        if function_type.has_varparam:
            while i < len(node.args):
                if (
                    isinstance(node.args[i].type, BasicType)
                    and node.args[i].type.kind == BasicTypeKind.FLOAT
                ):
                    node.args[i] = implicit_cast(
                        node.args[i], BasicType(BasicTypeKind.DOUBLE)
                    )
                elif is_integer_type(node.args[i].type):
                    node.args[i] = integer_promotion_cast(
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
                node.expr = implicit_cast(node.expr, i.func_type.return_type)
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

    @generic_implicit_cast
    def visit_ExplicitCast(self, node: ExplicitCast):
        self.generic_visit(node)
        node.type = node.type_name.type
        if not (
            is_scalar_type(node.type) or node.type == BasicType(BasicTypeKind.VOID)
        ):
            raise Error(f"不能转换成{node.type}", node.location)
        if not (
            is_scalar_type(node.expr.type)
            or node.expr.type == BasicType(BasicTypeKind.VOID)
        ):
            raise Error(f"{node.expr.type}不能进行转换", node.location)
        return node

    @generic_implicit_cast
    def visit_ConditionalOperator(self, node: ConditionalOperator):
        self.generic_visit(node)
        if not is_scalar_type(node.condition_expr.type):
            raise Error("条件得是标量类型", node.condition_expr.location)
        if is_arithmetic_type(node.true_expr.type) and is_arithmetic_type(
            node.false_expr.type
        ):
            node.true_expr, node.false_expr = usual_arithmetic_cast(
                node.true_expr, node.false_expr
            )
            node.type = node.true_expr.type
        elif (
            isinstance(node.true_expr.type, RecordType)
            and isinstance(node.false_expr.type, RecordType)
            and node.true_expr.type == node.false_expr.type
        ):
            node.type = node.true_expr.type
        elif (
            node.true_expr.type == BasicType(BasicTypeKind.VOID) == node.false_expr.type
        ):
            node.type = BasicType(BasicTypeKind.VOID)
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

                if a == BasicType(BasicTypeKind.VOID) == b:
                    if qa or qb:
                        pointee_type = QualifiedType(qa + qb, a)
                    node.type = PointerType(pointee_type)
                elif a == BasicType(BasicTypeKind.VOID):
                    node.type = node.false_expr.type
                elif b == BasicType(BasicTypeKind.VOID):
                    node.type = node.true_expr.type
                elif not is_compatible_type(a, b):
                    raise Error(f"{a}和{b}不兼容", node.location)
                else:
                    if a != b:
                        try:
                            pointee_type = composite_type(a, b)
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

    # TODO: sizeof
    # TODO: alignof

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
        node.type = node.select.type
        node.is_lvalue = node.select.is_lvalue
