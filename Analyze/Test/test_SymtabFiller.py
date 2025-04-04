import pytest
from Test.Common import *


def test_scope():
    parser = get_parser("scope.txt")
    parser.nexttoken()
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


def test_decl_point_record():
    parser = get_parser("decl_point_record.txt")
    parser.nexttoken()
    ast: TranslationUnit = parser.start()

    symtab = Symtab(ast.location)
    ast.accept(DeclAnalyzer(symtab))
    try:
        ast.accept(SymtabFiller(symtab))
    except (Diagnostic, Diagnostics):
        pytest.fail("期望没有错误, 但产生了一个错误")


def test_decl_point_enumconst():
    parser = get_parser("decl_point_enumconst.txt")
    parser.nexttoken()
    ast: TranslationUnit = parser.start()

    symtab = Symtab(ast.location)
    ast.accept(DeclAnalyzer(symtab))
    ast.accept(SymtabFiller(symtab))

    symtab0: Symtab = symtab.children[0]
    x1: EnumConst = symtab0.lookup("x")

    symtab00: Symtab = symtab0.children[0]
    x2: EnumConst = symtab00.lookup("x")
    y: EnumConst = symtab00.lookup("y")

    assert x1 is not x2
    assert x1 is x2.value.left.symbol
    assert x2 is y.value.left.symbol


def test_decl_point_other():
    parser = get_parser("decl_point_other.txt")
    parser.nexttoken()
    ast: TranslationUnit = parser.start()

    symtab = Symtab(ast.location)
    ast.accept(DeclAnalyzer(symtab))

    try:
        ast.accept(SymtabFiller(symtab))
    except (Diagnostic, Diagnostics):
        pytest.fail("期望没有错误, 但产生了一个错误")

    symtab0: Symtab = symtab.children[0]
    x1: Object = symtab0.lookup("x")

    symtab00: Symtab = symtab0.children[0]
    x2: Object = symtab00.lookup("x")

    assert x1 is not x2
    assert x2.initializer.symbol is x2
