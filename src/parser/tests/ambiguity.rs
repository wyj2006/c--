use crate::*;

#[test]
pub fn typedef_name() {
    CParser::parse(
        Rule::translation_unit,
        "int main()
{
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
