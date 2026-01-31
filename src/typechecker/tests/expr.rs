use crate::{
    parser::CParser, symtab::SymbolTable, typechecker::TypeChecker, typechecker_test_template,
};
use insta::assert_debug_snapshot;
use std::{cell::RefCell, rc::Rc};

typechecker_test_template!(
    builtin_constant,
    "
int main()
{
    true;
    false;
    nullptr;
}
"
);

typechecker_test_template!(
    string_and_char,
    r#"
int main()
{
    '猫';
    u'猫';
    U'猫';
    L'猫';
    "猫我";
    u"猫我";
    U"猫我";
    L"猫我";
}
"#
);

typechecker_test_template!(
    integer_literal,
    "
int main()
{
    1;
    1uwb;
}
"
);

typechecker_test_template!(
    float_literal,
    "
int main()
{
    0.1;
    0.;
    .1;
    0.1e-2;
    0x0.fp1;
}
"
);

typechecker_test_template!(
    generic_selection,
    "
int main()
{
    int a;
    _Generic(a,char:1,int:2,default:3);
}
"
);

typechecker_test_template!(
    function_call,
    "
int f(long long n)
{

}

int main()
{
    f(1);
}
"
);

typechecker_test_template!(
    subscript,
    r#"
int main()
{
    int a;
    int b[3];
    b[a];
    a["1"];
}
"#
);

typechecker_test_template!(
    member_access,
    "
int main()
{
    struct A{int a;} a;
    a.a;
}
"
);

typechecker_test_template!(
    deref_lvalue,
    "
int main()
{
    int *a;
    *a;
}
"
);

typechecker_test_template!(
    varparam_function_call,
    r#"
void printf(char* message,...);

int main()
{
    float a;
    const char b;
    printf("Hello,World",a,b);
}
"#
);

typechecker_test_template!(
    decls_in_expr,
    "
int main()
{
    alignas(8) int a;
    (struct A{int a;}){1};
    sizeof(struct B{int a;});
    typeof(struct C{int a;});
    struct C b;
    b.a;
    _Atomic(struct D{int a;});
    struct D c;
    c.a;
    struct A d;
    d.a;
}
"
);

#[test]
#[should_panic]
pub fn addressof_bitfield() {
    let parser = CParser::new(
        "
struct A{
    int a:1;
}a;

int main()
{
    &a.a;
}
",
    );
    let ast = parser.parse_to_ast().unwrap();

    let symtab = Rc::new(RefCell::new(SymbolTable::new()));
    let mut type_checker = TypeChecker::new("<string>", Rc::clone(&symtab));
    type_checker.check(Rc::clone(&ast)).unwrap();
}
