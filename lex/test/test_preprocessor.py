import sys
import os

sys.path.append(os.path.dirname(os.path.dirname(os.path.dirname(__file__))))

import pytest

from basic import TokenKind, MergeReader, Error
from lex import Preprocessor, PPFlag


def examplestest(examples, handle=None):
    for example in examples:
        print(example["filename"], "=" * 64)
        reader = MergeReader(
            os.path.join(os.path.dirname(__file__), "codes", example["filename"])
        )
        lexer = Preprocessor(reader)
        if handle != None:
            lexer = handle(lexer)
        expected_exception = example.get("expected_exception", [])
        if expected_exception:
            with pytest.raises(*expected_exception):
                token = lexer.next()
                while token.kind != TokenKind.END:
                    token = lexer.next()
            continue
        else:
            token = lexer.next()
            while token.kind != TokenKind.END:
                token = lexer.next()
        i = 0
        while i < len(lexer.tokens) and i < len(example["tokens"]):
            token = lexer.tokens[i]
            print(token, example["tokens"][i])
            for attr, val in example["tokens"][i].items():
                assert getattr(token, attr) == val
            i += 1
        assert i == len(lexer.tokens) and i == len(example["tokens"])


def test_stringconcat():
    examples = [
        {
            "filename": "stringconcat.txt",
            "tokens": [
                {
                    "kind": TokenKind.STRINGLITERAL,
                    "content": '1abab1我的世界Minecraft\123"fdas\xabc\n\t\f',
                    "prefix": "U",
                },
                {"kind": TokenKind.END},
            ],
        }
    ]
    examplestest(examples)


def test_comment():
    def donot_ignore_comment(pp: Preprocessor):
        pp.flag.add(PPFlag.KEEP_COMMENT)
        return pp

    examples = [
        {
            "filename": "comment.txt",
            "tokens": [
                {"kind": TokenKind.STRINGLITERAL, "content": "a//b"},
                {"kind": TokenKind.COMMENT, "content": " */ //"},
                {"kind": TokenKind.COMMENT, "content": "i();"},
                {"kind": TokenKind.COMMENT, "content": " j();"},
                {"kind": TokenKind.COMMENT, "content": "//"},
                {"kind": TokenKind.IDENTIFIER, "text": "l"},
                {"kind": TokenKind.SLASH},
                {"kind": TokenKind.END},
            ],
        }
    ]
    examplestest(examples, donot_ignore_comment)


