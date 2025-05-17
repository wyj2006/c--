from typing import Union
from analyses.analyzer import Analyzer, AnalyzerFlag, block_scope
from cast import (
    TypeOrVarDecl,
    ParamDecl,
    FunctionDef,
    Enumerator,
    RecordDecl,
    MemberDecl,
    StorageClass,
    FunctionSpecifier,
    StorageClassSpecifier,
    EnumDecl,
    AlignSpecifier,
    Reference,
)
from basic import (
    Object,
    Error,
    Member,
    EnumConst,
    MEMBER_NAMES,
    Symbol,
    Parameter,
    Function,
    Note,
    Diagnostics,
    Symtab,
)
from typesystem import FunctionType, EnumType, ArrayType


class SymtabFiller(Analyzer):
    """
    完成的任务:
    1. 将有关的符号插入符号表中
    2. 确定 Reference 引用的符号
    """

    def visit_TypeOrVarDecl(self, node: TypeOrVarDecl):
        node.declarator.accept(self)

        if node.is_typedef:
            return

        storage_specifiers = [i.specifier for i in node.storage_classes]
        if (
            StorageClassSpecifier.AUTO in storage_specifiers
            and StorageClassSpecifier.TYPEDEF in storage_specifiers
        ):
            auto_index = storage_specifiers.index(StorageClassSpecifier.AUTO)
            raise Error(
                "auto和typedef不能同时出现", node.storage_classes[auto_index].location
            )
        elif StorageClassSpecifier.THREAD_LOCAL in storage_specifiers:
            t = storage_specifiers[:]
            t.remove(StorageClassSpecifier.THREAD_LOCAL)
            if StorageClassSpecifier.STATIC in t:
                t.remove(StorageClassSpecifier.STATIC)
            if StorageClassSpecifier.EXTERN in t:
                t.remove(StorageClassSpecifier.EXTERN)
            if len(t) > 0:
                index = storage_specifiers.index(t[0])
                raise Error(
                    f"thread_local和{t[0].value}不能同时出现",
                    node.storage_classes[index].location,
                )
        elif StorageClassSpecifier.CONSTEXPR in storage_specifiers:
            t = storage_specifiers[:]
            t.remove(StorageClassSpecifier.CONSTEXPR)
            if StorageClassSpecifier.STATIC in t:
                t.remove(StorageClassSpecifier.STATIC)
            if StorageClassSpecifier.AUTO in t:
                t.remove(StorageClassSpecifier.AUTO)
            if StorageClassSpecifier.REGISTER in t:
                t.remove(StorageClassSpecifier.REGISTER)
            if len(t) > 0:
                index = storage_specifiers.index(t[0])
                raise Error(
                    f"constexpr和{t[0].value}不能同时出现",
                    node.storage_classes[index].location,
                )
        elif len(storage_specifiers) > 1:
            raise Error(
                f"{node.storage_classes[0].specifier.value}无法和{node.storage_classes[1].specifier.value}结合",
                node.storage_classes[1].location,
            )

        if isinstance(node.type, FunctionType):
            symbol = Function(
                node.name,
                node.type,
                node.function_specifiers,
                node.attribute_specifiers,
            )
        elif node.function_specifiers:
            raise Error(
                f"非函数不能有 {node.function_specifiers[0].specifier_name} 说明符",
                node.function_specifiers[0].location,
            )
        else:
            symbol = Object(
                node.name,
                node.type,
                node.storage_classes,
                node.align_specifier,
                node.attribute_specifiers,
            )

        if not self.cur_symtab.addSymbol(node.name, symbol):
            old_symbol: Union[Object, Function] = self.cur_symtab.lookup(node.name)
            diagnostics = Diagnostics(
                [
                    Error(f"重定义: {node.name}", node.location),
                    Note("上一个定义", old_symbol.define_location),
                ]
            )
            if (
                not isinstance(old_symbol, symbol.__class__)
                and symbol.type != old_symbol.type
            ):
                raise diagnostics
            if (
                isinstance(symbol, Object)
                and old_symbol.define_location != None
                and node.initializer != None
            ):
                raise diagnostics
            symbol = old_symbol

        symbol.declare_locations.append(node.location)

        if node.initializer != None:
            symbol.define_location = node.location
            node.initializer.accept(self)

        if isinstance(symbol, Object):
            symbol.initializer = node.initializer
        elif isinstance(symbol, Function) and node.initializer != None:
            raise Error("函数不能有初始化器", node.location)

    def visit_MemberDecl(self, node: MemberDecl):
        if isinstance(node.type, FunctionType):
            raise Error("成员不能是函数", node.location)

        self.generic_visit(node)

        symbol = Member(
            node.name,
            node.type,
            node.bit_field,
            node.storage_classes,  # 实际上Member是不会有storage_class的, 这是在语法分析时的限制
            node.align_specifier,
            node.attribute_specifiers,
        )

        if not self.cur_symtab.addSymbol(
            node.name, symbol, namespace_name=MEMBER_NAMES
        ):
            old_symbol: Symbol = self.cur_symtab.lookup(
                node.name, namespace_name=MEMBER_NAMES
            )
            raise Diagnostics(
                [
                    Error(f"重定义: {node.name}", node.location),
                    Note("上一个定义", old_symbol.define_location),
                ]
            )
        symbol.define_location = node.location
        symbol.declare_locations.append(node.location)

    def visit_ParamDecl(self, node: ParamDecl):
        if not node.name:
            return

        self.generic_visit(node)

        if not self.flag.has(AnalyzerFlag.FUNCDEF):
            return

        storage_classes: list[StorageClass] = [
            i for i in node.specifiers if isinstance(i, StorageClass)
        ]
        specifiers = [i.specifier for i in storage_classes]
        length = len(specifiers)
        if StorageClassSpecifier.REGISTER in specifiers:
            length -= 1
        if length != 0:
            i = 0
            while specifiers[i] == StorageClassSpecifier.REGISTER:
                i += 1
            raise Error(
                "对形参允许的存储类说明符仅有 register", storage_classes[i].location
            )

        align_specifiers = [i for i in node.specifiers if isinstance(i, AlignSpecifier)]
        if align_specifiers:
            raise Error("函数参数没有对齐说明符", node.location)

        symbol = Parameter(
            node.name, node.type, storage_classes, None, node.attribute_specifiers
        )

        if not self.cur_symtab.addSymbol(node.name, symbol):
            old_symbol: Symbol = self.cur_symtab.lookup(node.name)
            raise Diagnostics(
                [
                    Error(f"重定义: {node.name}", node.location),
                    Note("上一个定义", old_symbol.define_location),
                ]
            )
        symbol.define_location = node.location
        symbol.declare_locations.append(node.location)

    @block_scope
    def visit_FunctionDef(self, node: FunctionDef):
        if isinstance(node.func_type.return_type, ArrayType):
            raise Error("函数返回类型不能是数组", node.location)

        for specifier in node.specifiers:
            specifier.accept(self)
        for declarator in node.declarators:
            declarator.accept(self)

        # 此时已经进入了函数作用域, 但函数符号是添加在上一层的
        symtab: Symtab = self.cur_symtab.parent
        symbol = Function(
            node.func_name,
            node.func_type,
            [i for i in node.specifiers if isinstance(i, FunctionSpecifier)],
            node.attribute_specifiers,
        )
        if not symtab.addSymbol(node.func_name, symbol):
            old_symbol: Symbol = symtab.lookup(node.func_name)
            diagnostics = Diagnostics(
                [
                    Error(f"重定义: {node.func_name}", node.location),
                    Note("上一个定义", old_symbol.define_location),
                ]
            )
            if not isinstance(old_symbol, Function) or symbol.type != old_symbol.type:
                raise diagnostics
            if old_symbol.define_location != None:
                raise diagnostics
            symbol = old_symbol
        symbol.define_location = node.location

        self._visit_CompoundStmt(node.body)

    def visit_Enumerator(self, node: Enumerator):
        if node.value != None:
            node.value.accept(self)

        symbol: EnumConst = EnumConst(
            node.name, node.enum_type, node.value, node.attribute_specifiers
        )

        if not self.cur_symtab.addSymbol(node.name, symbol):
            old_symbol: Symbol = self.cur_symtab.lookup(node.name)
            raise Diagnostics(
                [
                    Error(f"重定义: {node.name}", node.location),
                    Note("上一个定义", old_symbol.define_location),
                ]
            )

        symbol.define_location = node.location
        symbol.declare_locations.append(node.location)

        enum_type: EnumType = symbol.enum_type
        enum_type.enumerators[symbol.name] = symbol

    def visit_EnumDecl(self, node: EnumDecl):
        if not node.enumerators:
            return

        # 由于可能没有名称, 所以不能通过查找的方法
        symbol: Symbol = node.type
        if symbol.define_location != None:
            raise Diagnostics(
                [
                    Error(f"重定义: {node.type.name}", node.location),
                    Note("上一个定义", symbol.define_location),
                ]
            )
        symbol.define_location = node.location
        self.generic_visit(node)

    def visit_RecordDecl(self, node: RecordDecl):
        if not node.members_declaration:
            return

        symbol: Symbol = node.type
        if symbol.define_location != None:
            raise Diagnostics(
                [
                    Error(f"重定义: {node.type.name}", node.location),
                    Note("上一个定义", symbol.define_location),
                ]
            )
        symbol.define_location = node.location

        _member_names = self.cur_symtab.member_names
        self.cur_symtab.member_names = node.type.members = {}

        for i in node.members_declaration:
            i.accept(self)

        self.cur_symtab.member_names = _member_names

    def visit_Reference(self, node: Reference):
        node.symbol = self.cur_symtab.lookup(node.name)
        if node.symbol == None:
            raise Error(f"未定义: {node.name}", node.location)
