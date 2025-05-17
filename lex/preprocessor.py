from contextlib import contextmanager
import os
import datetime
from typing import Optional
from basic import (
    Error,
    Location,
    Token,
    TokenKind,
    FileReader,
    FlagManager,
    Warn,
    Symtab,
)
from cast import (
    DumpVisitor,
    LineDirecvtive,
    Embed,
    ErrorDirecvtive,
    PPDirective,
    UndefDirective,
    DefineDirective,
    Pragma,
    Include,
    WarningDirecvtive,
    IfSection,
)
from lex.embed_lexer import EmbedLexer
from lex.macro import Macro, MacroArg
from lex.ppflag import PPFlag

from .gen_lexer import Gen_Lexer


class Preprocessor(Gen_Lexer):
    include_path: list[str] = []

    @staticmethod
    def find_include_file(filename: str, search_current_path: bool, current_path="."):
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
        self.flag = FlagManager(PPFlag.ALLOW_REPLACE | PPFlag.ALLOW_CONTACT)
        self.filename = self.reader.filename  # 用于 __FILE__ 替换
        self.line_shift = 0  # 用于 __LINE__ 替换时进行调整
        self.symtab = Symtab()  # 用于存放宏

    @contextmanager
    def set_flag(self, add_flags: int = 0, remove_flags: int = 0):
        _flag = self.flag.save()
        self.flag.add(add_flags)
        self.flag.remove(remove_flags)
        yield
        self.flag.restore(_flag)

    def check_pphash(self):
        """判断当前读到的'#'是否算是预处理器指令"""
        i = self.reader.nextindex - 2
        atbegineofline = True  # 位于行首
        while i >= 0:
            if self.reader.hasread[i][0] == "\n":
                break
            if not self.reader.hasread[i][0].isspace():
                atbegineofline = False
                break
            i -= 1
        return atbegineofline

    def get_new_token(self) -> Token:
        ch, location = self.reader.next()
        if ch == "/":
            ch, loc = self.reader.next()
            location.extend(loc)
            if ch == "/":
                text = ""
                ch, loc = self.reader.next()
                while ch and ch != "\n":
                    text += ch
                    location.extend(loc)
                    ch, loc = self.reader.next()
                return Token(TokenKind.COMMENT, location, "//" + text)
            elif ch == "*":
                text = ""
                ch, loc = self.reader.next()
                while ch and text[-2:] != "*/":
                    text += ch
                    location.extend(loc)
                    ch, loc = self.reader.next()
                self.reader.back()
                return Token(TokenKind.COMMENT, location, "/*" + text)
            else:
                # 多读了两个
                self.reader.back()
                self.reader.back()
        elif ch == "<" and self.flag.has(PPFlag.ALLOW_HEADERNAME):
            text = ""
            ch, loc = self.reader.next()
            while ch and ch != ">":
                text += ch
                location.extend(loc)
                ch, loc = self.reader.next()
            return Token(TokenKind.HEADERNAME, location, text)
        elif ch == "\n" and self.flag.has(PPFlag.KEEP_NEWLINE):
            return Token(TokenKind.NEWLINE, location, "\n")
        else:
            self.reader.back()
        token = super().get_new_token()
        if token != None:
            if token.kind == TokenKind.HASH:
                token.ispphash = self.check_pphash()
            elif token.kind == TokenKind.L_PAREN:
                i = self.reader.nextindex - 2
                token.islparen = i >= 0 and (
                    self.identifier_start(self.reader.hasread[i][0])
                    or self.identifier_continue(self.reader.hasread[i][0])
                )
        return token

    def next(self):
        while True:
            token = super().next()

            if (
                not self.flag.has(PPFlag.IGNORE_PPDIRECTIVE)
                and token.kind == TokenKind.UNHANDLE_PPDIRECTIVE
            ):
                self.nexttk_index -= 1
                self.tokens.pop(self.nexttk_index)
                self.handleDirective(token.pp_directive)
                continue
            elif (
                not self.flag.has(PPFlag.IGNORE_PPDIRECTIVE)
                and token.kind == TokenKind.HASH
                and token.ispphash
            ):
                from lex.pp_directive_parser import PPDirectiveParser

                start = self.nexttk_index - 1
                with self.set_flag(
                    PPFlag.KEEP_NEWLINE
                    | PPFlag.IGNORE_PPDIRECTIVE
                    | PPFlag.TRANS_PPKEYWORD,
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
                self.handleDirective(pp_directive)
                continue
            # 尝试进行宏替换
            elif (
                self.flag.has(PPFlag.ALLOW_REPLACE)
                and token.kind == TokenKind.IDENTIFIER
                and self.replaceMacro()  # 进行了替换
            ):
                continue
            elif (
                not self.flag.has(PPFlag.KEEP_NEWLINE)
                and token.kind == TokenKind.NEWLINE
            ):
                self.nexttk_index -= 1
                self.tokens.pop(self.nexttk_index)
                continue
            elif (
                not self.flag.has(PPFlag.KEEP_COMMENT)
                and token.kind == TokenKind.COMMENT
            ):
                self.nexttk_index -= 1
                self.tokens.pop(self.nexttk_index)
                continue
            elif (
                self.flag.has(PPFlag.TRANS_PPKEYWORD)
                and token.kind == TokenKind.IDENTIFIER
                and token.text in Token.ppkeywords
            ):
                token.kind = Token.ppkeywords[token.text]
            elif (
                not self.flag.has(PPFlag.TRANS_PPKEYWORD)
                and token.kind in Token.ppkeywords.values()
            ):
                token.kind = TokenKind.IDENTIFIER

            # 连接相邻的字符串字面量
            while (
                self.flag.has(PPFlag.ALLOW_CONTACT)
                and token.kind == TokenKind.STRINGLITERAL
            ):
                t = self.save()
                token2: Token = super().next()
                if token2.kind == TokenKind.STRINGLITERAL:
                    if (
                        token.prefix != "" and token2.prefix != ""
                    ) and token2.prefix != token.prefix:
                        raise Error("无法连接字符串", token2.location)
                    token.text += " " + token2.text
                    token.content += token2.content
                    token.location.extend(token2.location)
                    token.prefix = token.prefix if token.prefix != "" else token2.prefix
                    self.nexttk_index -= 1
                    self.tokens.pop(self.nexttk_index)
                else:
                    self.restore(t)
                    break
            return token

    def handleDirective(self, pp_directive: PPDirective):
        if isinstance(pp_directive, DefineDirective):
            name = pp_directive.name
            macro = Macro(
                name,
                pp_directive.parameters,
                pp_directive.replacement,
                pp_directive.is_object_like,
                pp_directive.hasvarparam,
            )
            if not self.symtab.addSymbol(name, macro):
                old_symbol = self.symtab.lookup(name)
                if old_symbol != macro:
                    raise Error(f"重定义宏: {name}", pp_directive.location)
                macro = old_symbol
            macro.define_location = pp_directive.location
            macro.declare_locations.append(pp_directive.location)
        elif isinstance(pp_directive, UndefDirective):
            self.symtab.removeSymbol(pp_directive.name)
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
            token = Token(
                TokenKind.SUB_TOKENGEN,
                pp_directive.location,
                f"<{pp_directive.__class__.__name__}>",
            )
            token.tokengen = pp_directive
            self.tokens.insert(self.nexttk_index, token)
        elif isinstance(pp_directive, Include):
            filepath = self.find_include_file(
                pp_directive.filename,
                pp_directive.search_current_path,
                os.path.dirname(self.reader.filename),
            )
            if filepath == None:
                raise Error(
                    f"无法包含文件: {pp_directive.filename}", pp_directive.location
                )
            if isinstance(pp_directive, Embed):
                args = pp_directive.analyze_parameters()
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
                pp.symtab.ordinary_names = self.symtab.ordinary_names
            token = Token(
                TokenKind.SUB_TOKENGEN,
                pp_directive.location,
                f"<{pp_directive.__class__.__name__}>",
            )
            token.tokengen = pp
            self.tokens.insert(self.nexttk_index, token)
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
        elif self.symtab.lookup(name) == None:
            return False

        start = self.nexttk_index - 1  # 替换开始的位置
        macro: Macro = self.symtab.lookup(name)
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
        with self.set_flag(remove_flags=PPFlag.ALLOW_REPLACE):
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
                with self.set_flag(PPFlag.ALLOW_REPLACE):
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
