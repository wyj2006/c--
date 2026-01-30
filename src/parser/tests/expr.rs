use crate::parser::CParser;
use crate::parser_test_template;
use insta::assert_debug_snapshot;

parser_test_template!(
    string_contact,
    r#"
int main()
{
    "f" "f";
}
"#
);
