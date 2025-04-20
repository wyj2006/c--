from AST import Visitor, Concat, Choice, Letter, Repeat0, EmptyString


class PosCalculator(Visitor):
    """计算nullable,firstpos,lastpos,followpos"""

    def __init__(self, followpos: dict[Letter, list[Letter]]):
        super().__init__()
        self.followpos = followpos

    def visit_Concat(self, node: Concat):
        self.generic_visit(node)

        node.nullable = all([i.nullable for i in node.exprs])
        node.firstpos = set()
        node.lastpos = set()

        for i in node.exprs:
            node.firstpos |= i.firstpos
            if not i.nullable:
                break

        for i in node.exprs[::-1]:
            node.lastpos |= i.lastpos
            if not i.nullable:
                break

        for i, v in enumerate(node.exprs[:-1]):
            for p in v.lastpos:
                for j in range(i + 1, len(node.exprs)):
                    self.followpos[p] |= node.exprs[j].firstpos
                    if not node.exprs[j].nullable:
                        break

    def visit_Choice(self, node: Choice):
        self.generic_visit(node)

        node.nullable = False
        node.firstpos = set()
        node.lastpos = set()

        for i in node.exprs:
            node.nullable = node.nullable or i.nullable
            node.firstpos |= i.firstpos
            node.lastpos |= i.lastpos

    def visit_Repeat0(self, node: Repeat0):
        node.repeat_expr.accept(self)

        node.nullable = True
        node.firstpos = node.repeat_expr.firstpos
        node.lastpos = node.repeat_expr.lastpos

        for i in node.lastpos:
            self.followpos[i] |= node.firstpos

    def visit_Letter(self, node: Letter):
        node.nullable = False
        node.firstpos = {node}
        node.lastpos = {node}

        if node not in self.followpos:
            self.followpos[node] = set()

    def visit_EmptyString(self, node: EmptyString):
        node.nullable = True
        node.firstpos = set()
        node.lastpos = set()

        if node not in self.followpos:
            self.followpos[node] = set()
