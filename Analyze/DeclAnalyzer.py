from typing import TypedDict
from AST import (
    Visitor,
    Declarator,
    NameDeclarator,
    BasicTypeSpecifier,
    Declaration,
    FunctionDef,
    TypeOrVarDecl,
    PointerDeclarator,
    ArrayDeclarator,
    FunctionDeclarator,
    ParamDecl,
    FunctionDef,
    RecordDecl,
    TypedefSpecifier,
    EnumDecl,
    StorageClass,
    TypeQualifier,
    SpecifierOrQualifier,
    StorageClassSpecifier,
    TypeQualifierKind,
    AtomicSpecifier,
    TypeOfSpecifier,
    Expr,
    TypeName,
    BitIntSpecifier,
    SingleDeclration,
    CompoundStmt,
    IfStmt,
    SwitchStmt,
    ForStmt,
    WhileStmt,
    DoWhileStmt,
    Node,
)
from Basic import (
    Symtab,
    BasicType,
    PointerType,
    ArrayType,
    FunctionType,
    RecordType,
    TypedefType,
    EnumType,
    QualifiedType,
    AtomicType,
    TypeofType,
    BitIntType,
    Error,
    TAG_NAMES,
)


class DeclInfoDict(TypedDict):
    qualifiers: list[TypeQualifier]
    store_classes: list[StorageClass]
    name: str
    type: Node
    is_func_prototype: bool


def default_declinfo_dict() -> DeclInfoDict:
    return {
        "name": "",
        "qualifiers": [],
        "store_classes": [],
        "type": None,
        "is_func_prototype": True,
    }


def block_scope(func):
    def inner(self: "DeclAnalyzer", node: Node, *args, **kwargs):
        self.cur_symtab = self.cur_symtab.enterScope(node.location)
        func(self, node, *args, **kwargs)
        self.cur_symtab = self.cur_symtab.leaveScope()

    return inner


