import ast
import os
from argparse import ArgumentParser
from basic import Diagnostic, Location, MergeReader, Error
from cast import DumpVisitor, RegExpr, Letter
from parse import generic_syntax_error

from tools.ParseBuilder import GrammarLexer, GrammarParser, LeftRecDetector

from .regexpr_builder import RegExprBuilder
from .pos_calculator import PosCalculator
from .simplifier import Simplifier
from . import generate_state, generate_code

argparser = ArgumentParser(description=f"词法分析器生成工具")
argparser.add_argument("file", help="词法对应的语法描述文件")
argparser.add_argument("--class-name", help="类名", action="store", default=None)
argparser.add_argument("--dump-ast", help="输出AST", action="store_true", default=False)
argparser.add_argument(
    "--dump-regexpr", help="输出正则表达式树", action="store_true", default=False
)
argparser.add_argument("-o", help="输出文件", default=None)

args = argparser.parse_args()
file: str = args.file
dump_regexpr: bool = args.dump_regexpr
dump_ast: bool = args.dump_ast
class_name: str = args.class_name
output_file = args.o

dirname = os.path.dirname(file)
filename, _ = os.path.splitext(os.path.basename(file))

if class_name == None:
    class_name = filename.title() + "Lexer"

if output_file == None:
    output_file = os.path.join(dirname, f"gen_{class_name.lower()}.py")

try:
    reader = MergeReader(file)
    lexer = GrammarLexer(reader)

    parser = GrammarParser(lexer)
    parser.nexttoken()

    grammar = parser.start()
    if grammar == None:
        raise generic_syntax_error(parser)

    grammar.merge()
    grammar.accept(LeftRecDetector())

    if dump_ast:
        grammar.accept(DumpVisitor())

    regexpr: RegExpr = grammar.accept(RegExprBuilder())
    regexpr = regexpr.accept(Simplifier())
    followpos: dict[Letter, set[Letter]] = {}
    regexpr.accept(PosCalculator(followpos))

    if dump_regexpr:
        regexpr.accept(DumpVisitor())

    states = generate_state(regexpr, followpos)
    module = generate_code(states, class_name, grammar.header)

    dirname = os.path.dirname(file)
    filename, _ = os.path.splitext(os.path.basename(file))
    with open(
        output_file,
        mode="w",
        encoding="utf-8",
    ) as file:
        file.write(ast.unparse(module))

except Diagnostic as e:
    e.dump()
except RecursionError:
    Error("无法转换成正则表达式(可能是因为有递归)", Location([])).dump()
