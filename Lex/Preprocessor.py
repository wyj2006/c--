from contextlib import contextmanager
import os
import datetime
from typing import Optional
from Basic import (
    Error,
    Location,
    Token,
    TokenKind,
    FileReader,
    FlagManager,
    Warn,
    TokenGen,
)
from AST import (
    DumpVisitor,
    LineDirecvtive,
    Embed,
    ErrorDirecvtive,
    UndefDirective,
    DefineDirective,
    Pragma,
    Include,
    WarningDirecvtive,
    IfSection,
)
from Lex.EmbedLexer import EmbedLexer
from Lex.Lexer import Lexer, LexerParser
from Lex.Macro import Macro, MacroArg
from Lex.PPFlag import PPFlag
from Lex.gen_PPLexerParser import Gen_PPLexerParser
from Lex.ConcatReader import ConcatReader
from Parse.Builder import memorize


class PPLexerParser(Gen_PPLexerParser, LexerParser):
    @memorize
    def h_char(self):
        """
        h-char:
            any member of the source character set except
                the new-line character and >
        """
        token = self.curtoken()
        if token.text not in "\n><":
            self.nexttoken()
            return token
        return None

    @memorize
    def single_line_comment(self):
        a = [Location([]), ""]
        while self.curtoken().kind != TokenKind.END:
            token = self.curtoken()
            ch = token.text
            token = self.curtoken()
            if ch == "\n":
                break
            a[0] += token.location
            a[1] += token.text
            self.nexttoken()
        return tuple(a)

    @memorize
    def multi_line_comment(self):
        a = [Location([]), ""]
        while self.curtoken().kind != TokenKind.END:
            _z = self.save()

            token = self.curtoken()
            ch = token.text
            if ch == "*":
                self.nexttoken()
                token = self.curtoken()
                ch = token.text
                if ch == "/":
                    self.restore(_z)
                    break
            self.restore(_z)

            token = self.curtoken()
            a[0] += token.location
            a[1] += token.text
            self.nexttoken()
        return tuple(a)


