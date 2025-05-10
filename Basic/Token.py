from enum import Enum

from Basic.Location import Location


class TokenKind(Enum):
    ALIGNAS = "alignas"
    ALIGNOF = "alignof"
    AUTO = "auto"
    BOOL = "bool"
    BREAK = "break"
    CASE = "case"
    CHAR = "char"
    CONST = "const"
    CONSTEXPR = "constexpr"
    CONTINUE = "continue"
    DEFAULT = "default"
    DO = "do"
    DOUBLE = "double"
    ELSE = "else"
    ENUM = "enum"
    EXTERN = "extern"
    FALSE = "false"
    FLOAT = "float"
    FOR = "for"
    GOTO = "goto"
    IF = "if"
    INLINE = "inline"
    INT = "int"
    LONG = "long"
    NULLPTR = "nullptr"
    REGISTER = "register"
    RESTRICT = "restrict"
    RETURN = "return"
    SHORT = "short"
    SIGNED = "signed"
    SIZEOF = "sizeof"
    STATIC = "static"
    STATIC_ASSERT = "static_assert"
    STRUCT = "struct"
    SWITCH = "switch"
    THREAD_LOCAL = "thread_local"
    TRUE = "true"
    TYPEDEF = "typedef"
    TYPEOF = "typeof"
    TYPEOF_UNQUAL = "typeof_unqual"
    UNION = "union"
    UNSIGNED = "unsigned"
    VOID = "void"
    VOLATILE = "volatile"
    WHILE = "while"
    _ATOMIC = "_Atomic"
    _BITINT = "_BitInt"
    _COMPLEX = "_Complex"
    _DECIMAL128 = "_Decimal128"
    _DECIMAL32 = "_Decimal32"
    _DECIMAL64 = "_Decimal64"
    _GENERIC = "_Generic"
    _IMAGINARY = "_Imaginary"
    _NORETURN = "_Noreturn"

    IDENTIFIER = "标识符"
    INTCONST = "整数常量"
    FLOATCONST = "浮点数常量"
    CHARCONST = "字符常量"
    STRINGLITERAL = "字符串字面量"

    L_SQUARE = "["
    R_SQUARE = "]"
    L_PAREN = "("
    R_PAREN = ")"
    L_BRACE = "{"
    R_BRACE = "}"
    PERIOD = "."
    ELLIPSIS = "..."
    AMP = "&"
    AMPAMP = "&&"
    AMPEQUAL = "&="
    STAR = "*"
    STAREQUAL = "*="
    PLUS = "+"
    PLUSPLUS = "++"
    PLUSEQUAL = "+="
    MINUS = "-"
    ARROW = "->"
    MINUSMINUS = "--"
    MINUSEQUAL = "-="
    TILDE = "~"
    EXCLAIM = "!"
    EXCLAIMEQUAL = "!="
    SLASH = "/"
    SLASHEQUAL = "/="
    PERCENT = "%"
    PERCENTEQUAL = "%="
    LESS = "<"
    LESSLESS = "<<"
    LESSEQUAL = "<="
    LESSLESSEQUAL = "<<="
    GREATER = ">"
    GREATERGREATER = ">>"
    GREATEREQUAL = ">="
    GREATERGREATEREQUAL = ">>="
    CARET = "^"
    CARETEQUAL = "^="
    PIPE = "|"
    PIPEPIPE = "||"
    PIPEEQUAL = "|="
    QUESTION = "?"
    COLON = ":"
    COLONCOLON = "::"
    SEMI = ";"
    EQUAL = "="
    EQUALEQUAL = "=="
    COMMA = ","
    HASH = "#"
    HASHHASH = "##"

    END = "文件结尾"  # 文件结尾
    UNKOWN = "未知"  # 未知

    # 预处理
    COMMENT = "注释"
    NEWLINE = "换行"
    DEFINE = "define"
    VA_ARGS = "__VA_ARGS__"
    VA_OPT = "__VA_OPT__"
    UNDEF = "undef"
    IFDEF = "ifdef"
    IFNDEF = "ifndef"
    ELIF = "elif"
    ELIFDEF = "elifdef"
    ELIFNDEF = "elifndef"
    ENDIF = "endif"
    INCLUDE = "include"
    HEADERNAME = "头文件名"
    LINE = "line"
    ERROR = "error"
    WARNING = "warning"
    PRAGMA = "pragma"
    EMBED = "embed"
    DEFINED = "defined"
    HAS_INCLUDE = "__has_include"
    HAS_EMBED = "__has_embed"
    HAS_C_ATTRIBUTE = "__has_c_attribute"

    # 语法分析器生成工具
    ACTION = "语义动作"
    HEADER = "头部代码"

    # 不是严格意义上的token
    SUB_TOKENGEN = "Token生成器"
    UNHANDLE_PPDIRECTIVE = "未处理的预处理指令"


