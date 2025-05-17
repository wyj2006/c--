import colorama

from colorama import Fore
from cast import Node, UnaryOperator, BinaryOperator

from .visitor import Visitor

colorama.init(autoreset=True)


class DumpVisitor(Visitor):
    """输出语法树"""

    color_cycle = (
        Fore.GREEN,
        Fore.YELLOW,
        Fore.BLUE,
        Fore.MAGENTA,
        Fore.CYAN,
        Fore.WHITE,
    )

    def visit_Node(self, node: Node, indent=0):
        color_i = 0
        print(
            " " * 2 * indent + self.color_cycle[color_i] + node.__class__.__name__,
            end=" ",
        )
        color_i += 1
        for i in node._attributes:
            if isinstance(node, (UnaryOperator, BinaryOperator)) and i == "op":
                attr = node.op.value
            elif hasattr(node, i):
                attr = getattr(node, i)
            else:
                attr = None
            if attr != None:
                print(self.color_cycle[color_i] + str(attr), end=" ")
                color_i += 1
            else:
                print(end="")
        print()
        self.generic_visit(node, lambda node, _: node.accept(self, indent + 1))
