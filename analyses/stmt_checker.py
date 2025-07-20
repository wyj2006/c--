from .analyzer import Analyzer
from cast import (
    IfStmt,
    SwitchStmt,
    WhileStmt,
    DoWhileStmt,
    ContinueStmt,
    BreakStmt,
    ReturnStmt,
    ForStmt,
    FunctionDef,
)
from basic import Error


class StmtChecker(Analyzer):
    def __init__(self, symtab):
        super().__init__(symtab)
        self.path = []

    def visit_IfStmt(self, node: IfStmt):
        if not node.condition_expr.type.is_scalar_type:
            raise Error("if语句表达式必须是标量类型", node.condition_expr.location)
        self.generic_visit(node)

    def visit_SwitchStmt(self, node: SwitchStmt):
        if not node.condition_expr.type.is_integer_type:
            raise Error("switch的条件必须是整数类型", node.condition_expr.location)
        super().visit_SwitchStmt(node)

    def visit_WhileStmt(self, node: WhileStmt):
        if not node.condition_expr.type.is_scalar_type:
            raise Error("while语句表达式必须是标量类型", node.condition_expr.location)
        super().visit_WhileStmt(node)

    def visit_DoWhileStmt(self, node: DoWhileStmt):
        if not node.condition_expr.type.is_scalar_type:
            raise Error(
                "do-while语句表达式必须是标量类型", node.condition_expr.location
            )
        super().visit_DoWhileStmt(node)

    def visit_ContinueStmt(self, node: ContinueStmt):
        for i in self.path:
            if isinstance(i, (WhileStmt, DoWhileStmt, ForStmt)):
                break
        else:
            raise Error("continue只能用在循环中", node.location)

    def visit_BreakStmt(self, node: BreakStmt):
        for i in self.path:
            if isinstance(i, (ForStmt, WhileStmt, DoWhileStmt, SwitchStmt)):
                break
        else:
            raise Error("break只能出现在循环或switch中", node.location)

    def visit_ReturnStmt(self, node: ReturnStmt):
        for i in self.path:
            if isinstance(i, FunctionDef):
                break
        else:
            raise Error("return只能出现在函数定义中", node.location)
