"""
语法分析器生成工具
参考PEG文法, 是它的不完全实现
"""

# TODO: PEG文法完全实现

from argparse import ArgumentParser
import ast
import os
from Basic import Diagnostic, FileReader
from AST import DumpVisitor
from .Generator import Generator
from .ParserBase import ParserBase
from .GrammarLexer import GrammarLexer
from .Memory import memorize, memorize_left_rec
from .LeftRecDetector import LeftRecDetector
from .GrammarParser import GrammarParser


def main(args):

    argparser = ArgumentParser(description=f"语法分析器生成工具")
    argparser.add_argument("file", help="语法文件")
    argparser.add_argument("-class_name", help="类名", action="store", default=None)
    argparser.add_argument(
        "-dump-ast", help="输出AST", action="store_true", default=False
    )
    args = argparser.parse_args(args)
    file: str = args.file
    class_name: str = args.class_name
    dump_ast: bool = args.dump_ast

    dirname = os.path.dirname(file)
    filename, _ = os.path.splitext(os.path.basename(file))

    if class_name == None:
        class_name = filename.title() + "Parser"

    try:
        reader = FileReader(file)
        lexer = GrammarLexer(reader)  # TODO 记得替换

        parser = GrammarParser(lexer)
        parser.nexttoken()

        grammar = parser.start()
        grammar.class_name = class_name
        grammar.merge()
        grammar.accept(LeftRecDetector())
        if dump_ast:
            grammar.accept(DumpVisitor())

        pyast = grammar.accept(Generator())
        code = ast.unparse(pyast)

        dirname = os.path.dirname(file)
        filename, _ = os.path.splitext(os.path.basename(file))
        with open(
            os.path.join(dirname, f"gen_{grammar.class_name}.py"),
            mode="w",
            encoding="utf-8",
        ) as file:
            file.write(code)
    except Diagnostic as e:
        e.dump()
