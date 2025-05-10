from argparse import ArgumentParser
from AST import DumpVisitor, Visitor, Transformer
from Analyze import DeclAnalyzer, SymtabFiller, AttrAnalyzer, TypeChecker
from Basic import Diagnostic, Diagnostics, Symtab, TokenKind, MergeReader, Token
from Lex import Preprocessor
from Parse import Parser, generic_syntax_error

version = "0.8.0"


def main():
    argparser = ArgumentParser(description=f"c--编译器 {version}")
    argparser.add_argument("file", help="源代码文件")
    argparser.add_argument(
        "--dump-tokens", help="输出tokens", action="store_true", default=False
    )
    argparser.add_argument(
        "--dump-ast", help="输出AST", action="store_true", default=False
    )
    argparser.add_argument(
        "--dump-symtab", help="输出符号表", action="store_true", default=False
    )
    argparser.add_argument(
        "-E", help="只进行预处理", action="store_true", default=False
    )
    args = argparser.parse_args()
    file: str = args.file
    dump_tokens: bool = args.dump_tokens
    dump_ast: bool = args.dump_ast
    dump_symtab: bool = args.dump_symtab
    pp_only: bool = args.E
    try:
        reader = MergeReader(file)
        lexer = Preprocessor(reader)

        if pp_only and True:
            token: Token = lexer.next()
            while token.kind != TokenKind.END:
                print(token)
                token = lexer.next()
            print(token)
            return

        parser = Parser(lexer)
        parser.nexttoken()
        ast = parser.start()

        if dump_tokens:
            for token in lexer.tokens:
                print(token)
        if ast == None:
            raise generic_syntax_error(parser)

        symtab = Symtab(ast.location)

        exceptions = []

        for analyzer in (
            AttrAnalyzer(symtab),
            DeclAnalyzer(symtab),
            SymtabFiller(symtab),
            TypeChecker(symtab),
        ):
            try:
                if isinstance(analyzer, Transformer):
                    ast = ast.accept(analyzer)
                else:
                    ast.accept(analyzer)
            except Exception as e:
                exceptions.append(e)

        if dump_ast:
            ast.accept(DumpVisitor())
        if dump_symtab:
            symtab.print()

        if exceptions:
            raise exceptions[0]
    except Diagnostic as e:
        e.dump()
    except Diagnostics as e:
        for i in e.list:
            i.dump()


if __name__ == "__main__":
    main()
