from basic import Location, Token, Symtab

# TODO: 简化 _fields和_attributes的设置
# TODO: 更加详尽的location信息和token信息


class Node:
    """语法树节点"""

    attribute_specifiers: list["AttributeSpecifier"] = []

    location: Location  # 语法树对应代码的位置

    _symtab: Symtab  # 如果进入语法树会进入一个新的作用域, 这个变量将会保存这个作用域对应的Symtab

    _fields: tuple[str] = ("attribute_specifiers",)  # 子节点名称
    _attributes: tuple[str] = ("location",)  # 节点属性

    def __init__(self, **kwargs):
        for key, val in kwargs.items():
            setattr(self, key, val)

    def accept(self, visitor, *args, **kwargs):
        for cls in self.__class__.__mro__:
            method = getattr(visitor, f"visit_{cls.__name__}", None)
            if method == None:
                continue
            return method(self, *args, **kwargs)


class Attribute(Node):
    prefix_name: str
    name: str
    args: list[Token]

    _attributes = Node._attributes + ("prefix_name", "name")


class DeprecatedAttr(Attribute):
    pass


class MaybeUnusedAttr(Attribute):
    pass


class NoReturnAttr(Attribute):
    pass


class UnsequencedAttr(Attribute):
    pass


class FallthroughAttr(Attribute):
    pass


class NodiscardAttr(Attribute):
    pass


class ReproducibleAttr(Attribute):
    pass


class AttributeSpecifier(Node):
    attributes: list[Attribute]

    _fields = Node._fields + ("attributes",)


class TranslationUnit(Node):
    body: list[Node]

    _fields = Node._fields + ("body",)
