from contextlib import contextmanager
from enum import Enum
from typing import Union
from AST import (
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
from Basic import Symtab


class AnalyzerState(Enum):
    NORMAL = "normal"  # 正常状态
    FUNCDEF = "function def"  # 进入FunctionDef后切换
    FUNCPARAM = "function parameter"  # 进入函数声明或定义中的参数列表后切换


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
            _state = self.state
            self.state = AnalyzerState.FUNCDEF

        _cur_symtab = self.cur_symtab
        if hasattr(node, "_symtab"):
            self.cur_symtab = node._symtab
        else:
            self.cur_symtab = node._symtab = self.cur_symtab.enterScope(node.location)

        func(self, node, *args, **kwargs)

        self.cur_symtab = _cur_symtab

        if isinstance(node, FunctionDef):
            self.state = _state

    return inner


def func_prototype_scope(func):
    def inner(self: "Analyzer", node: FunctionDeclarator, *args, **kwargs):
        if self.state != AnalyzerState.FUNCDEF:
            _cur_symtab = self.cur_symtab
            if hasattr(node, "_symtab"):
                self.cur_symtab = node._symtab
            else:
                self.cur_symtab = node._symtab = self.cur_symtab.enterScope(
                    node.location
                )

        with self.setState(AnalyzerState.FUNCPARAM):
            func(self, node, *args, **kwargs)

        if self.state != AnalyzerState.FUNCDEF:
            self.cur_symtab = _cur_symtab

    return inner


def file_scope(func):
    def inner(self: "Analyzer", node: TranslationUnit, *args, **kwargs):
        if not hasattr(node, "_symtab"):
            node._symtab = self.cur_symtab
        else:
            assert node._symtab is self.cur_symtab
        func(self, node, *args, **kwargs)


class Analyzer(Visitor):
    def __init__(self, symtab: Symtab):
        super().__init__()
        self.cur_symtab = symtab
        self.state: AnalyzerState = AnalyzerState.NORMAL

    @contextmanager
    def setState(self, state: AnalyzerState):
        _state = self.state
        self.state = state
        yield
        self.state = _state

    def _visit_FunctionDef(self, node: FunctionDef, *args, **kwargs):
        for specifier in node.specifiers:
            specifier.accept(self)
        for declarator in node.declarators:
            declarator.accept(self)
        self._visit_CompoundStmt(node.body)

    @block_scope
    def visit_FunctionDef(self, node: FunctionDef, *args, **kwargs):
        self._visit_FunctionDef(node, *args, **kwargs)

    def _visit_CompoundStmt(self, node: CompoundStmt, *args, **kwargs):
        if node.items == None:
            node.items = []
        for item in node.items:
            item.accept(self)

    @block_scope
    def visit_CompoundStmt(self, node: CompoundStmt, *args, **kwargs):
        self._visit_CompoundStmt(node, *args, **kwargs)

    @block_scope
    def visit_IfStmt(self, node: IfStmt, *args, **kwargs):
        node.condition_expr.accept(self)
        node.body.accept(self)
        if node.else_body != None:
            node.else_body.accept(self)

    @block_scope
    def visit_SwitchStmt(self, node: SwitchStmt, *args, **kwargs):
        node.condition_expr.accept(self)
        node.body.accept(self)

    @block_scope
    def visit_ForStmt(self, node: ForStmt, *args, **kwargs):
        if hasattr(node, "init_decl"):
            node.init_decl.accept(self)
        if hasattr(node, "init_expr"):
            node.init_expr.accept(self)
        if node.condition_expr != None:
            node.condition_expr.accept(self)
        if node.increase_expr != None:
            node.increase_expr.accept(self)
        node.body.accept(self)

    @block_scope
    def visit_WhileStmt(self, node: WhileStmt, *args, **kwargs):
        node.condition_expr.accept(self)
        node.body.accept(self)

    @block_scope
    def visit_DoWhileStmt(self, node: DoWhileStmt, *args, **kwargs):
        node.condition_expr.accept(self)
        node.body.accept(self)

    @func_prototype_scope
    def visitParam(self, node: FunctionDeclarator, *args, **kwargs):
        """访问FunctionDeclarator的参数部分"""
        for param in node.parameters:
            param.accept(self)

    def visit_FunctionDeclarator(self, node: FunctionDeclarator, *args, **kwargs):
        node.declarator.accept(self)
        self.visitParam(node, *args, **kwargs)
