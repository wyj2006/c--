import ast
import os

from argparse import ArgumentParser
from Basic import Diagnostic, MergeReader
from AST import DumpVisitor
from Parse import generic_syntax_error

from .GrammarLexer import GrammarLexer
from .GrammarParser import GrammarParser
from .LeftRecDetector import LeftRecDetector
from .Generator import Generator

argparser = ArgumentParser(description=f"语法分析器生成工具")
argparser.add_argument("file", help="语法文件")
argparser.add_argument("--class-name", help="类名", action="store", default=None)
argparser.add_argument("--dump-ast", help="输出AST", action="store_true", default=False)
args = argparser.parse_args()
file: str = args.file
class_name: str = args.class_name
dump_ast: bool = args.dump_ast

dirname = os.path.dirname(file)
filename, _ = os.path.splitext(os.path.basename(file))

if class_name == None:
    class_name = filename.title() + "Parser"

try:
    reader = MergeReader(file)
    lexer = GrammarLexer(reader)

    parser = GrammarParser(lexer)
    parser.nexttoken()

    grammar = parser.start()
    if grammar == None:
        raise generic_syntax_error(parser)
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
