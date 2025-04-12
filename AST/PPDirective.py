from typing import TYPE_CHECKING, Union
from AST.Node import Node, Attribute
from Basic import Diagnostic, Error, Symtab, Token, TokenGen, TokenKind
from AST.Expr import Expr

if TYPE_CHECKING:
    from Lex import Macro


class PPDirective(Node):
    """预处理指令"""


class EmptyDirective(PPDirective):
    pass


class Include(PPDirective):
    filename: str
    search_current_path: bool

    _attributes = PPDirective._attributes + ("filename", "search_current_path")


class Embed(Include):
    parameters: list["PPParameter"]

    def analyzeParameters(self):
        """解析参数"""
        from Lex import Evaluater

        limit = None
        prefix = None
        suffix = None
        if_empty = None
        for param in self.parameters:
            if isinstance(param, LimitParam):
                if limit != None:
                    raise Error("limit只能出现一次", param.location)
                param.expr.accept(Evaluater())
                limit = param.expr.value
            elif isinstance(param, PrefixParam):
                if prefix != None:
                    raise Error("prefix只能出现一次", param.location)
                prefix = param.args
            elif isinstance(param, SuffixParam):
                if suffix != None:
                    raise Error("suffix只能出现一次", param.location)
                suffix = param.args
            elif isinstance(param, IfEmptyParam):
                if if_empty != None:
                    raise Error("if_empty只能出现一次", param.location)
                if_empty = param.args
            else:
                raise Error("未知的参数", param.location)
        return {
            "limit": limit if limit != None else float("inf"),
            "prefix": prefix,
            "suffix": suffix,
            "if_empty": if_empty,
        }

    _fields = Include._fields + ("parameters",)


class DefineDirective(PPDirective):
    name: str
    parameters: list[str]
    hasvarparam: bool
    is_object_like: bool
    replacement: list[Token]

    _attributes = PPDirective._attributes + (
        "name",
        "parameters",
        "hasvarparam",
        "is_object_like",
        "replacement",
    )


class UndefDirective(PPDirective):
    name: str

    _attributes = PPDirective._attributes + ("name",)


class LineDirecvtive(PPDirective):
    lineno: int
    filename: str

    _attributes = PPDirective._attributes + ("lineno", "filename")


class ErrorDirecvtive(PPDirective):
    messages: list[Token]

    _attributes = PPDirective._attributes + ("messages",)


class WarningDirecvtive(PPDirective):
    messages: list[Token]

    _attributes = PPDirective._attributes + ("messages",)


class Pragma(PPDirective):
    args: list[Token]

    _attributes = PPDirective._attributes + ("args",)


class Group(PPDirective, TokenGen):
    parts: list[Union[Token, "IfSection"]]

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self.nexttk_index = 0
        self.sub_tokengen: list[TokenGen] = []

    def next(self) -> Token:
        # 只负责提供token
        while self.sub_tokengen:
            token = self.sub_tokengen[0].next()
            if token.kind == TokenKind.END:
                self.sub_tokengen.pop(0)
            return token

        if self.nexttk_index >= len(self.parts):
            return Token(
                TokenKind.END,
                self.parts[-1].location if self.parts else self.location,
                "",
            )
        token = self.parts[self.nexttk_index]
        self.nexttk_index += 1
        if isinstance(token, IfSection):
            self.sub_tokengen.append(token)
            return self.next()
        elif isinstance(token, PPDirective):
            pp_directive = token
            token = Token(
                TokenKind.UNHANDLE_PPDIRECTIVE,
                token.location,
                f"<{token.__class__.__name__}>",
            )
            token.pp_directive = pp_directive
        return token

    def back(self):
        if self.sub_tokengen:
            self.sub_tokengen[0].back()
            return
        assert self.nexttk_index > 0
        self.nexttk_index -= 1

    def save(self):
        if self.sub_tokengen:
            return self.sub_tokengen[0].save()
        return self.nexttk_index

    def restore(self, index):
        if self.sub_tokengen:
            self.sub_tokengen[0].restore(index)
            return
        self.nexttk_index = index

    @property
    def pp_directive_parts(self):
        return [i for i in self.parts if isinstance(i, PPDirective)]

    _attributes = PPDirective._attributes + ("parts",)
    _fields = PPDirective._fields + ("pp_directive_parts",)


class IfSection(PPDirective, TokenGen):
    group: Group
    else_group: "IfSection"
    macros: dict[str, "Macro"]
    symtab: Symtab  # 所处上下文对应的符号表

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self.sub_tokengen: TokenGen = None

    def check(self):
        """检查是否符合条件"""
        return True

    def next(self) -> Token:
        if self.sub_tokengen == None:
            if self.check():
                self.sub_tokengen = self.group
            elif hasattr(self, "else_group"):
                self.sub_tokengen = self.else_group
            else:
                return Token(TokenKind.END, self.location, "")
        return self.sub_tokengen.next()

    def back(self):
        self.sub_tokengen.back()

    def save(self):
        return self.sub_tokengen.save()

    def restore(self, index):
        self.sub_tokengen.restore(index)

    _fields = PPDirective._fields + ("group", "else_group")


class IfDirecvtive(IfSection):
    """#if"""

    expr: Expr

    def check(self):
        from Lex import Evaluater

        if self.expr == None:
            return False

        self.expr.accept(Evaluater(self.symtab))

        return self.expr.value

    _fields = IfSection._fields + ("expr",)


class ElifDirecvtive(IfDirecvtive):
    """#elif"""

    diagnostic: Diagnostic  # 可能的报错信息

    def check(self):
        if hasattr(self, "diagnostic"):
            raise self.diagnostic
        return super().check()


class IfdefDirecvtive(IfSection):
    """#ifdef"""

    name: str

    def check(self):
        return self.symtab.lookup(self.name) != None

    _attributes = IfSection._attributes + ("name",)


class IfndefDirecvtive(IfSection):
    """#ifndef"""

    name: str

    def check(self):
        return self.symtab.lookup(self.name) == None

    _attributes = IfSection._attributes + ("name",)


class ElseDirecvtive(IfSection):
    """#else"""

    def check(self):
        return True


class EndifDirecvtive(PPDirective):
    """#endif"""


class PPParameter(PPDirective):
    prefix: str
    name: str
    args: list[Token]

    _attributes = PPDirective._attributes + ("prefix", "name")


class LimitParam(PPParameter):
    prefix = ""
    name = "limit"
    args = []
    expr: Expr

    _fields = PPParameter._fields + ("expr",)


class SuffixParam(PPParameter):
    pass


class PrefixParam(PPParameter):
    pass


class IfEmptyParam(PPParameter):
    pass


class Defined(Expr):
    name: str

    _attributes = Expr._attributes + ("name",)


class HasInclude(Expr):
    filename: str
    search_current_path: bool

    _attributes = Expr._attributes + ("filename", "search_current_path")


class HasEmbed(HasInclude):
    parameters: list["PPParameter"]

    _attributes = HasInclude._attributes + ("filename",)
    _fields = HasInclude._fields + ("parameters",)


class HasCAttribute(Expr):
    attribute: Attribute

    _fields = Expr._fields + ("attribute",)
