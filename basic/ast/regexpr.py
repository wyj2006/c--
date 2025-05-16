from .node import Node


class RegExpr(Node):
    """正则表达式"""

    nullable: bool
    firstpos: set["Letter"]
    lastpos: set["Letter"]

    _attributes = Node._attributes + ("nullable",)


class Concat(RegExpr):
    """连接运算"""

    exprs: list[RegExpr]

    _fields = RegExpr._fields + ("exprs",)


class Repeat0(RegExpr):
    """重复运算"""

    repeat_expr: RegExpr

    _fields = RegExpr._fields + ("repeat_expr",)


class Choice(RegExpr):
    """选择运算"""

    exprs: list[RegExpr]

    _fields = RegExpr._fields + ("exprs",)


class Letter(RegExpr):
    """字母"""

    char: str  # 对应的值
    is_func: bool  # char对应的内容是否是函数

    action: str = None  # 行为

    _attributes = RegExpr._attributes + ("char", "is_func", "action")

    def __repr__(self):
        return f"{self.__class__.__name__}(char='{self.char}',location={self.location})"


class EmptyString(Letter):
    """空串"""

    char = ""
    is_func = False
