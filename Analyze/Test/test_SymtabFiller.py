from Test.Common import *


def test_Scope():
    parser = get_parser("scope.txt")
    ast: TranslationUnit = parser.start()

    symtab = Symtab(ast.location)
    ast.accept(DeclAnalyzer(symtab))
    ast.accept(SymtabFiller(symtab))

    assert len(symtab.children) == 2
    assert len(symtab.children[1].children) == 1
    assert len(symtab.children[1].children[0].children) == 1

    func_prototype: FunctionDeclarator = ast.body[0].declarators[0].declarator
    assert (
        hasattr(func_prototype, "_symtab")
        and func_prototype._symtab == symtab.children[0]
    )

    function_def: FunctionDef = ast.body[1]
    assert (
        hasattr(function_def, "_symtab") and function_def._symtab == symtab.children[1]
    )
    assert not hasattr(function_def.declarators[0], "_symtab")
    assert not hasattr(function_def.body, "_symtab")

    block: IfStmt = function_def.body.items[1]
    assert hasattr(block, "_symtab") and block._symtab == symtab.children[1].children[0]

    nested: CompoundStmt = block.body
    assert (
        hasattr(nested, "_symtab")
        and nested._symtab == symtab.children[1].children[0].children[0]
    )
