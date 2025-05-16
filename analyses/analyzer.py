from contextlib import contextmanager
from enum import IntEnum
from typing import Union
from basic import (
    Visitor,
    CompoundStmt,
    IfStmt,
    SwitchStmt,
    ForStmt,
    WhileStmt,
    FunctionDef,
    DoWhileStmt,
    FunctionDeclarator,
    TranslationUnit,
)
from basic import Symtab, FlagManager


class AnalyzerFlag(IntEnum):
    NORMAL = 0b1  # 正常状态
    FUNCDEF = 0b10  # 进入FunctionDef后设置
    FUNCPARAM = 0b100  # 进入函数声明或定义中的参数列表后设置


def block_scope(func):
    def inner(
        self: "Analyzer",
        node: Union[
            CompoundStmt,
            IfStmt,
            SwitchStmt,
            ForStmt,
            WhileStmt,
            DoWhileStmt,
            FunctionDef,
        ],
        *args,
        **kwargs
    ):
        if isinstance(node, FunctionDef):
            _flag = self.flag.save()
            self.flag.add(AnalyzerFlag.FUNCDEF)

        _cur_symtab = self.cur_symtab
        if hasattr(node, "_symtab"):
            self.cur_symtab = node._symtab
        else:
            self.cur_symtab = node._symtab = self.cur_symtab.enterScope(node.location)

        ret = func(self, node, *args, **kwargs)

        self.cur_symtab = _cur_symtab

        if isinstance(node, FunctionDef):
            self.flag.restore(_flag)

        return ret

    return inner


def func_prototype_scope(func):
    def inner(self: "Analyzer", node: FunctionDeclarator, *args, **kwargs):
        if not self.flag.has(AnalyzerFlag.FUNCDEF):
            _cur_symtab = self.cur_symtab
            if hasattr(node, "_symtab"):
                self.cur_symtab = node._symtab
            else:
                self.cur_symtab = node._symtab = self.cur_symtab.enterScope(
                    node.location
                )

        with self.setFlag(AnalyzerFlag.FUNCPARAM):
            ret = func(self, node, *args, **kwargs)

        if not self.flag.has(AnalyzerFlag.FUNCDEF):
            self.cur_symtab = _cur_symtab

        return ret

    return inner


def file_scope(func):
    def inner(self: "Analyzer", node: TranslationUnit, *args, **kwargs):
        if not hasattr(node, "_symtab"):
            node._symtab = self.cur_symtab
        else:
            assert node._symtab is self.cur_symtab
        func(self, node, *args, **kwargs)

    return inner


class Analyzer(Visitor):
    def __init__(self, symtab: Symtab):
        super().__init__()
        self.cur_symtab = symtab
        self.flag = FlagManager(AnalyzerFlag.NORMAL)

    @contextmanager
    def setFlag(self, flag: AnalyzerFlag):
        """临时设置一个flag"""
        _flag = self.flag.save()
        self.flag.add(flag)
        yield
        self.flag.restore(_flag)

    def _visit_FunctionDef(self, node: FunctionDef, *args, **kwargs):
        for specifier in node.specifiers:
            specifier.accept(self)
        for declarator in node.declarators:
            declarator.accept(self)
        self._visit_CompoundStmt(node.body)
        return node

    @block_scope
    def visit_FunctionDef(self, node: FunctionDef, *args, **kwargs):
        return self._visit_FunctionDef(node, *args, **kwargs)

    def _visit_CompoundStmt(self, node: CompoundStmt, *args, **kwargs):
        if node.items == None:
            node.items = []
        for item in node.items:
            item.accept(self)
        return node

    @block_scope
    def visit_CompoundStmt(self, node: CompoundStmt, *args, **kwargs):
        return self._visit_CompoundStmt(node, *args, **kwargs)

    @block_scope
    def visit_IfStmt(self, node: IfStmt, *args, **kwargs):
        node.condition_expr.accept(self)
        node.body.accept(self)
        if node.else_body != None:
            node.else_body.accept(self)
        return node

    @block_scope
    def visit_SwitchStmt(self, node: SwitchStmt, *args, **kwargs):
        node.condition_expr.accept(self)
        node.body.accept(self)
        return node

    @block_scope
    def visit_ForStmt(self, node: ForStmt, *args, **kwargs):
        if node.init != None:
            node.init.accept(self)
        if node.condition_expr != None:
            node.condition_expr.accept(self)
        if node.increase_expr != None:
            node.increase_expr.accept(self)
        node.body.accept(self)
        return node

    @block_scope
    def visit_WhileStmt(self, node: WhileStmt, *args, **kwargs):
        node.condition_expr.accept(self)
        node.body.accept(self)
        return node

    @block_scope
    def visit_DoWhileStmt(self, node: DoWhileStmt, *args, **kwargs):
        node.condition_expr.accept(self)
        node.body.accept(self)
        return node

    @func_prototype_scope
    def visitParam(self, node: FunctionDeclarator, *args, **kwargs):
        """访问FunctionDeclarator的参数部分"""
        for param in node.parameters:
            param.accept(self)

    def visit_FunctionDeclarator(self, node: FunctionDeclarator, *args, **kwargs):
        node.declarator.accept(self)
        self.visitParam(node, *args, **kwargs)
        return node
