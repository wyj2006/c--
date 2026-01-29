use crate::{
    parser::CParser,
    symtab::{Namespace, SymbolTable},
    typechecker::TypeChecker,
};
use std::{cell::RefCell, rc::Rc};

#[test]
pub fn reassign() {
    let parser = CParser::new(
        "
int main()
{
    struct A a;
    struct A {struct A a;int b;}b;
    struct A c;
    typedef int d;
    d e;
}
",
    );
    let ast = parser.parse_to_ast().unwrap();

    let symtab = Rc::new(RefCell::new(SymbolTable::new()));
    let mut type_checker = TypeChecker::new(Rc::clone(&symtab));
    type_checker.check(Rc::clone(&ast)).unwrap();

    let symtab = &symtab.borrow().children[0];
    let a = symtab
        .borrow()
        .lookup(Namespace::Ordinary, &"a".to_string())
        .unwrap();
    let b = symtab
        .borrow()
        .lookup(Namespace::Ordinary, &"b".to_string())
        .unwrap();
    let c = symtab
        .borrow()
        .lookup(Namespace::Ordinary, &"c".to_string())
        .unwrap();
    let d = symtab
        .borrow()
        .lookup(Namespace::Ordinary, &"d".to_string())
        .unwrap();
    let e = symtab
        .borrow()
        .lookup(Namespace::Ordinary, &"e".to_string())
        .unwrap();

    assert!(Rc::ptr_eq(&a.borrow().r#type, &b.borrow().r#type));
    assert!(Rc::ptr_eq(&a.borrow().r#type, &c.borrow().r#type));
    assert!(Rc::ptr_eq(&b.borrow().r#type, &c.borrow().r#type));
    assert!(Rc::ptr_eq(&d.borrow().r#type, &e.borrow().r#type));
}
