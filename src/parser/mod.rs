use pest::pratt_parser::{Assoc, Op, PrattParser};
use pest_derive::Parser;
use std::sync::LazyLock;
#[cfg(test)]
pub mod tests;

static _PRATT_PARSER: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    PrattParser::new()
        .op(Op::infix(Rule::comma, Assoc::Left))
        .op(Op::infix(Rule::or, Assoc::Left))
        .op(Op::infix(Rule::and, Assoc::Left))
        .op(Op::infix(Rule::bit_or, Assoc::Left))
        .op(Op::infix(Rule::bit_xor, Assoc::Left))
        .op(Op::infix(Rule::bit_and, Assoc::Left))
        .op(Op::infix(Rule::eq, Assoc::Left) | Op::infix(Rule::neq, Assoc::Left))
        .op(Op::infix(Rule::lt, Assoc::Left)
            | Op::infix(Rule::le, Assoc::Left)
            | Op::infix(Rule::gt, Assoc::Left)
            | Op::infix(Rule::ge, Assoc::Left))
        .op(Op::infix(Rule::lshift, Assoc::Left) | Op::infix(Rule::rshift, Assoc::Left))
        .op(Op::infix(Rule::add, Assoc::Left) | Op::infix(Rule::sub, Assoc::Left))
        .op(Op::infix(Rule::mul, Assoc::Left)
            | Op::infix(Rule::div, Assoc::Left)
            | Op::infix(Rule::r#mod, Assoc::Left))
        .op(Op::prefix(Rule::positve)
            | Op::prefix(Rule::negative)
            | Op::prefix(Rule::bit_not)
            | Op::prefix(Rule::not)
            | Op::prefix(Rule::dereference)
            | Op::prefix(Rule::addressof)
            | Op::prefix(Rule::cast)
            | Op::prefix(Rule::sizeof)
            | Op::prefix(Rule::prefix_increment)
            | Op::prefix(Rule::prefix_decrement))
        .op(Op::postfix(Rule::subscript)
            | Op::postfix(Rule::function_call)
            | Op::postfix(Rule::member_access)
            | Op::postfix(Rule::postfix_increment)
            | Op::postfix(Rule::postfix_decrement))
});

#[derive(Parser)]
#[grammar = "src/grammar/lexer.pest"]
#[grammar = "src/grammar/parser.pest"]
pub struct CParser {}
