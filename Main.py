import colorama
from argparse import ArgumentParser
from AST import DumpVisitor
from Analyze import DeclAnalyzer
from Basic import Error, FileReader, Diagnostics, Symtab
from Lex import Preprocessor
from Parse import Parser, generate_diagnostic

version = "1.0.0"

colorama.init(autoreset=True)


def main():
    argparser = ArgumentParser(description=f"c--编译器 {version}")
    argparser.add_argument("file", help="源代码文件")
    argparser.add_argument(
        "-dump-tokens", help="输出tokens", action="store_true", default=False
    )
    argparser.add_argument(
        "-dump-ast", help="输出AST", action="store_true", default=False
    )
    argparser.add_argument(
        "-dump-calltree", help="输出调用树", action="store_true", default=False
    )
    args = argparser.parse_args()
    try:
        reader = FileReader(args.file)
        lexer = Preprocessor(reader)

        parser = Parser(lexer)
        ast = parser.start()

        if args.dump_tokens:
            for token in lexer.tokens:
                print(token)
        if ast == None:
            diagnostics = generate_diagnostic(parser.call_tree)
            if args.dump_calltree:
                parser.call_tree.print()
            raise diagnostics

        symtab = Symtab(ast.location)
        ast.accept(DeclAnalyzer(symtab))

        symtab.print()

        if args.dump_ast and ast != None:
            ast.accept(DumpVisitor())
    except Error as e:
        e.dump()
    except Diagnostics as e:
        for i in e.list:
            i.dump()


if __name__ == "__main__":
    main()
