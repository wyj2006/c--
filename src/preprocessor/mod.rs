pub mod cmacro;
pub mod expressions;
pub mod pragma;
#[cfg(test)]
pub mod tests;
pub mod token;

use crate::diagnostic::{from_pest_span, map_pest_err, warning};
use crate::file_map::source_map;
use crate::files;
use crate::preprocessor::cmacro::MacroKind;
use crate::preprocessor::token::{Token, TokenKind, to_string};
use cmacro::Macro;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::Files;
use lazy_static::lazy_static;
use pest::iterators::{Pair, Pairs};
use pest::{Parser, Span};
use pest_derive::Parser;
use std::cmp::min;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::{fs, usize};

lazy_static! {
    pub static ref user_macro: RwLock<HashMap<String, Macro>> = RwLock::new(HashMap::new());
    pub static ref include_paths: RwLock<Vec<PathBuf>> = RwLock::new(Vec::new());
}

#[derive(Parser)]
#[grammar = "src/grammar/lexer.pest"]
#[grammar = "src/grammar/preprocessor.pest"]
pub struct Preprocessor {
    pub file_id: usize,
    pub line_offset: isize,
    pub file_name: String,
}

impl Preprocessor {
    pub fn new(file_id: usize) -> Preprocessor {
        Preprocessor {
            file_id,
            line_offset: 0,
            file_name: files.lock().unwrap().name(file_id).unwrap(),
        }
    }

    pub fn file_path(&self) -> String {
        files.lock().unwrap().name(self.file_id).unwrap()
    }

    pub fn process(&mut self) -> Result<Vec<Token>, Diagnostic<usize>> {
        let source = files.lock().unwrap().source(self.file_id).unwrap();
        let rules = map_pest_err(
            self.file_id,
            Preprocessor::parse(Rule::preprocessing_file, source.as_str()),
        )?;
        let mut result = Vec::new();
        for rule in rules {
            if let Rule::preprocessing_file = rule.as_rule() {
                for rule in rule.into_inner() {
                    match rule.as_rule() {
                        Rule::group_part => result.extend(self.process_group(rule)?),
                        Rule::WHITESPACE => result.extend(self.to_tokens(rule, false)),
                        Rule::EOI => {}
                        _ => unreachable!(),
                    }
                }
            }
        }
        Ok(result)
    }

