use crate::parser::*;
use pest::Parser;

#[test]
pub fn typedef_name() {
    CParser::parse(
        Rule::translation_unit,
        "int main()
{
    struct A{
        int a:1;
    };
    int a;
    sizeof a;
    A a;
    A;
    sizeof(a);
    A f(int*);
}
",
    )
    .unwrap();
}
