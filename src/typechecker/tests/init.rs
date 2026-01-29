use crate::{parser::CParser, symtab::SymbolTable, test_template, typechecker::TypeChecker};
use insta::assert_debug_snapshot;
use std::{cell::RefCell, rc::Rc};

test_template!(
    init_scale,
    "
int main()
{
    int a={1};
}
"
);

test_template!(
    init_array_without_designator,
    "
int main()
{
    int a[3][4][5]={{1,2},3,4};
}
"
);

test_template!(
    init_array_with_designator,
    "
int main()
{
    int a[3][4][5]={[0][1]={1,2},3,4};
}
"
);

test_template!(
    init_incomplete_array,
    "
int main()
{   
    int a[][4][5]={1,2,3,4};
}
"
);

test_template!(
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
