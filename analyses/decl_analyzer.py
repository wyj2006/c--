from typing import TypedDict
from cast import (
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
    Node,
    Enumerator,
    AlignSpecifier,
    MemberDecl,
)
from basic import (
    Error,
    TAG_NAMES,
    Diagnostics,
    Symbol,
    Note,
)
from analyses.analyzer import Analyzer, block_scope, func_prototype_scope, AnalyzerFlag
from typesystem import (
    VoidType,
    IntType,
    BoolType,
    CharType,
    LongType,
    UIntType,
    FloatType,
    ShortType,
    ULongType,
    BitIntType,
    DoubleType,
    UShortType,
    LongLongType,
    Decimal32Type,
    Decimal64Type,
    ULongLongType,
    Decimal128Type,
    LongDoubleType,
    SCharType,
    FloatComplexType,
    UCharType,
    DoubleComplexType,
    FloatImaginaryType,
    DoubleImaginaryType,
    LongDoubleComplexType,
    LongDoubleImaginaryType,
    TypedefType,
    TypeofType,
    AutoType,
    EnumType,
    ArrayType,
    AtomicType,
    RecordType,
    PointerType,
    ArrayPtrType,
    FunctionType,
    QualifiedType,
)

basictype_combination = {  # 类型组合
    None.__class__: {
        "void": VoidType,
        "char": CharType,
        "short": ShortType,
        "int": IntType,
        "long": LongType,
        "bool": BoolType,
        "float": FloatType,
        "double": DoubleType,
        "_Decimal32": Decimal32Type,
        "_Decimal64": Decimal64Type,
        "_Decimal128": Decimal128Type,
        "signed": IntType,
        "unsigned": UIntType,
    },
    CharType: {
        "signed": SCharType,
        "unsigned": UCharType,
    },
    ShortType: {
        "signed": ShortType,
        "unsigned": UShortType,
    },
    IntType: {
        "signed": IntType,
        "unsigned": UIntType,
        "short": ShortType,
        "long": LongType,
    },
    LongType: {
        "signed": LongType,
        "unsigned": ULongType,
        "long": LongLongType,
        "double": LongDoubleType,
    },
    LongLongType: {
        "signed": LongLongType,
        "unsigned": ULongLongType,
    },
    FloatType: {
        "_Complex": FloatComplexType,
        "_Imaginary": FloatImaginaryType,
    },
    DoubleType: {
        "_Complex": DoubleComplexType,
        "_Imaginary": DoubleImaginaryType,
    },
    LongDoubleType: {
        "_Complex": LongDoubleComplexType,
        "_Imaginary": LongDoubleImaginaryType,
    },
}


class DeclInfoDict(TypedDict):
    qualifiers: list[TypeQualifier]
    storage_classes: list[StorageClass]
    name: str
    type: Node
    is_func_prototype: bool
    declaration: Declaration


def default_declinfo_dict() -> DeclInfoDict:
    return {
        "name": "",
        "qualifiers": [],
        "storage_classes": [],
        "type": None,
        "is_func_prototype": True,
        "declaration": None,
    }


