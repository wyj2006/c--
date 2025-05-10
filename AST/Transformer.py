from typing import Callable
from AST.Visitor import Visitor
from AST.Node import Node


class Transformer(Visitor):
    """遍历语法树节点, 并用visit_XXX的返回值去替换或移除旧节点"""

    def generic_visit(
        self, node: Node, callback: Callable[[Node, "Transformer"], None] = None
    ):
        if callback == None:
            callback = lambda node, visitor: node.accept(visitor)
        for field in node._fields:
            child = getattr(node, field, None)
            if child == None:
                continue
            if isinstance(child, (list, tuple)):
                new_values = []
                for i in child:
                    if i == None:
                        continue
                    if not isinstance(i, Node):
                        new_values.append(i)
                        continue
                    t = callback(i, self)
                    if t != None:
                        new_values.append(t)
                node.__dict__[field] = new_values
                # setattr(node, field, new_values)
            elif isinstance(child, Node):
                t = callback(child, self)
                node.__dict__[field] = t
                # setattr(node, field, t)
        return node
