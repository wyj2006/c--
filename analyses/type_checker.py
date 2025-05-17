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
)
from basic import (
    Error,
)
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
)
from analyses.analyzer import Analyzer
from analyses.cast_operation import (
    implicit_cast,
    generic_implicit_cast,
    is_lvalue,
    usual_arithmetic_cast,
    integer_promotion_cast,
    is_modifiable_lvalue,
)


class TypeChecker(Analyzer, Transformer):
    def __init__(self, symtab):
        super().__init__(symtab)
        self.path = []  # 调用路径

    @generic_implicit_cast
    def visit_Expr(self, node: Expr):
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
                if not node.operand.type.is_scalar_type:
                    raise Error("不能对非标量类型进行'!'操作", node.location)
                node.type = IntType()
            case UnaryOpKind.POSITIVE | UnaryOpKind.NEGATIVE:
                if not node.operand.type.is_arithmetic_type:
                    raise Error(
                        f"'{node.op.value}'的操作数应该是算术类型", node.location
                    )
                node.operand = integer_promotion_cast(node.operand, node.operand.type)
                node.type = node.operand.type
            case UnaryOpKind.INVERT:
                if not node.operand.type.is_integer_type:
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
                    if not a.is_complete():
                        raise Error(f"{a.pointee_type}不完整", node.left.location)
                elif isinstance(b, PointerType) and a.is_integer_type:
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
                if a.is_arithmetic_type and b.is_arithmetic_type:
                    pass
                elif isinstance(a, PointerType) and b.is_integer_type:
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
                node.type = IntType()
                if (
                    node.left.type.is_arithmetic_type
                    and node.right.type.is_arithmetic_type
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
                    is_a_void = a == VoidType()
                    is_b_void = b == VoidType()
                    if not is_compatible_type(a, b) and not (is_a_void or is_b_void):
                        raise Error(f"{a}和{b}不兼容", node.location)
                    if is_a_void or is_b_void:
                        node.left = implicit_cast(node.left, PointerType(VoidType()))
                        node.right = implicit_cast(node.right, PointerType(VoidType()))
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
                    node.left, node.right = usual_arithmetic_cast(node.left, node.right)
                elif (
                    isinstance(node.left.type, PointerType)
                    and node.right.type.is_integer_type
                ):
                    if not node.left.type.is_complete():
                        raise Error(
                            f"{node.left.type.pointee_type}不完整", node.left.location
                        )
                elif (
                    isinstance(node.right.type, PointerType)
                    and node.left.type.is_integer_type
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
                if (
                    node.left.type.is_arithmetic_type
                    and node.right.type.is_arithmetic_type
                ):
                    node.left, node.right = usual_arithmetic_cast(node.left, node.right)
                elif (
                    isinstance(node.left.type, PointerType)
                    and node.right.type.is_integer_type
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
                    node.left.type.is_arithmetic_type
                    and node.right.type.is_arithmetic_type
                ):
                    raise Error(f"'{node.op.value}'运算需要两个算术类型", node.location)
                node.left, node.right = usual_arithmetic_cast(node.left, node.right)
                node.type = node.left.type
            case BinOpKind.BITAND | BinOpKind.BITOR | BinOpKind.BITXOR:
                if not (
                    node.left.type.is_integer_type and node.right.type.is_integer_type
                ):
                    raise Error(f"'{node.op.value}'需要整数类型", node.location)
                node.left, node.right = usual_arithmetic_cast(node.left, node.right)
                node.type = node.left.type
            case BinOpKind.LSHIFT | BinOpKind.RSHIFT:
                if not (
                    node.left.type.is_integer_type and node.right.type.is_integer_type
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
                if isinstance(node.args[i].type, FloatType):
                    node.args[i] = implicit_cast(node.args[i], DoubleType)
                elif node.args[i].type.is_integer_type:
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
    def visit_ConditionalOperator(self, node: ConditionalOperator):
        self.generic_visit(node)
        if not node.condition_expr.type.is_scalar_type:
            raise Error("条件得是标量类型", node.condition_expr.location)
        if (
            node.true_expr.type.is_arithmetic_type
            and node.false_expr.type.is_arithmetic_type
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
