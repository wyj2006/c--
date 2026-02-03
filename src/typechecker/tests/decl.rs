use crate::{
    ctype::pointee,
    symtab::{Namespace, SymbolTable},
    typechecker::{TypeChecker, tests::quick_new_parser},
    typechecker_test_template,
};
use std::{cell::RefCell, rc::Rc};

typechecker_test_template!(
    ambiguity,
    "
typedef int A;
void f(A);
void g(A a);
void scanf(char* format,...);
int main()
{
    typedef int A;
    typedef A B;
    A;
    A a;
    B b;
}
"
);

#[test]
pub fn reassign() {
    let parser = quick_new_parser(
        "
int main()
{
    struct A a;
    struct A {int b;}b;
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

#[test]
#[should_panic]
pub fn incomplete_member() {
    let parser = quick_new_parser(
        "
struct A{
    struct A a;
};
",
    );
    let ast = parser.parse_to_ast().unwrap();

    let symtab = Rc::new(RefCell::new(SymbolTable::new()));
    let mut type_checker = TypeChecker::new(Rc::clone(&symtab));
    type_checker.check(Rc::clone(&ast)).unwrap();
}

#[test]
pub fn forward_declare() {
    let parser = quick_new_parser(
        "
struct s* p = (void*)0;
struct s { int a; };
void g(void)
{
    struct s;
    struct s *p;
    struct s { char* p; };
}
",
    );
    let ast = parser.parse_to_ast().unwrap();

    let symtab = Rc::new(RefCell::new(SymbolTable::new()));
    let mut type_checker = TypeChecker::new(Rc::clone(&symtab));
    type_checker.check(Rc::clone(&ast)).unwrap();

    let s1 = symtab
        .borrow()
        .lookup(Namespace::Tag, &"s".to_string())
        .unwrap();
    let p1 = symtab
        .borrow()
        .lookup(Namespace::Ordinary, &"p".to_string())
        .unwrap();

    assert!(Rc::ptr_eq(
        &s1.borrow().r#type,
        &pointee(Rc::clone(&p1.borrow().r#type)).unwrap()
    ));

    let symtab2 = &symtab.borrow().children[0];
    let s2 = symtab2
        .borrow()
        .lookup(Namespace::Tag, &"s".to_string())
        .unwrap();
    let p2 = symtab2
        .borrow()
        .lookup(Namespace::Ordinary, &"p".to_string())
        .unwrap();

    assert!(Rc::ptr_eq(
        &s2.borrow().r#type,
        &pointee(Rc::clone(&p2.borrow().r#type)).unwrap()
    ));

    assert!(!Rc::ptr_eq(&s1.borrow().r#type, &s2.borrow().r#type));
}
