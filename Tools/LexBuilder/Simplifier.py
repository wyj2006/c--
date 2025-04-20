from AST import Transformer, Choice, Concat


class Simplifier(Transformer):
    """化简正则表达式"""

    def visit_Choice(self, node: Choice):
        i = 0
        while i < len(node.exprs):
            node.exprs[i] = node.exprs[i].accept(self)
            if isinstance(node.exprs[i], Choice):
                node.exprs[i : i + 1] = node.exprs[i].exprs
                continue
            i += 1

        if len(node.exprs) == 0:
            return None
        if len(node.exprs) == 1:
            return node.exprs[0]

        return node

    def visit_Concat(self, node: Concat):
        i = 0
        while i < len(node.exprs):
            node.exprs[i] = node.exprs[i].accept(self)
            if isinstance(node.exprs[i], Concat):
                node.exprs[i : i + 1] = node.exprs[i].exprs
                continue
            i += 1

        if len(node.exprs) == 0:
            return None
        if len(node.exprs) == 1:
            return node.exprs[0]

        return node