def test_define():
    examples = [
        {
            "filename": "define1.txt",
            "tokens": [
                {"kind": TokenKind.INTCONST, "text": "42"},
                {"kind": TokenKind.END},
            ],
        },
        {
            "filename": "define2.txt",
            "tokens": [
                {"kind": TokenKind.IDENTIFIER, "text": "f"},
                {"kind": TokenKind.L_PAREN},
                {"kind": TokenKind.INTCONST, "text": "0"},
                {"kind": TokenKind.COMMA},
                {"kind": TokenKind.IDENTIFIER, "text": "a"},
                {"kind": TokenKind.COMMA},
                {"kind": TokenKind.IDENTIFIER, "text": "b"},
                {"kind": TokenKind.COMMA},
                {"kind": TokenKind.IDENTIFIER, "text": "c"},
                {"kind": TokenKind.R_PAREN},
                {"kind": TokenKind.IDENTIFIER, "text": "f"},
                {"kind": TokenKind.L_PAREN},
                {"kind": TokenKind.INTCONST, "text": "0"},
                {"kind": TokenKind.R_PAREN},
                {"kind": TokenKind.IDENTIFIER, "text": "f"},
                {"kind": TokenKind.L_PAREN},
                {"kind": TokenKind.INTCONST, "text": "0"},
                {"kind": TokenKind.R_PAREN},
                {"kind": TokenKind.IDENTIFIER, "text": "f"},
                {"kind": TokenKind.L_PAREN},
                {"kind": TokenKind.INTCONST, "text": "0"},
                {"kind": TokenKind.COMMA},
                {"kind": TokenKind.IDENTIFIER, "text": "a"},
                {"kind": TokenKind.COMMA},
                {"kind": TokenKind.IDENTIFIER, "text": "b"},
                {"kind": TokenKind.COMMA},
                {"kind": TokenKind.IDENTIFIER, "text": "c"},
                {"kind": TokenKind.R_PAREN},
                {"kind": TokenKind.IDENTIFIER, "text": "f"},
                {"kind": TokenKind.L_PAREN},
                {"kind": TokenKind.INTCONST, "text": "0"},
                {"kind": TokenKind.COMMA},
                {"kind": TokenKind.IDENTIFIER, "text": "a"},
                {"kind": TokenKind.R_PAREN},
                {"kind": TokenKind.IDENTIFIER, "text": "f"},
                {"kind": TokenKind.L_PAREN},
                {"kind": TokenKind.INTCONST, "text": "0"},
                {"kind": TokenKind.COMMA},
                {"kind": TokenKind.IDENTIFIER, "text": "a"},
                {"kind": TokenKind.R_PAREN},
                {"kind": TokenKind.IDENTIFIER, "text": "S"},
                {"kind": TokenKind.IDENTIFIER, "text": "foo"},
                {"kind": TokenKind.IDENTIFIER, "text": "S"},
                {"kind": TokenKind.IDENTIFIER, "text": "bar"},
                {"kind": TokenKind.EQUAL},
                {"kind": TokenKind.L_BRACE},
                {"kind": TokenKind.INTCONST, "text": "1"},
                {"kind": TokenKind.COMMA},
                {"kind": TokenKind.INTCONST, "text": "2"},
                {"kind": TokenKind.R_BRACE},
                {"kind": TokenKind.IDENTIFIER, "text": "ab"},
                {"kind": TokenKind.COMMA},
                {"kind": TokenKind.IDENTIFIER, "text": "c"},
                {"kind": TokenKind.COMMA},
                {"kind": TokenKind.IDENTIFIER, "text": "d"},
                {"kind": TokenKind.STRINGLITERAL, "content": ""},
                {"kind": TokenKind.IDENTIFIER, "text": "a"},
                {"kind": TokenKind.IDENTIFIER, "text": "b"},
                {"kind": TokenKind.IDENTIFIER, "text": "ab"},
                {"kind": TokenKind.END},
            ],
        },
        {
            "filename": "define3.txt",
            "tokens": [
                {"kind": TokenKind.STRINGLITERAL, "content": "x ## y"},
                {"kind": TokenKind.END},
            ],
        },
    ]
    examplestest(examples)


def test_defineerror():
    examples = [
        {"filename": f"defineerror{i}.txt", "expected_exception": [Error], "tokens": []}
        for i in range(1, 13)
        if i != 7
    ]
    examplestest(examples)


def test_undef():
    examples = [
        {
            "filename": "undef.txt",
            "tokens": [
                {"kind": TokenKind.INTCONST, "text": "3"},
                {"kind": TokenKind.IDENTIFIER, "text": "A"},
                {"kind": TokenKind.END},
            ],
        }
    ]
    examplestest(examples)


def test_conditional():
    examples = [
        {
            "filename": "conditional.txt",
            "tokens": [
                {"kind": TokenKind.INTCONST, "text": "5"},
                {"kind": TokenKind.END},
            ],
        }
    ]
    examplestest(examples)


def test_include():
    examples = [
        {
            "filename": "include.txt",
            "tokens": [
                {
                    "kind": TokenKind.STRINGLITERAL,
                    "content": '1abab1我的世界Minecraft\123"fdas\xabc\n\t\fc--',
                    "prefix": "U",
                },
                {
                    "kind": TokenKind.INTCONST,
                    "text": "0",
                },
                {
                    "kind": TokenKind.COMMA,
                    "text": ",",
                },
                {
                    "kind": TokenKind.INTCONST,
                    "text": "0x22",
                },
                {
                    "kind": TokenKind.COMMA,
                    "text": ",",
                },
                {
                    "kind": TokenKind.INTCONST,
                    "text": "0x31",
                },
                {
                    "kind": TokenKind.COMMA,
                    "text": ",",
                },
                {
                    "kind": TokenKind.INTCONST,
                    "text": "0x61",
                },
                {
                    "kind": TokenKind.COMMA,
                    "text": ",",
                },
                {
                    "kind": TokenKind.INTCONST,
                    "text": "0x62",
                },
                {
                    "kind": TokenKind.COMMA,
                    "text": ",",
                },
                {
                    "kind": TokenKind.INTCONST,
                    "text": "0x22",
                },
                {
                    "kind": TokenKind.COMMA,
                    "text": ",",
                },
                {
                    "kind": TokenKind.INTCONST,
                    "text": "6",
                },
                {
                    "kind": TokenKind.INTCONST,
                    "text": "0",
                },
                {"kind": TokenKind.END},
            ],
        }
    ]
    examplestest(examples)