class DeclAnalyzer(Analyzer):
    """
    完成的任务:
    1. 根据声明的specifier和declarator生成对应的类型
    2. 将一些类型加入符号表: typedef, struct, union,enum
    3. 获取声明的变量名(如果有的话)
    4. 将这些信息和其它specifier或storage_class汇集到一个Node(TypeOrVarDecl)里面供后续步骤使用

    在该Visitor中, 每个方法都会额外传一个DeclInfoDict类型的参数
    """

    def visit_Node(self, node, decl_info=None):
        self.generic_visit(
            node,
            callback=lambda node, visitor: (node.accept(visitor, decl_info)),
        )

    def sortSpecifiers(self, specifiers: list[SpecifierOrQualifier]):
        """
        排序Declaration.specifiers, 方便以后的操作
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
                    "bool",
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
        decl_info["declaration"] = node

        for specifier in self.sortSpecifiers(node.specifiers):
            specifier.accept(self, decl_info)

        decl_info["type"].attribute_specifiers.extend(node.specifier_attributes)

        for declarator in node.declarators:
            # 每个declarator使用独立的DeclInfoDict
            # 但由声明符声明的类型是共享的
            sub_decl_info = decl_info.copy()
            sub_decl_info["qualifiers"] = []
            declarator.accept(self, sub_decl_info)

    def visit_SingleDeclration(
        self, node: SingleDeclration, decl_info: DeclInfoDict = None
    ):
        decl_info = decl_info if decl_info != None else default_declinfo_dict()
        decl_info["declaration"] = node

        for specifier in self.sortSpecifiers(node.specifiers):
            specifier.accept(self, decl_info)
        decl_info["type"].attribute_specifiers.extend(node.specifier_attributes)
        # specifier的这些东西与declarator是分开的
        decl_info["qualifiers"] = []
        if node.declarator != None:
            node.declarator.accept(self, decl_info)

        return decl_info  # 给派生类用的

    def visit_TypeName(self, node: TypeName, decl_info: DeclInfoDict = None):
        decl_info = default_declinfo_dict()
        decl_info = self.visit_SingleDeclration(node, decl_info)
        node.type = decl_info["type"]

    @block_scope
    def visit_FunctionDef(self, node: FunctionDef, decl_info: DeclInfoDict = None):
        decl_info = self.visit_SingleDeclration(node, decl_info)

        node.func_name = decl_info["name"]
        node.func_type = decl_info["type"]

        self._visit_CompoundStmt(node.body)

    def visit_Declarator(self, node: Declarator, decl_info: DeclInfoDict):
        self.generic_visit(
            node,
            callback=lambda node, visitor: (node.accept(visitor, decl_info)),
        )

    def visit_TypeOrVarDecl(self, node: TypeOrVarDecl, decl_info: DeclInfoDict):
        self.visit_Declarator(node, decl_info)
        node.name = decl_info["name"]
        node.type = decl_info["type"]
        node.declaration = decl_info["declaration"]
        node.storage_classes = decl_info["storage_classes"]

        align_specifiers = [
            i for i in node.declaration.specifiers if isinstance(i, AlignSpecifier)
        ]
        if len(align_specifiers) > 1:
            raise Error("只能有一个对齐说明符", align_specifiers[1].location)

        if node.is_typedef:
            if not self.cur_symtab.addSymbol(
                node.name, TypedefType(node.name, node.type)
            ):
                raise Error(f"重定义: {node.name}", node.location)

        if hasattr(node, "initializer") and node.initializer != None:
            node.initializer.accept(self, None)

    def visit_MemberDecl(self, node: MemberDecl, decl_info: DeclInfoDict):
        self.visit_TypeOrVarDecl(node, decl_info)
        if node.bit_field != None:
            node.bit_field.accept(self, None)

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
        if self.flag.has(AnalyzerFlag.FUNCPARAM):
            decl_info["type"] = ArrayPtrType(
                decl_info["type"]
            )  # 将数组类型的形参调整到对应的指针类型
        if decl_info["qualifiers"]:
            decl_info["type"] = QualifiedType(
                decl_info["qualifiers"], decl_info["type"]
            )

        self.visit_Declarator(node, decl_info)

        if node.size != None:
            node.size.accept(self, None)

    @func_prototype_scope
    def visitParam(
        self,
        node: FunctionDeclarator,
        functype: FunctionType,
    ):
        for parameter in node.parameters:
            parameter.accept(self, None)
            functype.parameters_type.append(parameter.type)

    def visit_FunctionDeclarator(
        self,
        node: FunctionDeclarator,
        decl_info: DeclInfoDict,
    ):
        return_type = decl_info["type"]
        decl_info["type"] = functype = FunctionType(
            [],
            return_type,
            node.has_varparam,
            node.attribute_specifiers,
        )  # 此时的functype是不完整的
        node.declarator.accept(self, decl_info)

        self.visitParam(node, functype)

    def visit_ParamDecl(self, node: ParamDecl, decl_info: DeclInfoDict = None):
        decl_info = self.visit_SingleDeclration(node, decl_info)

        node.name = decl_info["name"]
        node.type = decl_info["type"]

        if node.name and isinstance(node.type, VoidType):
            raise Error("形参不能拥有 void 类型", node.location)
        if isinstance(node.type, FunctionType):
            node.type = PointerType(node.type)

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

        key = decl_info["type"].__class__
        if (
            key not in basictype_combination
            or node.specifier_name not in basictype_combination[key]
        ):
            raise Error(
                f"无法结合{decl_info['type']}和{node.specifier_name}", node.location
            )
        decl_info["type"] = basictype_combination[key][node.specifier_name]()

    def visit_RecordDecl(self, node: RecordDecl, decl_info: DeclInfoDict):
        name = node.name
        is_anonymous = False
        if not name:
            is_anonymous = True
            name = f"<unnamed {node.struct_or_union} at {node.location}>"

        as_define = True  # 是否以定义的方式处理

        if not node.members_declaration:  # 这是声明
            node.type = self.cur_symtab.lookup(name, TAG_NAMES)
            if node.type != None:
                as_define = False

        if as_define:
            node.type = RecordType(
                node.struct_or_union, name, is_anonymous, node.attribute_specifiers
            )
            if not self.cur_symtab.addSymbol(name, node.type, TAG_NAMES):
                old_symbol: Symbol = self.cur_symtab.lookup(name, TAG_NAMES)
                if node.type != old_symbol:
                    raise Diagnostics(
                        [
                            Error(f"重定义: {name}", node.location),
                            Note("上一个定义", old_symbol.declare_locations[0]),
                        ]
                    )
                node.type = decl_info["type"] = old_symbol

        decl_info["type"] = node.type
        node.type.declare_locations.append(node.location)

        for i in node.members_declaration:
            i.accept(self, None)

    def visit_TypedefSpecifier(self, node: TypedefSpecifier, decl_info: DeclInfoDict):
        decl_info["type"] = self.cur_symtab.lookup(node.specifier_name)
        if decl_info["type"] == None:
            raise Error(f"未定义标识符: {node.specifier_name}", node.location)

    def visit_EnumDecl(self, node: EnumDecl, decl_info: DeclInfoDict):
        name = node.name
        is_anonymous = False
        if not name:
            is_anonymous = True
            name = f"<unnamed enum at {node.location}>"

        if node.specifiers == None:
            node.specifiers = []
        _decl_info = default_declinfo_dict()
        for specifier in self.sortSpecifiers(node.specifiers):
            specifier.accept(self, _decl_info)
        node.underlying_type = _decl_info["type"]

        as_define = True  # 同RecordDecl的as_define

        if not node.enumerators:
            node.type = self.cur_symtab.lookup(name, TAG_NAMES)
            if node.type != None:
                as_define = False

        if as_define:
            node.type = decl_info["type"] = EnumType(
                name, node.underlying_type, is_anonymous, node.attribute_specifiers
            )
            if not self.cur_symtab.addSymbol(name, node.type, TAG_NAMES):
                old_symbol: Symbol = self.cur_symtab.lookup(name, TAG_NAMES)
                if node.type != old_symbol:
                    raise Diagnostics(
                        [
                            Error(f"重定义: {name}", node.location),
                            Note("上一个定义", old_symbol.declare_locations[0]),
                        ]
                    )
                node.type = decl_info["type"] = old_symbol

        decl_info["type"] = node.type
        node.type.declare_locations.append(node.location)

        for i in node.enumerators:
            i.accept(self, decl_info)

    def visit_Enumerator(self, node: Enumerator, decl_info: DeclInfoDict):
        # Enumerator也像一个声明
        node.enum_type = decl_info["type"]
        if node.value != None:
            node.value.accept(self, None)

    def visit_StorageClass(self, node: StorageClass, decl_info: DeclInfoDict):
        if node.specifier == StorageClassSpecifier.AUTO:
            decl_info["type"] = AutoType(None)
        else:
            decl_info["storage_classes"].append(node)

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
        node.size.accept(self, None)
        decl_info["type"] = BitIntType(node.size)