class Token:
    keywords = {
        "alignas": TokenKind.ALIGNAS,
        "_Alignas": TokenKind.ALIGNAS,
        "alignof": TokenKind.ALIGNOF,
        "_Alignof": TokenKind.ALIGNOF,
        "auto": TokenKind.AUTO,
        "bool": TokenKind.BOOL,
        "_Bool": TokenKind.BOOL,
        "break": TokenKind.BREAK,
        "case": TokenKind.CASE,
        "char": TokenKind.CHAR,
        "const": TokenKind.CONST,
        "constexpr": TokenKind.CONSTEXPR,
        "continue": TokenKind.CONTINUE,
        "default": TokenKind.DEFAULT,
        "do": TokenKind.DO,
        "double": TokenKind.DOUBLE,
        "else": TokenKind.ELSE,
        "enum": TokenKind.ENUM,
        "extern": TokenKind.EXTERN,
        "false": TokenKind.FALSE,
        "float": TokenKind.FLOAT,
        "for": TokenKind.FOR,
        "goto": TokenKind.GOTO,
        "if": TokenKind.IF,
        "inline": TokenKind.INLINE,
        "int": TokenKind.INT,
        "long": TokenKind.LONG,
        "nullptr": TokenKind.NULLPTR,
        "register": TokenKind.REGISTER,
        "restrict": TokenKind.RESTRICT,
        "return": TokenKind.RETURN,
        "short": TokenKind.SHORT,
        "signed": TokenKind.SIGNED,
        "sizeof": TokenKind.SIZEOF,
        "static": TokenKind.STATIC,
        "static_assert": TokenKind.STATIC_ASSERT,
        "_Static_assert": TokenKind.STATIC_ASSERT,
        "struct": TokenKind.STRUCT,
        "switch": TokenKind.SWITCH,
        "thread_local": TokenKind.THREAD_LOCAL,
        "_Thread_local": TokenKind.THREAD_LOCAL,
        "true": TokenKind.TRUE,
        "typedef": TokenKind.TYPEDEF,
        "typeof": TokenKind.TYPEOF,
        "typeof_unqual": TokenKind.TYPEOF_UNQUAL,
        "union": TokenKind.UNION,
        "unsigned": TokenKind.UNSIGNED,
        "void": TokenKind.VOID,
        "volatile": TokenKind.VOLATILE,
        "while": TokenKind.WHILE,
        "_Atomic": TokenKind._ATOMIC,
        "_BitInt": TokenKind._BITINT,
        "_Complex": TokenKind._COMPLEX,
        "_Decimal128": TokenKind._DECIMAL128,
        "_Decimal32": TokenKind._DECIMAL32,
        "_Decimal64": TokenKind._DECIMAL64,
        "_Generic": TokenKind._GENERIC,
        "_Imaginary": TokenKind._IMAGINARY,
        "_Noreturn": TokenKind._NORETURN,
    }
    ppkeywords = {
        "define": TokenKind.DEFINE,
        "__VA_ARGS__": TokenKind.VA_ARGS,
        "__VA_OPT__": TokenKind.VA_OPT,
        "undef": TokenKind.UNDEF,
        "ifdef": TokenKind.IFDEF,
        "ifndef": TokenKind.IFNDEF,
        "elif": TokenKind.ELIF,
        "elifdef": TokenKind.ELIFDEF,
        "elifndef": TokenKind.ELIFNDEF,
        "endif": TokenKind.ENDIF,
        "include": TokenKind.INCLUDE,
        "line": TokenKind.LINE,
        "error": TokenKind.ERROR,
        "warning": TokenKind.WARNING,
        "pragma": TokenKind.PRAGMA,
        "embed": TokenKind.EMBED,
        "defined": TokenKind.DEFINED,
        "__has_include": TokenKind.HAS_INCLUDE,
        "__has_embed": TokenKind.HAS_EMBED,
        "__has_c_attribute": TokenKind.HAS_C_ATTRIBUTE,
    }
    punctuator = {
        "[": TokenKind.L_SQUARE,
        "]": TokenKind.R_SQUARE,
        "(": TokenKind.L_PAREN,
        ")": TokenKind.R_PAREN,
        "{": TokenKind.L_BRACE,
        "}": TokenKind.R_BRACE,
        ".": TokenKind.PERIOD,
        "...": TokenKind.ELLIPSIS,
        "&": TokenKind.AMP,
        "&&": TokenKind.AMPAMP,
        "&=": TokenKind.AMPEQUAL,
        "*": TokenKind.STAR,
        "*=": TokenKind.STAREQUAL,
        "+": TokenKind.PLUS,
        "++": TokenKind.PLUSPLUS,
        "+=": TokenKind.PLUSEQUAL,
        "-": TokenKind.MINUS,
        "->": TokenKind.ARROW,
        "--": TokenKind.MINUSMINUS,
        "-=": TokenKind.MINUSEQUAL,
        "~": TokenKind.TILDE,
        "!": TokenKind.EXCLAIM,
        "!=": TokenKind.EXCLAIMEQUAL,
        "/": TokenKind.SLASH,
        "/=": TokenKind.SLASHEQUAL,
        "%": TokenKind.PERCENT,
        "%=": TokenKind.PERCENTEQUAL,
        "<": TokenKind.LESS,
        "<<": TokenKind.LESSLESS,
        "<=": TokenKind.LESSEQUAL,
        "<<=": TokenKind.LESSLESSEQUAL,
        ">": TokenKind.GREATER,
        ">>": TokenKind.GREATERGREATER,
        ">=": TokenKind.GREATEREQUAL,
        ">>=": TokenKind.GREATERGREATEREQUAL,
        "^": TokenKind.CARET,
        "^=": TokenKind.CARETEQUAL,
        "|": TokenKind.PIPE,
        "||": TokenKind.PIPEPIPE,
        "|=": TokenKind.PIPEEQUAL,
        "?": TokenKind.QUESTION,
        ":": TokenKind.COLON,
        "::": TokenKind.COLONCOLON,
        ";": TokenKind.SEMI,
        "=": TokenKind.EQUAL,
        "==": TokenKind.EQUALEQUAL,
        ",": TokenKind.COMMA,
        "#": TokenKind.HASH,
        "##": TokenKind.HASHHASH,
    }

    def __init__(self, kind: TokenKind, location: Location, text: str):
        self.kind = kind
        self.location = location
        self.text = text
        self.content = None  #  内容
        self.prefix = None  # 前缀
        self.suffix = None  # 后缀
        self.ispphash = False  # 是否是预处理指令开头的'#'
        self.islparen = False  # define预处理指令中跟在宏名后面的括号
        self.pp_directive = None  # 未处理的预处理指令
        self.tokengen = None  # Token生成器

        match kind:
            case TokenKind.CHARCONST | TokenKind.STRINGLITERAL:
                i = self.text.find('"' if kind == TokenKind.STRINGLITERAL else "'")
                self.prefix = self.text[:i]
                self.content = self.text[i:]
                self.content = eval(self.content)
            case TokenKind.COMMENT:
                if text.startswith("//"):
                    self.content = text[2:]
                elif text.startswith("/*"):
                    self.content = text[2:-2]
                else:
                    assert False, "未知的注释结构"
            case TokenKind.HEADER:
                self.content = text[3:-3]
            case TokenKind.ACTION:
                self.content = text[1:-1]
            case TokenKind.INTCONST | TokenKind.FLOATCONST:
                self.content = self.text.replace("'", "")
                self.prefix = ""
                self.suffix = []
                for prefix in ("0x", "0X", "0B", "0b"):
                    if self.content.startswith(prefix):
                        self.prefix = prefix.lower()
                        self.content = self.content[len(prefix) :]
                        break
                else:
                    if self.content[0] == "0" and len(self.content) > 1:
                        self.prefix = "0"
                        self.content = self.content[1:]
                while True:
                    for suffix in (
                        "wb",
                        "WB",
                        "ll",
                        "LL",
                        "l",
                        "L",
                        "U",
                        "u",
                        "df",
                        "dd",
                        "dl",
                        "DF",
                        "DD",
                        "DL",
                        "f",
                        "F",
                        "l",
                        "L",
                    ):
                        if self.content.endswith(suffix):
                            self.suffix.append(suffix.lower())
                            self.content = self.content[: -len(suffix)]
                            break
                    else:
                        break

    def __repr__(self):
        return f"Token({self.kind.name},{self.location},{repr(self.text)})"


class TokenGen:
    """token生成器基类"""

    def curtoken(self) -> Token:
        """获取当前token"""

    def next(self) -> Token:
        """获取下一个token"""

    def back(self):
        """回退当前token"""

    def save(self):
        """保存当前状态, 并返回一个可以用于恢复的值"""

    def restore(self):
        """接受用于恢复的值并恢复状态"""
