use crate::{
    parser::CParser, symtab::SymbolTable, typechecker::TypeChecker, typechecker_test_template,
};
use insta::assert_debug_snapshot;
use std::{cell::RefCell, rc::Rc};

typechecker_test_template!(
    init_scale,
    "
int main()
{
    int a={1};
}
"
);

typechecker_test_template!(
    init_array_without_designator,
    "
int main()
{
    int a[3][4][5]={{1,2},3,4};
}
"
);

typechecker_test_template!(
    init_array_with_designator,
    "
int main()
{
    int a[3][4][5]={[0][1]={1,2},3,4};
    int b[3][4]={[1]=1,2,3};
}
"
);

typechecker_test_template!(
    init_incomplete_array,
    "
int main()
{   
    int a[][4][5]={1,2,3,4};
}
"
);

typechecker_test_template!(
    init_record,
    "
int main()
{
    struct{
        int x[3][4][5];
    }a={1,2,3,4};
}
"
);

typechecker_test_template!(
    init_with_string,
    r#"
int main()
{
    char a[2]="fdas";
    char b[]={"fdsa"};
    char c[][4][5]={[1]="fdas","f",1};
    struct A{char a[5];}d={"fdsa"};
}
"#
);
