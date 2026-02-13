use crate::typechecker_test_template;

typechecker_test_template!(
    declexpr_expr,
    r#"int main()
{
    int a;
    a;
}
"#
);

typechecker_test_template!(
    declexpr_decl,
    r#"int main()
{
    typedef int a;
    a;
}
"#
);
