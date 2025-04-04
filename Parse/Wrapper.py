"""
各种各样的装饰器
"""

from typing import TYPE_CHECKING, Callable, TypeVar
from AST import Declaration, StorageClass, StorageClassSpecifier, NameDeclarator

if TYPE_CHECKING:
    from Parse.Parser import Parser

_T = TypeVar("_T")


def may_update_type_symbol(parser_method: Callable[["Parser"], _T]):
    """该Parser方法的返回值可能能够用来更新Parser.type_symbol"""

    def wrapper(self: "Parser", *args, **kwargs) -> _T:
        node = parser_method(self, *args, **kwargs)
        if not isinstance(node, Declaration):
            return node
        for i in node.specifiers:
            if (
                isinstance(i, StorageClass)
                and i.specifier == StorageClassSpecifier.TYPEDEF
            ):
                break
        else:
            return node
        for i in node.declarators:
            a = i
            while a != None:
                if isinstance(a, NameDeclarator):
                    self.type_symbol.append(a.name)
                    break
                a = a.declarator
        return node

    return wrapper


def may_enter_scope(parser_method: Callable[["Parser"], _T]):
    """该Parser方法对应的语法结构可能会进入一个新的作用域"""

    def wrapper(self: "Parser", *args, **kwargs) -> _T:
        type_symbol = tuple(self.type_symbol)
        ret = parser_method(self, *args, **kwargs)
        self.type_symbol = list(type_symbol)
        return ret

    return wrapper
