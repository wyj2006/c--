from typing import Union
from Analyze.Analyzer import Analyzer
from AST import (
    TypeOrVarDecl,
    ParamDecl,
    FunctionDef,
    Node,
    Enumerator,
    RecordDecl,
    MemberDecl,
    StorageClass,
    FunctionSpecifier,
    StorageClassSpecifier,
    EnumDecl,
)
from Basic import (
    Object,
    Error,
    Member,
    EnumConst,
    ORDINARY_NAMES,
    MEMBER_NAMES,
    Namespace,
    Parameter,
    Function,
    FunctionType,
    EnumType,
    Note,
    Diagnostics,
    Symbol,
    Diagnostic,
    ArrayType,
)


class SymtabFiller(Analyzer):
    """
    完成的任务:
    1. 将有关的符号插入符号表中
    """

    def addSymbol(
        self,
        node: Node,
        name: str,
        *args,
        cls=Object,
        namespace_name=ORDINARY_NAMES,
    ) -> Symbol:
        new_symbol = cls(name, *args)
        new_symbol.define_location = node.location
        if not self.cur_symtab.addSymbol(name, new_symbol, namespace_name):
            symbol = self.cur_symtab.lookup(name, namespace_name)
            raise Diagnostics(
                [
                    Error(f"重定义: {name}", node.location),
                    Note("上一个定义", symbol.define_location),
                ]
            )
        return new_symbol

    def visit_TypeOrVarDecl(self, node: TypeOrVarDecl):
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

        cls = Object
        args = (node.type, node.storage_classes, node.align_specifier)
        if isinstance(node.type, FunctionType):
            cls = Function
            args = (node.type, node.function_specifiers)
        elif node.function_specifiers:
            raise Error(
                f"非函数不能有 {node.function_specifiers[0].specifier_name} 说明符",
                node.function_specifiers[0].location,
            )

        try:
            symbol = self.addSymbol(node, node.name, *args, cls=cls)
        except Diagnostic as e:
            symbol: Union[Object, Function] = self.cur_symtab.lookup(node.name)
            # cls==Object and symbol.type!=node.type or cls==Function and symbol.type!=node.type
            if symbol.type != node.type:
                raise e
            if (
                isinstance(symbol, Object)
                and symbol.initializer != None
                and node.initializer != None
            ):  # 重定义
                raise e

        if node.initializer != None:
            node.initializer.accept(self)

        if isinstance(symbol, Object):
            symbol.initializer = node.initializer
        elif isinstance(symbol, Function) and node.initializer != None:
            raise Error("函数不能有初始化器", node.location)

    def visit_MemberDecl(self, node: MemberDecl):
        if isinstance(node.type, FunctionType):
            raise Error("成员不能是函数", node.location)
        self.addSymbol(
            node,
            node.name,
            node.type,
            node.bit_field,
            node.storage_classes,
            node.align_specifier,
            cls=Member,
            namespace_name=MEMBER_NAMES,
        )

    def visit_ParamDecl(self, node: ParamDecl):
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

        self.addSymbol(
            node,
            node.name,
            node.type,
            storage_classes,
            cls=Parameter,
        )

    def visit_FunctionDef(self, node: FunctionDef):
        if isinstance(node.func_type.return_type, ArrayType):
            raise Error("函数返回类型不能是数组", node.location)
        self.addSymbol(
            node,
            node.func_name,
            node.func_type,
            [i for i in node.specifiers if isinstance(i, FunctionSpecifier)],
            cls=Function,
        )
        super().visit_FunctionDef(node)

    def visit_Enumerator(self, node: Enumerator):
        if node.value != None:
            node.value.accept(self)
        enumconst: EnumConst = self.addSymbol(
            node,
            node.name,
            node.enum_type,
            node.value,
            cls=EnumConst,
        )
        enum_type: EnumType = enumconst.enum_type
        if enumconst not in enum_type.enumerators:
            enum_type.enumerators.append(enumconst)

    def visit_EnumDecl(self, node: EnumDecl):
        if node.enumerators and node.type.enumerators:
            raise Error(f"重定义: {node.type.name}", node.location)
        self.generic_visit(node)

    def visit_RecordDecl(self, node: RecordDecl):
        if node.members_declaration == None:
            return

        if node.type.member != None:
            raise Error(f"重定义: {node.type.name}", node.location)

        if MEMBER_NAMES in self.cur_symtab.namespaces:
            _member_names = self.cur_symtab[MEMBER_NAMES]
        else:
            _member_names = None

        self.cur_symtab.namespaces[MEMBER_NAMES] = node.type.member = Namespace(
            MEMBER_NAMES
        )

        for i in node.members_declaration:
            i.accept(self)

        if _member_names == None:
            self.cur_symtab.namespaces.pop(MEMBER_NAMES)
        else:
            self.cur_symtab.namespaces[MEMBER_NAMES] = _member_names
