use super::{Preprocessor, Rule};
use crate::preprocessor::token::Token;
use codespan_reporting::diagnostic::Diagnostic;
use pest::iterators::Pair;

impl Preprocessor {
    pub fn process_pragma(&mut self, rule: Pair<Rule>) -> Result<Vec<Token>, Diagnostic<usize>> {
        let mut result = Vec::new();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::pp_tokens => {
                    //TODO 具体处理这些指令
                    match rule
                        .as_str()
                        .split_ascii_whitespace()
                        .collect::<Vec<&str>>()
                        .as_slice()
                    {
                        ["STDC", "FP_CONTRACT", "ON" | "OFF" | "DEFAULT"] => {}
                        ["STDC", "FENV_ACCESS", "ON" | "OFF" | "DEFAULT"] => {}
                        [
                            "STDC",
                            "FENV_DEC_ROUND",
                            "FE_DEC_DOWNWARD"
                            | "FE_DEC_TONEAREST"
                            | "FE_DEC_TONEARESTFROMZERO"
                            | "FE_DEC_TOWARDZERO"
                            | "FE_DEC_UPWARD"
                            | "FE_DEC_DYNAMIC",
                        ] => {}
                        [
                            "STDC",
                            "FENV_ROUND",
                            "FE_DOWNWARD"
                            | "FE_TONEAREST"
                            | "FE_TONEARESTFROMZERO"
                            | "FE_TOWARDZERO"
                            | "FE_UPWARD"
                            | "FE_DYNAMIC",
                        ] => {}
                        ["STDC", "CX_LIMITED_RANGE", "ON" | "OFF" | "DEFAULT"] => {}
                        _ => {}
                    }
                }
                Rule::newline => result.extend(self.to_tokens(rule, false)),
                _ => unreachable!(),
            }
        }
        Ok(result)
    }
}
