///包含处理pragma的代码
use super::{Preprocessor, Rule};
use codespan_reporting::diagnostic::Diagnostic;
use pest::iterators::Pair;

impl Preprocessor {
    pub fn process_pragma(&mut self, rule: Pair<Rule>) -> Result<String, Diagnostic<usize>> {
        for rule in rule.into_inner() {
            if let Rule::pp_tokens = rule.as_rule() {
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
        }
        Ok("\n".to_string())
    }
}