class Preprocessor(Lexer):
    include_path: list[str] = []

    @staticmethod
    def findIncludeFile(filename: str, search_current_path: bool, current_path="."):
        """查找包含文件的路径, 找不到返回None"""
        include_path = Preprocessor.include_path.copy()
        if search_current_path:
            include_path.insert(0, current_path)
        for path in include_path:
            filepath = os.path.join(path, filename)
            if not os.path.exists(filepath):
                continue
            return filepath
        return None

    def __init__(self, reader):
        super().__init__(reader)
        self.lexerparser = PPLexerParser(self.charlexer)
        self.flag = FlagManager(PPFlag.ALLOW_REPLACE | PPFlag.ALLOW_CONTACT)
        self.macros: dict[str, Macro] = {}
        self.filename = self.reader.filename  # 用于 __FILE__ 替换
        self.line_shift = 0  # 用于 __LINE__ 替换时进行调整

    @contextmanager
    def setFlag(self, add_flags: int = 0, remove_flags: int = 0):
        _flag = self.flag.save()
        self.flag.add(add_flags)
        self.flag.remove(remove_flags)
        yield
        self.flag.restore(_flag)

    def next(self):
        token = super().next()

        if (
            not self.flag.has(PPFlag.IGNORE_PPDIRECTIVE)
            and token.kind == TokenKind.HASH
            and token.ispphash
        ):
            self.handleDirective()
            return self.next()

        # 尝试进行宏替换
        if self.flag.has(PPFlag.ALLOW_REPLACE) and token.kind == TokenKind.IDENTIFIER:
            if self.replaceMacro():  # 进行了替换
                return self.next()

        if not self.flag.has(PPFlag.KEEP_NEWLINE) and token.kind == TokenKind.NEWLINE:
            self.nexttk_index -= 1
            self.tokens.pop(self.nexttk_index)
            return self.next()
        elif not self.flag.has(PPFlag.KEEP_COMMENT) and token.kind == TokenKind.COMMENT:
            self.nexttk_index -= 1
            self.tokens.pop(self.nexttk_index)
            return self.next()
        elif self.flag.has(PPFlag.TRANS_PPKEYWORD) and token.text in Token.ppkeywords:
            token.kind = Token.ppkeywords[token.text]
        elif (
            not self.flag.has(PPFlag.ALLOW_HEADERNAME)
            and token.kind == TokenKind.HEADERNAME
        ):
            self.nexttk_index -= 1
            self.tokens.pop(self.nexttk_index)
            reader = ConcatReader([token])
            lexer = Lexer(reader)
            self.tokens.insert(self.nexttk_index, lexer)
            return self.next()

        # 连接相邻的字符串字面量
        while (
            self.flag.has(PPFlag.ALLOW_CONTACT)
            and token.kind == TokenKind.STRINGLITERAL
        ):
            t = self.save()
            token2 = self.next()
            if token2.kind == TokenKind.STRINGLITERAL:
                token.text += " " + token2.text
                token.content += token2.content
                token.location.extend(token2.location)
                prefix_index = ["", "u8", "L", "u", "U"]
                token.prefix = prefix_index[
                    max(
                        prefix_index.index(token.prefix),
                        prefix_index.index(token2.prefix),
                    )
                ]
                self.nexttk_index -= 1
                self.tokens.pop(self.nexttk_index)
            else:
                self.restore(t)
                break
        return token

    def handleDirective(self):
        from Lex.PPDirectiveParser import PPDirectiveParser

        start = self.nexttk_index - 1
        with self.setFlag(
            PPFlag.KEEP_NEWLINE | PPFlag.IGNORE_PPDIRECTIVE | PPFlag.TRANS_PPKEYWORD,
            PPFlag.ALLOW_CONTACT | PPFlag.ALLOW_REPLACE,
        ):
            parser = PPDirectiveParser(self)
            pp_directive = parser.start()
        if isinstance(pp_directive, list):
            # pp_directive[0]是'#'
            raise Error("未知的预处理指令", pp_directive[1].location)
        end = self.nexttk_index - 1
        self.tokens[start:end] = []
        self.nexttk_index = start
        # pp_directive.accept(DumpVisitor())
        if isinstance(pp_directive, DefineDirective):
            name = pp_directive.name
            macro = Macro(
                name,
                pp_directive.parameters,
                pp_directive.replacement,
                pp_directive.is_object_like,
                pp_directive.hasvarparam,
            )
            if name not in self.macros:
                self.macros[name] = macro
            elif self.macros[name] != macro:
                raise Error(f"重定义宏: {name}", pp_directive.location)
        elif isinstance(pp_directive, UndefDirective):
            name = pp_directive.name
            if name in self.macros:
                self.macros.pop(name)
        elif isinstance(pp_directive, (ErrorDirecvtive, WarningDirecvtive)):
            messages = []
            for message in pp_directive.messages:
                if message.kind in (TokenKind.STRINGLITERAL, TokenKind.CHARCONST):
                    messages.append(message.content)
                else:
                    messages.append(message.text)
            message = " ".join(messages)
            if isinstance(pp_directive, ErrorDirecvtive):
                raise Error(message, pp_directive.location)
            elif isinstance(pp_directive, WarningDirecvtive):
                Warn(message, pp_directive.location).dump()
        elif isinstance(pp_directive, LineDirecvtive):
            lineno = pp_directive.lineno
            if lineno < 0:
                raise Error("期望一个非负整数", pp_directive.location)
            self.line_shift = lineno - pp_directive.location[0]["lineno"]
            if hasattr(pp_directive, "filename"):
                self.filename = pp_directive.filename
        elif isinstance(pp_directive, IfSection):
            self.tokens.insert(self.nexttk_index, pp_directive)
        elif isinstance(pp_directive, Include):
            filepath = self.findIncludeFile(
                pp_directive.filename,
                pp_directive.search_current_path,
                os.path.dirname(self.reader.filename),
            )
            if filepath == None:
                raise Error(
                    f"无法包含文件: {pp_directive.filename}", pp_directive.location
                )
            if isinstance(pp_directive, Embed):
                args = pp_directive.analyzeParameters()
                reader = FileReader(filepath, mode="rb")
                pp = EmbedLexer(
                    reader,
                    args["limit"],
                    args["prefix"],
                    args["suffix"],
                    args["if_empty"],
                )
            else:
                reader = self.reader.__class__(filepath)  # 防止这是子类
                pp = self.__class__(reader)  # 防止这是子类
                pp.macros = self.macros
            self.tokens.insert(self.nexttk_index, pp)
        elif isinstance(pp_directive, Pragma):
            # TODO: 支持pragma
            Warn("暂不支持#pragma", pp_directive.location)

    def replaceMacro(self):
        """宏替换, 如果发生了替换就返回True, 否则返回False"""
        name = self.curtoken().text
        token = None
        match (name):
            case "__DATE__":
                token = Token(
                    TokenKind.STRINGLITERAL,
                    self.curtoken().location,
                    f'"{datetime.datetime.now().strftime("%b %d %Y")}"',
                )
            case "__FILE__":
                token = Token(
                    TokenKind.STRINGLITERAL,
                    self.curtoken().location,
                    f'"{self.filename}"',
                )
            case "__LINE__":
                token = Token(
                    TokenKind.INTCONST,
                    self.curtoken().location,
                    f'{self.curtoken().location[0]["lineno"]+self.line_shift}',
                )
            case "__TIME__":
                token = Token(
                    TokenKind.STRINGLITERAL,
                    self.curtoken().location,
                    f'"{datetime.datetime.now().strftime("%H:%M:%S")}"',
                )
            case "__STDC_EMBED_NOT_FOUND__":
                token = Token(TokenKind.INTCONST, self.curtoken().location, "0")
            case "__STDC_EMBED_FOUND__":
                token = Token(TokenKind.INTCONST, self.curtoken().location, "1")
            case "__STDC_EMBED_EMPTY__":
                token = Token(TokenKind.INTCONST, self.curtoken().location, "2")
        if token != None:
            self.tokens[self.nexttk_index - 1 : self.nexttk_index] = [token]
            self.nexttk_index -= 1
            return True
        elif name not in self.macros:
            return False

        start = self.nexttk_index - 1  # 替换开始的位置
        macro = self.macros[name]
        if macro.is_object_like:
            replaced_token = macro.replace([])
        else:
            args = self.getMacroArgs(macro)
            if (
                args == None
                # 参数数量不匹配
                or (
                    not macro.hasvarparam
                    and (len(args) != len(macro.params) and len(args) != 0)
                )
                or (macro.hasvarparam and len(args) < len(macro.params) - 1)
            ):
                self.nexttk_index = start + 1
                return False
            replaced_token = macro.replace(args)
        end = self.nexttk_index  # 替换结束位置
        self.tokens[start:end] = replaced_token
        self.nexttk_index = start
        return True

    def getMacroArgs(self, macro: Macro) -> Optional[list[MacroArg]]:
        """获取宏替换的实参"""
        token = self.next()
        if token.kind != TokenKind.L_PAREN:  # 不存在实参
            return None
        with self.setFlag(remove_flags=PPFlag.ALLOW_REPLACE):
            token = self.next()
            args: list[MacroArg] = []
            while token.kind not in (TokenKind.R_PAREN, TokenKind.END):
                text = ""
                paren = 0
                invarparam = macro.hasvarparam and len(args) >= len(
                    macro.params
                )  # 当前获取的是否是变长参数
                argtk_start = self.nexttk_index - 1
                while token.kind != TokenKind.END:
                    if token.kind == TokenKind.COMMA and paren == 0 and not invarparam:
                        break
                    elif token.kind == TokenKind.L_PAREN:
                        paren += 1
                    elif token.kind == TokenKind.R_PAREN:
                        if paren == 0:
                            break
                        else:
                            paren -= 1
                    if token.text != "\n":
                        text += token.text + " "
                    token = self.next()

                # 对参数进行展开
                with self.setFlag(PPFlag.ALLOW_REPLACE):
                    lasttk = self.curtoken()
                    self.nexttk_index = argtk_start
                    token = self.next()
                    while token is not lasttk:
                        token = self.next()

                argtk_end = self.nexttk_index
                tokens = self.tokens[argtk_start : argtk_end - 1]
                arg = MacroArg(tokens, text.strip())
                args.append(arg)

                if token.kind == TokenKind.COMMA:
                    token = self.next()
        if token.kind != TokenKind.R_PAREN:
            return None
        return args
