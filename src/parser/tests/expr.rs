use crate::parser_test_template;

parser_test_template!(
    string_contact,
    r#"
int main()
{
    "f" "f";
}
"#
);