    fn process_group(&mut self, rule: Pair<Rule>) -> Result<Vec<Token>, Diagnostic<usize>> {
        let mut result = Vec::new();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::up_directives => {
                    result.extend(self.process_up_directive(rule)?);
                }
                Rule::text_line => {
                    result
                        .extend(self.replace_macro(self.to_tokens(rule, false), &mut Vec::new())?);
                }
                Rule::non_directive_line => {
                    warning(
                        format!("unkown preprocessing directive: {}", rule.as_str()),
                        self.file_id,
                        from_pest_span(rule.as_span()),
                        vec![],
                    );
                }
                Rule::WHITESPACE => result.extend(self.to_tokens(rule, false)),
                _ => unreachable!(),
            }
        }
        Ok(result)
    }

    fn process_up_directive(&mut self, rule: Pair<Rule>) -> Result<Vec<Token>, Diagnostic<usize>> {
        let mut input = String::new();
        if let Rule::up_directives = rule.as_rule() {
            for pair in rule.clone().into_inner() {
                if let Rule::directive_keyword = pair.as_rule()
                    //不允许替换后再处理的指令
                    && !(vec!["include", "embed", "line"].contains(&pair.as_str()))
                {
                    input = rule.as_str().to_string();
                    break;
                } else if let Rule::up_if_section = pair.as_rule() {
                    input = rule.as_str().to_string();
                    break;
                }
            }
        }
        if input == "" {
            use codespan::Span;
            // '#'的位置
            let hash_start = rule.as_span().start();
            let hash_end = rule.as_span().start() + 1;
            //最后换行的位置
            let newline_start = rule.as_span().end();
            let newline_end = rule.as_span().end() + 1;
            let mut input_tokens = vec![Token::new(
                self.file_id,
                Span::new(hash_start as u32, hash_end as u32),
                TokenKind::Text {
                    is_whitespace: false,
                    content: "#".to_string(),
                },
            )];
            input_tokens.extend(self.replace_macro(self.to_tokens(rule, false), &mut Vec::new())?);
            input_tokens.push(Token::new(
                self.file_id,
                Span::new(newline_start as u32, newline_end as u32),
                TokenKind::Newline,
            ));
            //替换后再处理
            input = to_string(&input_tokens);

            let part_id = source_map(self.file_path(), &input_tokens);
            let mut child = Preprocessor::new(part_id);
            let result = child.process_directive(map_pest_err(
                self.file_id,
                Preprocessor::parse(Rule::directives, &input),
            )?)?;
            self.line_offset = child.line_offset;
            self.file_name = child.file_name;

            Ok(result)
        } else {
            self.process_directive(map_pest_err(
                self.file_id,
                Preprocessor::parse(Rule::directives, &input),
            )?)
        }
    }

    fn process_directive(&mut self, rules: Pairs<Rule>) -> Result<Vec<Token>, Diagnostic<usize>> {
        let mut result = Vec::new();
        for rule in rules {
            if let Rule::directives = rule.as_rule() {
                for rule in rule.into_inner() {
                    match rule.as_rule() {
                        Rule::macro_define => {
                            result.extend(self.process_macro_define(rule)?);
                        }
                        Rule::macro_undef => {
                            result.extend(self.process_macro_undef(rule)?);
                        }
                        Rule::warning_directive => {
                            result.extend(self.process_warning(rule)?);
                        }
                        Rule::error_directive => {
                            result.extend(self.process_error(rule)?);
                        }
                        Rule::line_control => {
                            result.extend(self.process_line_control(rule)?);
                        }
                        Rule::source_file_inclusion => {
                            result.extend(self.process_source_file_inclusion(rule)?);
                        }
                        Rule::binary_resource_inclusion => {
                            result.extend(self.process_binary_resource_inclusion(rule)?);
                        }
                        Rule::if_section => {
                            result.extend(self.process_if_section(rule)?);
                        }
                        Rule::pragma_directive => {
                            result.extend(self.process_pragma(rule)?);
                        }
                        Rule::newline => result.extend(self.to_tokens(rule, false)),
                        Rule::WHITESPACE => {}
                        _ => unreachable!(),
                    }
                }
            }
        }
        Ok(result)
    }

    pub fn process_macro_define(
        &mut self,
        rule: Pair<Rule>,
    ) -> Result<Vec<Token>, Diagnostic<usize>> {
        let mut result = Vec::new();
        let mut object_like = true;
        let mut name = String::new();
        let mut name_span = rule.as_span();
        let mut parameters = Vec::new();
        let mut has_varparam = false;
        let mut replace_list = Vec::new();
        let span = rule.as_span();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::identifier => {
                    name = rule.as_str().to_string();
                    name_span = rule.as_span();
                }
                Rule::function_identifier => {
                    object_like = false;
                    //去掉紧跟在后面的'('
                    name = rule.as_str()[0..rule.as_str().len() - 1].to_string();
                }
                Rule::identifier_list => {
                    for rule in rule.into_inner() {
                        if let Rule::identifier = rule.as_rule() {
                            parameters.push(rule.as_str().to_string());
                        }
                    }
                }
                Rule::replacement_list => replace_list = self.to_tokens(rule, false),
                Rule::varparam_symbol => has_varparam = true,
                Rule::newline => result.extend(self.to_tokens(rule, false)),
                Rule::WHITESPACE => {}
                _ => unreachable!(),
            }
        }
        let cmacro = Macro {
            replace_list,
            kind: if object_like {
                MacroKind::Object
            } else {
                MacroKind::Function {
                    parameters,
                    has_varparam,
                }
            },
            ..Macro::new(self.file_id, from_pest_span(span), name.clone())
        };
        if let Some(pre_macro) = self.find_macro(&name, from_pest_span(name_span)) {
            if pre_macro != cmacro {
                return Err(Diagnostic::error()
                    .with_message(format!("macro '{}' redefined", name))
                    .with_label(Label::primary(self.file_id, from_pest_span(span))));
            }
        } else {
            user_macro.write().unwrap().insert(name, cmacro);
        }
        Ok(result)
    }

    pub fn process_macro_undef(
        &mut self,
        rule: Pair<Rule>,
    ) -> Result<Vec<Token>, Diagnostic<usize>> {
        let mut result = Vec::new();
        let mut name = String::new();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::identifier => name = rule.as_str().to_string(),
                Rule::newline => result.extend(self.to_tokens(rule, false)),
                Rule::WHITESPACE => {}
                _ => unreachable!(),
            }
        }
        user_macro.write().unwrap().remove(&name);
        Ok(result)
    }

    pub fn process_warning(&mut self, rule: Pair<Rule>) -> Result<Vec<Token>, Diagnostic<usize>> {
        let mut result = Vec::new();
        let mut message = String::new();
        let span = rule.as_span();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::pp_tokens => message = rule.as_str().to_string(),
                Rule::newline => result.extend(self.to_tokens(rule, false)),
                Rule::WHITESPACE => {}
                _ => unreachable!(),
            }
        }
        warning(message, self.file_id, from_pest_span(span), vec![]);
        Ok(result)
    }

    pub fn process_error(&mut self, rule: Pair<Rule>) -> Result<Vec<Token>, Diagnostic<usize>> {
        let mut message = String::new();
        let span = rule.as_span();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::pp_tokens => message = rule.as_str().to_string(),
                Rule::newline => {}
                _ => unreachable!(),
            }
        }
        Err(Diagnostic::error()
            .with_message(message)
            .with_label(Label::primary(self.file_id, from_pest_span(span))))
    }

    pub fn process_line_control(
        &mut self,
        rule: Pair<Rule>,
    ) -> Result<Vec<Token>, Diagnostic<usize>> {
        let mut result = Vec::new();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::digit_sequence => {
                    self.line_offset = rule.as_str().parse::<isize>().unwrap()
                        - rule.as_span().start_pos().line_col().0 as isize
                        - 1
                }
                Rule::string_literal => {
                    self.file_name = self.process_string_literal(rule)?;
                }
                Rule::newline => result.extend(self.to_tokens(rule, false)),
                Rule::WHITESPACE => {}
                _ => unreachable!(),
            }
        }
        Ok(result)
    }

    pub fn get_possible_filepath(&self, header_name: &str) -> Vec<PathBuf> {
        let mut search_path = include_paths
            .read()
            .unwrap()
            .iter()
            .map(|x| PathBuf::from(x))
            .collect::<Vec<PathBuf>>();
        let mut possible_path = Vec::new();
        if header_name.starts_with('"') {
            if let Some(t) = Path::new(&self.file_path()).parent() {
                search_path.push(PathBuf::from(t));
            }
        }
        let file_name = &header_name[1..header_name.len() - 1].to_string(); //去掉前后的<>或"
        for path in search_path {
            let file_path = path.join(&file_name);
            if file_path.exists() {
                possible_path.push(file_path)
            }
        }
        possible_path
    }

    pub fn process_source_file_inclusion(
        &mut self,
        rule: Pair<Rule>,
    ) -> Result<Vec<Token>, Diagnostic<usize>> {
        let mut result = Vec::new();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::header_name => 'outer: loop {
                    for file_path in self.get_possible_filepath(rule.as_str()) {
                        let file_content = fs::read_to_string(&file_path).or_else(|e| {
                            Err(Diagnostic::error().with_message(e.to_string()).with_label(
                                Label::primary(self.file_id, from_pest_span(rule.as_span()))
                                    .with_message("error occured when opening or reading"),
                            ))
                        })?;
                        let include_id = files
                            .lock()
                            .unwrap()
                            .add(file_path.to_string_lossy().to_string(), file_content);
                        let mut child = Preprocessor::new(include_id);
                        result.extend(child.process()?);
                        break 'outer;
                    }
                    return Err(Diagnostic::error()
                        .with_message(format!(
                            "file '{}' not found",
                            rule.as_str()[1..rule.as_str().len() - 1].to_string()
                        ))
                        .with_label(Label::primary(self.file_id, from_pest_span(rule.as_span()))));
                },
                Rule::newline => result.extend(self.to_tokens(rule, false)),
                Rule::WHITESPACE => {}
                _ => unreachable!(),
            }
        }
        Ok(result)
    }

    pub fn process_binary_resource_inclusion(
        &mut self,
        rule: Pair<Rule>,
    ) -> Result<Vec<Token>, Diagnostic<usize>> {
        let mut result = Vec::new();
        let mut header_name = "";
        let mut limit = usize::MAX;
        let mut prefix = Vec::new();
        let mut suffix = Vec::new();
        let mut if_empty = Vec::new();
        let span = rule.as_span();

        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::header_name => {
                    header_name = rule.as_str();
                }
                Rule::embed_parameter_sequence => {
                    (limit, prefix, suffix, if_empty) = self.process_embed_parameters(rule)?;
                }
                Rule::newline => result.extend(self.to_tokens(rule, false)),
                Rule::WHITESPACE => {}
                _ => unreachable!(),
            }
        }

        for file_path in self.get_possible_filepath(header_name) {
            let mut buf = [0; 1024];
            let file = File::open(file_path).or_else(|e| {
                Err(Diagnostic::error().with_message(e.to_string()).with_label(
                    Label::primary(self.file_id, from_pest_span(span))
                        .with_message("error occurred when opening"),
                ))
            })?;
            let mut reader = BufReader::new(file);
            let mut data: Vec<u8> = Vec::new();
            loop {
                let n = reader.read(&mut buf).or_else(|e| {
                    Err(Diagnostic::error().with_message(e.to_string()).with_label(
                        Label::primary(self.file_id, from_pest_span(span))
                            .with_message("error occurred when reading"),
                    ))
                })?;
                if n == 0 {
                    break;
                }
                data.extend(&buf[0..n]);
                if data.len() >= limit {
                    break;
                }
            }
            result.splice(
                0..0,
                if data.len() > 0 {
                    let mut t = Vec::new();
                    t.extend(prefix);
                    t.push(Token::new(
                        self.file_id,
                        from_pest_span(span),
                        TokenKind::Text {
                            is_whitespace: false,
                            content: data[0..min(data.len(), limit)]
                                .iter()
                                .map(|x| x.to_string())
                                .collect::<Vec<String>>()
                                .join(", "),
                        },
                    ));
                    t.extend(suffix);
                    t
                } else {
                    if_empty
                },
            );
            return Ok(result);
        }
        Err(Diagnostic::error()
            .with_message(format!(
                "file '{}' not found",
                header_name[1..header_name.len() - 1].to_string()
            ))
            .with_label(Label::primary(self.file_id, from_pest_span(span))))
    }

    pub fn process_embed_parameters(
        &mut self,
        rule: Pair<Rule>,
    ) -> Result<(usize, Vec<Token>, Vec<Token>, Vec<Token>), Diagnostic<usize>> {
        let mut limit = usize::MAX;
        let mut prefix = Vec::new();
        let mut suffix = Vec::new();
        let mut if_empty = Vec::new();

        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::pp_parameter => {
                    let mut param_name = "";
                    let mut param_clause_str = "";
                    let mut param_clause_tks = Vec::new();
                    let mut param_clause_span = None;
                    let span = rule.as_span();
                    for rule in rule.into_inner() {
                        match rule.as_rule() {
                            Rule::pp_parameter_name => {
                                param_name = rule.as_str();
                            }
                            Rule::pp_parameter_clause => {
                                let span = rule.as_span();
                                //排除两边的括号
                                param_clause_span =
                                    Span::new(span.get_input(), span.start() + 1, span.end() - 1);
                                param_clause_str = rule.as_str();
                                param_clause_str = &param_clause_str[1..param_clause_str.len() - 1]; //去掉'(' ')'
                                param_clause_tks = self.to_tokens(rule, false);
                            }
                            Rule::WHITESPACE => {}
                            _ => unreachable!(),
                        }
                    }
                    match param_name {
                        "limit" | "__limit__" => {
                            let part_id = source_map(
                                self.file_path(),
                                &vec![Token::new(
                                    self.file_id,
                                    from_pest_span(param_clause_span.unwrap_or(span)),
                                    TokenKind::Text {
                                        is_whitespace: false,
                                        content: param_clause_str.to_string(),
                                    },
                                )],
                            );
                            limit = Preprocessor::new(part_id).process_constant_expression(
                                map_pest_err(
                                    part_id,
                                    Preprocessor::parse(
                                        Rule::constant_expression,
                                        param_clause_str,
                                    ),
                                )?
                                .into_iter()
                                .next()
                                .unwrap(),
                            )? as usize;
                        }
                        "prefix" | "__prefix__" => {
                            prefix = param_clause_tks;
                        }
                        "suffix" | "__suffix__" => {
                            suffix = param_clause_tks;
                        }
                        "if_empty" | "__if_empty__" => {
                            if_empty = param_clause_tks;
                        }
                        _ => {
                            warning(
                                format!("unkown parameter: {}", param_name),
                                self.file_id,
                                from_pest_span(span),
                                vec![],
                            );
                        }
                    }
                }
                Rule::WHITESPACE => {}
                _ => unreachable!(),
            }
        }

        Ok((limit, prefix, suffix, if_empty))
    }

    pub fn process_if_section(
        &mut self,
        rule: Pair<Rule>,
    ) -> Result<Vec<Token>, Diagnostic<usize>> {
        let mut result = Vec::new();
        for rule in rule.into_inner() {
            match rule.as_rule() {
                Rule::if_group | Rule::elif_group => {
                    let mut tag = "";
                    let mut is_true = false;
                    let mut condition = 0;
                    let mut macro_name = String::new();
                    let mut macro_span = rule.as_span();
                    let mut groups = Vec::new();
                    let mut after_newline = false;
                    for rule in rule.into_inner() {
                        match rule.as_rule() {
                            Rule::constant_expression => {
                                condition = self.process_constant_expression(rule)?;
                                tag = "if";
                            }
                            Rule::group_part => groups.push(rule),
                            Rule::identifier => {
                                macro_name = rule.as_str().to_string();
                                macro_span = rule.as_span();
                                match rule.as_node_tag().unwrap_or("") {
                                    "ifdef" => tag = "ifdef",
                                    "ifndef" => tag = "ifndef",
                                    _ => unreachable!(),
                                }
                            }
                            Rule::newline => {
                                result.extend(self.to_tokens(rule, false));
                                after_newline = true;
                            }
                            Rule::WHITESPACE => {
                                if after_newline {
                                    result.extend(self.to_tokens(rule, false));
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                    match tag {
                        "if" => is_true = condition != 0,
                        "ifdef" => {
                            if let Some(_) =
                                self.find_macro(&macro_name, from_pest_span(macro_span))
                            {
                                is_true = true;
                            }
                        }
                        "ifndef" => {
                            if let None = self.find_macro(&macro_name, from_pest_span(macro_span)) {
                                is_true = true;
                            }
                        }
                        _ => unreachable!(),
                    }

                    if is_true {
                        for group in groups {
                            result.extend(self.process_group(group)?);
                        }
                        return Ok(result);
                    }
                }
                Rule::else_group => {
                    let mut groups = Vec::new();
                    let mut after_newline = false;
                    for rule in rule.into_inner() {
                        match rule.as_rule() {
                            Rule::group_part => groups.push(rule),
                            Rule::newline => {
                                result.extend(self.to_tokens(rule, false));
                                after_newline = true;
                            }
                            Rule::WHITESPACE => {
                                if after_newline {
                                    result.extend(self.to_tokens(rule, false));
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                    for group in groups {
                        result.extend(self.process_group(group)?);
                    }
                    return Ok(result);
                }
                Rule::endif_line => {}
                Rule::newline => result.extend(self.to_tokens(rule, false)),
                Rule::WHITESPACE => {}
                _ => unreachable!(),
            }
        }
        Ok(result)
    }
}