class DeclAnalyzer(Visitor):
    """在该Visitor中, 每个方法都会额外传一个DeclInfoDict类型的参数"""

    def __init__(self, symtab: Symtab):
        super().__init__()
        self.cur_symtab = symtab

    def visit_Node(self, node, decl_info=None):
        self.generic_visit(
            node,
            callback=lambda node, visitor: (node.accept(visitor, decl_info)),
        )

    def sortSpecifiers(self, specifiers: list[SpecifierOrQualifier]):
        """
        排序Declaration.specifiers
        并对BasicTypeSpecifier进行组合
        """

        def get_key(a: SpecifierOrQualifier):
            key = []
            # 针对大类排序
            if isinstance(a, StorageClass):
                key.append(0)
                # 针对小类分类
                storage_class_list = [
                    StorageClassSpecifier.TYPEDEF,
                    StorageClassSpecifier.CONSTEXPR,
                    StorageClassSpecifier.THREAD_LOCAL,
                    StorageClassSpecifier.STATIC,
                    StorageClassSpecifier.EXTERN,
                    StorageClassSpecifier.AUTO,
                    StorageClassSpecifier.REGISTER,
                ]
                key.append(storage_class_list.index(a.specifier))
            elif isinstance(a, BitIntSpecifier):
                key.append(1)
            elif isinstance(a, BasicTypeSpecifier):
                key.append(2)
                basic_type_list = [
                    "void",
                    "_Decimal32",
                    "_Decimal64",
                    "_Decimal128",
                    "char",
                    "int",
                    "long",
                    "float",
                    "double",
                    "_Bool",
                    "_Complex",
                    "_Imaginary",
                    "unsigned",
                    "signed",
                ]
                key.append(basic_type_list.index(a.specifier_name))
            elif isinstance(a, TypeQualifier):
                key.append(4)
                qualifier_list = [
                    TypeQualifierKind.CONST,
                    TypeQualifierKind.RESTRICT,
                    TypeQualifierKind.VOLATILE,
                    TypeQualifierKind._ATOMIC,
                ]
                key.append(qualifier_list.index(a.qualifier))
            else:
                key.append(3)
            return key

        return sorted(specifiers, key=get_key)

    def visit_Declaration(self, node: Declaration, decl_info: DeclInfoDict = None):
        decl_info = decl_info if decl_info != None else default_declinfo_dict()

        for specifier in self.sortSpecifiers(node.specifiers):
            specifier.accept(self, decl_info)

        for declarator in node.declarators:
            # 每个declarator使用独立的DeclInfoDict
            # 但由声明符声明的类型是共享的
            sub_decl_info = default_declinfo_dict()
            sub_decl_info["store_classes"] = decl_info["store_classes"]
            sub_decl_info["type"] = decl_info["type"]
            sub_decl_info["is_func_prototype"] = decl_info["is_func_prototype"]
            declarator.accept(self, sub_decl_info)

    def visit_SingleDeclration(
        self, node: SingleDeclration, decl_info: DeclInfoDict = None
    ):
        decl_info = decl_info if decl_info != None else default_declinfo_dict()

        for specifier in self.sortSpecifiers(node.specifiers):
            specifier.accept(self, decl_info)
        # specifier的这些东西与declarator是分开的
        decl_info["qualifiers"] = []
        node.declarator.accept(self, decl_info)

        return decl_info  # 给派生类用的

    def visit_TypeName(self, node: TypeName, decl_info: DeclInfoDict = None):
        decl_info = self.visit_SingleDeclration(node, decl_info)
        node.type = decl_info["type"]

    def visit_FunctionDef(self, node: FunctionDef, decl_info: DeclInfoDict = None):
        self.cur_symtab = self.cur_symtab.enterScope(node.location)

        decl_info = decl_info if decl_info != None else default_declinfo_dict()
        decl_info["is_func_prototype"] = False

        decl_info = self.visit_SingleDeclration(node, decl_info)

        node.func_name = decl_info["name"]
        node.func_type = decl_info["type"]

        if node.body.items == None:
            node.body.items = []
        for item in node.body.items:
            item.accept(self, None)

        self.cur_symtab.leaveScope()

    def visit_Declarator(self, node: Declarator, decl_info: DeclInfoDict):
        self.generic_visit(
            node,
            callback=lambda node, visitor: (node.accept(visitor, decl_info)),
        )

    def visit_TypeOrVarDecl(self, node: TypeOrVarDecl, decl_info: DeclInfoDict):
        self.visit_Declarator(node, decl_info)
        node.name = decl_info["name"]
        node.type = decl_info["type"]
        if (
            decl_info["store_classes"]
            and decl_info["store_classes"][0].specifier == StorageClassSpecifier.TYPEDEF
        ):
            node.is_typedef = True
            if not self.cur_symtab.addSymbol(
                node.name, TypedefType(node.name, node.type)
            ):
                raise Error(f"重定义: {node.name}", node.location)
        else:
            node.is_typedef = False

    def visit_NameDeclarator(self, node: NameDeclarator, decl_info: DeclInfoDict):
        decl_info["name"] = node.name

    def visit_PointerDeclarator(self, node: PointerDeclarator, decl_info: DeclInfoDict):
        pointee_type = decl_info["type"]
        decl_info["type"] = PointerType(pointee_type, node.attribute_specifiers)
        if decl_info["qualifiers"]:
            decl_info["type"] = QualifiedType(
                decl_info["qualifiers"], decl_info["type"]
            )

        self.visit_Declarator(node, decl_info)

    def visit_ArrayDeclarator(self, node: ArrayDeclarator, decl_info: DeclInfoDict):
        element_type = decl_info["type"]
        decl_info["type"] = ArrayType(
            element_type,
            node.size,
            node.is_star_modified,
            node.is_static,
            node.attribute_specifiers,
        )
        if decl_info["qualifiers"]:
            decl_info["type"] = QualifiedType(
                decl_info["qualifiers"], decl_info["type"]
            )

        self.visit_Declarator(node, decl_info)

    def visit_FunctionDeclarator(
        self, node: FunctionDeclarator, decl_info: DeclInfoDict
    ):
        return_type = decl_info["type"]
        decl_info["type"] = functype = FunctionType(
            [], return_type, node.has_varparam, node.attribute_specifiers
        )  # 不完整
        node.declarator.accept(self, decl_info)

        if decl_info["is_func_prototype"]:
            self.cur_symtab = self.cur_symtab.enterScope(node.location)
        for parameter in node.parameters:
            parameter.accept(self, None)
            functype.parameters_type.append(parameter.type)
        if decl_info["is_func_prototype"]:
            self.cur_symtab = self.cur_symtab.leaveScope()

    def visit_ParamDecl(self, node: ParamDecl, decl_info: DeclInfoDict = None):
        decl_info = self.visit_SingleDeclration(node, decl_info)

        node.name = decl_info["name"]
        node.type = decl_info["type"]

    def visit_BasicTypeSpecifier(
        self, node: BasicTypeSpecifier, decl_info: DeclInfoDict
    ):
        if isinstance(decl_info["type"], BitIntType):
            if node.specifier_name == "unsigned":
                decl_info["type"].signed = False
                return
            elif node.specifier_name == "signed":
                decl_info["type"].signed = True
                return
        if decl_info["type"] != None and (
            not isinstance(decl_info["type"], BasicType)
            or decl_info["type"].kind not in BasicType.combination
            or node.specifier_name not in BasicType.combination[decl_info["type"].kind]
        ):
            raise Error(
                f"无法结合{decl_info['type']}和{node.specifier_name}", node.location
            )
        if decl_info["type"] != None:
            decl_info["type"] = BasicType(
                BasicType.combination[decl_info["type"].kind][node.specifier_name]
            )
        else:
            decl_info["type"] = BasicType(
                BasicType.combination[None][node.specifier_name]
            )

    def visit_RecordDecl(self, node: RecordDecl, decl_info: DeclInfoDict):
        name = node.name
        if not name:
            name = f"<unnamed {node.struct_or_union} at {node.location}>"

        if (a := self.cur_symtab.lookup(name, TAG_NAMES)) != None:
            decl_info["type"] = a
            return

        decl_info["type"] = RecordType(
            node.struct_or_union, name, node.attribute_specifiers
        )

        if not self.cur_symtab.addSymbol(name, decl_info["type"], TAG_NAMES):
            raise Error(f"重定义: {name}", node.location)

        for i in node.members_declaration:
            i.accept(self, None)

    def visit_TypedefSpecifier(self, node: TypedefSpecifier, decl_info: DeclInfoDict):
        decl_info["type"] = self.cur_symtab.lookup(node.specifier_name)
        if decl_info["type"] == None:
            raise Error(f"未定义标识符: {node.specifier_name}", node.location)

    def visit_EnumDecl(self, node: EnumDecl, decl_info: DeclInfoDict):
        name = node.name
        if not name:
            name = f"<unnamed enum at {node.location}>"

        if (a := self.cur_symtab.lookup(name, TAG_NAMES)) != None:
            decl_info["type"] = a
            return

        decl_info["type"] = EnumType(name, node.attribute_specifiers)

        if not self.cur_symtab.addSymbol(name, decl_info["type"], TAG_NAMES):
            raise Error(f"重定义: {name}", node.location)

        for i in node.enumerators:
            i.accept(self, None)

    def visit_StorageClass(self, node: StorageClass, decl_info: DeclInfoDict):
        decl_info["store_classes"].append(node)

    def visit_TypeQualifier(self, node: TypeQualifier, decl_info: DeclInfoDict):
        if node.qualifier == TypeQualifierKind._ATOMIC:
            decl_info["type"] = AtomicType(decl_info["type"])
        elif isinstance(decl_info["type"], QualifiedType):
            decl_info["type"].qualifiers.append(node)
        else:
            decl_info["type"] = QualifiedType([node], decl_info["type"])

    def visit_AtomicSpecifier(self, node: AtomicSpecifier, decl_info: DeclInfoDict):
        node.type_name.accept(self, None)
        decl_info["type"] = AtomicType(node.type_name.type)

    def visit_TypeOfSpecifier(self, node: TypeOfSpecifier, decl_info: DeclInfoDict):
        node.arg.accept(self, None)
        if isinstance(node.arg, Expr):
            decl_info["type"] = TypeofType(node.arg, node.is_unqual)
        else:
            decl_info["type"] = TypeofType(node.arg.type, node.is_unqual)

    def visit_BitIntSpecifier(self, node: BitIntSpecifier, decl_info: DeclInfoDict):
        decl_info["type"] = BitIntType(node.size)

    @block_scope
    def visit_CompoundStmt(self, node: CompoundStmt, _: DeclInfoDict = None):
        if node.items == None:
            node.items = []
        for item in node.items:
            item.accept(self, None)

    @block_scope
    def visit_IfStmt(self, node: IfStmt, _=None):
        node.condition_expr.accept(self, None)
        node.body.accept(self, None)
        if node.else_body != None:
            node.else_body.accept(self, None)

    @block_scope
    def visit_SwitchStmt(self, node: SwitchStmt, _=None):
        node.condition_expr.accept(self, None)
        node.body.accept(self, None)

    @block_scope
    def visit_ForStmt(self, node: ForStmt, _=None):
        if hasattr(node, "init_decl"):
            node.init_decl.accept(self, None)
        if hasattr(node, "init_expr"):
            node.init_expr.accept(self, None)
        if node.condition_expr != None:
            node.condition_expr.accept(self, None)
        if node.increase_expr != None:
            node.increase_expr.accept(self, None)
        node.body.accept(self, None)

    @block_scope
    def visit_WhileStmt(self, node: WhileStmt, _=None):
        node.condition_expr.accept(self, None)
        node.body.accept(self, None)

    @block_scope
    def visit_DoWhileStmt(self, node: DoWhileStmt, _=None):
        node.condition_expr.accept(self, None)
        node.body.accept(self, None)
