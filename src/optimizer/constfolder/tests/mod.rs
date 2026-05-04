#[macro_export]
macro_rules! constfolder_test_template {
    ($name:ident,$code:expr) => {
        #[test]
        pub fn $name() {
            use crate::ast::printer::Print;
            use crate::{
                optimizer::constfolder::ConstFolder,
                symtab::SymbolTable,
                typechecker::{TypeChecker, tests::quick_new_parser},
            };
            use insta::assert_snapshot;
            use std::{cell::RefCell, rc::Rc};

            let parser = quick_new_parser($code);
            let ast = parser.parse_to_ast().unwrap();

            let symtab = Rc::new(RefCell::new(SymbolTable::new()));
            let mut type_checker = TypeChecker::new(Rc::clone(&symtab));
            type_checker.check(Rc::clone(&ast)).unwrap();

            ConstFolder::new().fold(Rc::clone(&ast)).unwrap();

            assert_snapshot!(ast.print_line(0).join("\n"));
        }
    };
}

constfolder_test_template!(
    normal,
    "
int main()
{
    int a = 5;
    int b = a + 10;
    int c = b * 2;
}
"
);

constfolder_test_template!(
    r#loop,
    "
int main()
{
    int a = 5;
    int b;
    while (b) a++;
    int c = a + 5;
    for (a = 6; a <= 10; a++) b++;
    c = b + 7;
}
"
);

constfolder_test_template!(
    branch,
    "
int main()
{
    int a = 5;
    int b;
    if (b)
        a = 6;
    else
        a = 7;
    int c = a + 8;
    a = 9;
    if (b)
        a = 9;
    else
        a = 9;
    c = a + 9;
    if (b) a = 10;
    c = a + 11;
}
"
);

constfolder_test_template!(
    break_and_continue,
    "
int main()
{
    int a = 1;
    int x;
    while (x)
    {
        a = 2;
        a = 1;
    }
    int b = a + 3;

    a = 1;
    while (x)
    {
        a = 2;
        break;
        a = 1;
    }
    b = a + 4;

    a = 1;
    while (x)
    {
        a = 1;
        continue;
        a = 2;
    }
    b = a + 5;
}
"
);

constfolder_test_template!(
    function_call,
    "
void f() {}

int main()
{
    int a = 1;
    f();
    int b = a + 2;
}
"
);

constfolder_test_template!(
    global_value,
    "
const int GLOBAL_CONST = 100;
int global_var = 200;

int main()
{
    int p = GLOBAL_CONST + 1;
    int q = global_var + 1;
}
"
);

constfolder_test_template!(
    pointer_alias,
    "
int main()
{
    int x = 5;
    int *p = &x;
    *p = 10;
    int y = x + 1;
}
"
);

constfolder_test_template!(
    switch,
    "
int main()
{
    int a = 1, b = 2;
    int c;
    switch (c)
    {
    case 1: a = 1; break;
    case 2: b = 2; break;
    }
    c = a + b;
}
"
);

constfolder_test_template!(
    initializer,
    "
typedef struct {
    int a : 3;
    int b : 4;
} A;

typedef struct {
    A c;
} B;

int main() { B x = {.c = {1, 2}}; }
"
);
