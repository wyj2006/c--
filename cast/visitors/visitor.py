from typing import Callable
from cast import Node


class Visitor:
    """遍历语法树节点"""

    def generic_visit(
        self, node: Node, callback: Callable[[Node, "Visitor"], None] = None
    ):
        """
        通用访问方法
        """
        if callback == None:
            callback = lambda node, visitor: node.accept(visitor)
        for field in node._fields:
            child = getattr(node, field, None)
            if child == None:
                continue
            if isinstance(child, (list, tuple)):
                for i in child:
                    if i == None:
                        continue
                    assert isinstance(i, Node), (node, node._fields, field, child, i)
                    callback(i, self)
            else:
                assert isinstance(child, Node), (node, node._fields, field, child)
                callback(child, self)
        return node

    def visit_Node(self, node: Node):
        return self.generic_visit(node)
