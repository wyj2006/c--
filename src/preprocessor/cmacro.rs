use super::Preprocessor;
use crate::file_map::source_lookup;
use crate::files;
use crate::preprocessor::token::{Token, TokenKind};
use crate::preprocessor::user_macro;
use chrono::{Datelike, Local};
use codespan::Span;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::Files;

pub static STDC_EMBED_NOT_FOUND: isize = 0;
pub static STDC_EMBED_FOUND: isize = 1;
pub static STDC_EMBED_EMPTY: isize = 2;

#[derive(Debug, Clone)]
pub struct Macro {
    pub file_id: usize,
    pub span: Span,
    pub name: String,
    pub replace_list: Vec<Token>,
    pub kind: MacroKind,
}

impl Macro {
    pub fn new(file_id: usize, span: Span, name: String) -> Macro {
        let (file_id, span) = source_lookup(file_id, span);
        Macro {
            file_id,
            span,
            name,
            replace_list: vec![],
            kind: MacroKind::Object,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MacroKind {
    Object,
    Function {
        parameters: Vec<String>,
        has_varparam: bool,
    },
}

impl PartialEq for Macro {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.kind == other.kind
            && self.replace_list == other.replace_list
    }
}

impl Preprocessor {
    ///查找宏, 包括预定义的和用户定义的
    pub fn find_macro(&self, macro_name: &String, span: Span) -> Option<Macro> {
        match macro_name.as_str() {
            "__DATE__" => {
                let nowtime = Local::now();
                //留出日期的位置
                let mut timeformat: Vec<char> =
                    nowtime.format("%h    %Y").to_string().chars().collect();
                //手动对日期进行格式化
                timeformat[5] = ((nowtime.day() % 10) as u8 + b'0') as char;
                if nowtime.day() >= 10 {
                    timeformat[4] = ((nowtime.day() / 10) as u8 + b'0') as char;
                }
                Some(Macro {
                    replace_list: vec![Token::new(
                        self.file_id,
                        span,
                        TokenKind::Text {
                            is_whitespace: false,
                            content: format!("{:?}", timeformat.into_iter().collect::<String>()),
                        },
                    )],
                    kind: MacroKind::Object,
                    ..Macro::new(self.file_id, span, macro_name.to_string())
                })
            }
            "__FILE__" => Some(Macro {
                replace_list: vec![Token::new(
                    self.file_id,
                    span,
                    TokenKind::Text {
                        is_whitespace: false,
                        content: format!("{:?}", self.file_name),
                    },
                )],
                kind: MacroKind::Object,
                ..Macro::new(self.file_id, span, macro_name.to_string())
            }),
            "__TIME__" => Some(Macro {
                replace_list: vec![Token::new(
                    self.file_id,
                    span,
                    TokenKind::Text {
                        is_whitespace: false,
                        content: format!("{:?}", Local::now().format("%H:%M:%S").to_string()),
                    },
                )],
                kind: MacroKind::Object,
                ..Macro::new(self.file_id, span, macro_name.to_string())
            }),
            "__LINE__" => Some(Macro {
                replace_list: vec![Token::new(
                    self.file_id,
                    span,
                    TokenKind::Text {
                        is_whitespace: false,
                        content: format!(
                            "{}",
                            files
                                .lock()
                                .unwrap()
                                .location(self.file_id, span.start().to_usize())
                                .unwrap()
                                .line_number as isize
                                + self.line_offset,
                        ),
                    },
                )],
                kind: MacroKind::Object,
                ..Macro::new(self.file_id, span, macro_name.to_string())
            }),
            "__STDC__" | "__STDC_HOSTED__" | "__STDC_UTF_16__" | "__STDC_UTF_32__" => Some(Macro {
                replace_list: vec![Token::new(
                    self.file_id,
                    span,
                    TokenKind::Text {
                        is_whitespace: false,
                        content: format!("1"),
                    },
                )],
                kind: MacroKind::Object,
                ..Macro::new(self.file_id, span, macro_name.to_string())
            }),
            "__STDC_EMBED_FOUND__" => Some(Macro {
                replace_list: vec![Token::new(
                    self.file_id,
                    span,
                    TokenKind::Text {
                        is_whitespace: false,
                        content: format!("{STDC_EMBED_FOUND}"),
                    },
                )],
                kind: MacroKind::Object,
                ..Macro::new(self.file_id, span, macro_name.to_string())
            }),
            "__STDC_EMBED_NOT_FOUND__" => Some(Macro {
                replace_list: vec![Token::new(
                    self.file_id,
                    span,
                    TokenKind::Text {
                        is_whitespace: false,
                        content: format!("{STDC_EMBED_NOT_FOUND}"),
                    },
                )],
                kind: MacroKind::Object,
                ..Macro::new(self.file_id, span, macro_name.to_string())
            }),
            "__STDC_EMBED_EMPTY__" => Some(Macro {
                replace_list: vec![Token::new(
                    self.file_id,
                    span,
                    TokenKind::Text {
                        is_whitespace: false,
                        content: format!("{STDC_EMBED_EMPTY}"),
                    },
                )],
                kind: MacroKind::Object,
                ..Macro::new(self.file_id, span, macro_name.to_string())
            }),
            "__STDC_VERSION__" => Some(Macro {
                replace_list: vec![Token::new(
                    self.file_id,
                    span,
                    TokenKind::Text {
                        is_whitespace: false,
                        content: format!("202311L"),
                    },
                )],
                kind: MacroKind::Object,
                ..Macro::new(self.file_id, span, macro_name.to_string())
            }),
            _ => {
                if let Some(t) = user_macro.read().unwrap().get(macro_name) {
                    Some((*t).clone())
                } else {
                    None
                }
            }
        }
    }

    pub fn replace_macro(
        &self,
        mut tokens: Vec<Token>,
        expanded_macro: &mut Vec<String>, //已经展开过的宏
    ) -> Result<Vec<Token>, Diagnostic<usize>> {
        let mut need_rescan = true;
        while need_rescan {
            need_rescan = false;

            let mut macro_call_name = Vec::new(); //正在解析的marco_call的名称
            let mut paren_count = Vec::new(); //匹配的paran数量, 每一个嵌套对应一个元素
            let mut arg_num = Vec::new(); //当前匹配到的参数数量

            let old_tokens = tokens.clone();
            tokens.clear();
            for token in old_tokens {
                let span = token.span;
                match token.kind {
                    TokenKind::Identifier { name: macro_name }
                        if !expanded_macro.contains(&macro_name) =>
                    {
                        match self.find_macro(&macro_name, span) {
                            Some(Macro {
                                kind: MacroKind::Function { .. },
                                ..
                            }) => {
                                //手动解析macro call
                                tokens.push(Token::new(
                                    self.file_id,
                                    span,
                                    TokenKind::CallStart {
                                        name: macro_name.clone(),
                                    },
                                ));
                                tokens.push(Token::new(
                                    self.file_id,
                                    span,
                                    TokenKind::Text {
                                        is_whitespace: false,
                                        content: macro_name.clone(),
                                    },
                                ));
                                macro_call_name.push(macro_name.clone());
                                paren_count.push(0);
                                arg_num.push(0);
                                expanded_macro.push(macro_name.clone());
                            }
                            Some(Macro {
                                replace_list,
                                kind: MacroKind::Object { .. },
                                ..
                            }) => {
                                need_rescan = true;
                                tokens.extend(replace_list.to_vec());
                                expanded_macro.push(macro_name.clone());
                            }
                            _ => tokens.push(Token::new(
                                self.file_id,
                                span,
                                TokenKind::Text {
                                    is_whitespace: false,
                                    content: macro_name.clone(),
                                },
                            )),
                        }
                    }
                    TokenKind::Text { ref content, .. }
                        if content == "(" && macro_call_name.len() > 0 =>
                    {
                        *paren_count.last_mut().unwrap() += 1;
                        tokens.push(token.clone());

                        if *paren_count.last().unwrap() == 1 {
                            tokens.push(Token::new(
                                self.file_id,
                                span,
                                TokenKind::CallArgStart {
                                    belong: macro_call_name.last().unwrap().to_string(),
                                    index: *arg_num.last().unwrap(),
                                },
                            ));
                        }
                    }
                    TokenKind::Text { ref content, .. }
                        if content == ")" && macro_call_name.len() > 0 =>
                    {
                        *paren_count.last_mut().unwrap() -= 1;
                        if *paren_count.last().unwrap() == 0 {
                            tokens.push(Token::new(
                                self.file_id,
                                span,
                                TokenKind::CallArgEnd {
                                    belong: macro_call_name.last().unwrap().to_string(),
                                    index: *arg_num.last().unwrap(),
                                },
                            ));

                            paren_count.pop();
                            arg_num.pop();

                            tokens.push(token.clone());
                            tokens.push(Token::new(
                                self.file_id,
                                span,
                                TokenKind::CallEnd {
                                    name: macro_call_name.pop().unwrap(),
                                },
                            ));
                        } else {
                            tokens.push(token.clone());
                        }
                    }
                    TokenKind::Text { ref content, .. }
                        if content == "," && macro_call_name.len() > 0 =>
                    {
                        if *paren_count.last().unwrap() > 1 {
                            //这个','是在参数内部的括号中
                            tokens.push(token.clone());
                        } else {
                            tokens.push(Token::new(
                                self.file_id,
                                span,
                                TokenKind::CallArgEnd {
                                    belong: macro_call_name.last().unwrap().to_string(),
                                    index: *arg_num.last().unwrap(),
                                },
                            ));

                            tokens.push(token.clone());
                            *arg_num.last_mut().unwrap() += 1;

                            tokens.push(Token::new(
                                self.file_id,
                                span,
                                TokenKind::CallArgStart {
                                    belong: macro_call_name.last().unwrap().to_string(),
                                    index: *arg_num.last().unwrap(),
                                },
                            ));
                        }
                    }
                    _ => tokens.push(token.clone()),
                }
            }
            tokens = self.remove_placemarker(tokens, &mut need_rescan, expanded_macro)?;
        }

        Ok(tokens)
    }

    ///用在展开类函数宏的替换列表时
    pub fn expand_funclike_macro(
        &self,
        replace_list: &Vec<Token>,
        parameters: &Vec<String>,
        has_varparam: &bool,
        args: &Vec<Vec<Token>>,
    ) -> Result<Vec<Token>, Diagnostic<usize>> {
        let mut tokens = replace_list.clone();
        let mut unmatch_stringize = 0; //已经存在的StringizeStart的数量
        let mut stringize_param = String::new(); //'#'后面的参数名, 用于确保添加StringizeEnd的是正确的参数
        let mut in_parse_vaopt = false; //是否正在处理__VA_OPT__
        let mut va_opt_paren = 0; //当前__VA_OPT__匹配的括号数量
        let mut i = 0;
        while i < tokens.len() {
            let span = tokens[i].span;
            match &tokens[i].kind {
                TokenKind::Identifier { name } if name == "__VA_ARGS__" => {
                    if !*has_varparam {
                        return Err(Diagnostic::error()
                            .with_message(format!(
                                "'__VA_ARGS__' can only appear in the expansion of a variadic macro"
                            ))
                            .with_label(Label::primary(self.file_id, span)));
                    }

                    tokens.remove(i);

                    let mut vaargs: Vec<Token> = Vec::new();
                    for (i, arg) in args[parameters.len()..].iter().enumerate() {
                        if i != 0 {
                            //解析的时候已经包含了空白字符, 所以这里不需要考虑
                            vaargs.push(Token::new(
                                self.file_id,
                                span,
                                TokenKind::Text {
                                    is_whitespace: false,
                                    content: ",".to_string(),
                                },
                            ));
                        }
                        //对参数的替换在之前就已经完成
                        vaargs.extend(arg.to_vec());
                    }

                    if unmatch_stringize > 0 && stringize_param == "" {
                        vaargs.push(Token::new(self.file_id, span, TokenKind::StringizeEnd));
                        unmatch_stringize -= 1;
                    }

                    tokens.splice(i..i, vaargs);
                }
                TokenKind::Identifier { name } if name == "__VA_OPT__" => {
                    if !*has_varparam {
                        return Err(Diagnostic::error()
                            .with_message(format!(
                                "'__VA_OPT__' can only appear in the expansion of a variadic macro"
                            ))
                            .with_label(Label::primary(self.file_id, span)));
                    }

                    if in_parse_vaopt {
                        return Err(Diagnostic::error()
                            .with_message(format!("'__VA_OPT__' may not appear in a '__VA_OPT__'"))
                            .with_label(Label::primary(self.file_id, span)));
                    }

                    if unmatch_stringize > 0 && stringize_param == "" {
                        stringize_param = "__VA_OPT__".to_string();
                    }

                    in_parse_vaopt = true;
                    tokens[i] = Token::new(
                        self.file_id,
                        span,
                        TokenKind::VaOptStart {
                            keep: args.len() > parameters.len(),
                        },
                    );
                }
                TokenKind::Text { content, .. } if content == "(" && in_parse_vaopt => {
                    va_opt_paren += 1;
                    if va_opt_paren == 1 {
                        //这是__VA_OPT__最外层的括号, 无需保留
                        tokens.remove(i);
                    } else {
                        i += 1;
                    }
                }
                TokenKind::Text { content, .. } if content == ")" && in_parse_vaopt => {
                    va_opt_paren -= 1;
                    if va_opt_paren == 0 {
                        in_parse_vaopt = false;

                        tokens[i] = Token::new(self.file_id, span, TokenKind::VaOptEnd);

                        i += 1;
                        if unmatch_stringize > 0 && stringize_param == "__VA_OPT__" {
                            tokens
                                .insert(i, Token::new(self.file_id, span, TokenKind::StringizeEnd));
                            unmatch_stringize -= 1;
                            i += 1;
                        }
                    } else {
                        i += 1;
                    }
                }
                TokenKind::Identifier { name } if parameters.contains(name) => {
                    //对参数的替换在之前就已经完成
                    let mut replace_arg =
                        args[parameters.iter().position(|x| x == name).unwrap()].to_vec();

                    tokens.remove(i);

                    if unmatch_stringize > 0 && stringize_param == "" {
                        replace_arg.push(Token::new(self.file_id, span, TokenKind::StringizeEnd));
                        unmatch_stringize -= 1;
                    }

                    tokens.splice(i..i, replace_arg);
                }
                TokenKind::Text { content, .. } if content == "#" => {
                    tokens[i] = Token::new(self.file_id, span, TokenKind::StringizeStart);
                    stringize_param = String::new(); //交给参数自己确定
                    unmatch_stringize += 1;
                    i += 1;
                }
                _ => {
                    i += 1;
                }
            }
        }
        Ok(tokens)
    }

    ///以从内到外的顺序移除placemarker(部分token)
    pub fn remove_placemarker(
        &self,
        mut tokens: Vec<Token>,
        need_rescan: &mut bool,
        expanded_macro: &mut Vec<String>,
    ) -> Result<Vec<Token>, Diagnostic<usize>> {
        let mut i = 0;
        while i < tokens.len() {
            let span = tokens[i].span;
            match &tokens[i].clone().kind {
                TokenKind::StringizeStart => {
                    //参与字符串化的placemarker
                    let mut args = Vec::new();

                    tokens.remove(i);

                    while i < tokens.len() {
                        if let TokenKind::StringizeEnd = tokens[i].kind {
                            break;
                        }
                        args.push(tokens.remove(i));
                    }

                    if let TokenKind::StringizeEnd = tokens[i].kind {
                        tokens.remove(i);

                        let mut stringize_tokens =
                            self.remove_placemarker(args, need_rescan, expanded_macro)?;
                        let mut j = 0;

                        //移除空的placemarker, 这会影响对后面空白字符的判断
                        while j < stringize_tokens.len() {
                            if stringize_tokens[j].to_string() == "" {
                                stringize_tokens.remove(j);
                            } else {
                                j += 1;
                            }
                        }

                        j = 0;
                        //将中间的多个空白字符变为一个空格
                        while j < stringize_tokens.len() {
                            if j >= 1
                                && let TokenKind::Text {
                                    is_whitespace: true,
                                    ..
                                } = &stringize_tokens[j].kind
                                && let TokenKind::Text {
                                    is_whitespace: true,
                                    ..
                                } = &stringize_tokens[j - 1].kind
                            {
                                stringize_tokens.remove(j);
                            } else {
                                j += 1;
                            }
                        }

                        //移除两边的空白字符
                        while stringize_tokens.len() > 0 {
                            if let TokenKind::Text {
                                is_whitespace: true,
                                ..
                            } = &stringize_tokens[0].kind
                            {
                                stringize_tokens.remove(0);
                            } else {
                                break;
                            }
                        }
                        while stringize_tokens.len() > 0 {
                            if let TokenKind::Text {
                                is_whitespace: true,
                                ..
                            } = stringize_tokens.last().unwrap().kind
                            {
                                stringize_tokens.pop();
                            } else {
                                break;
                            }
                        }

                        tokens.insert(
                            i,
                            Token::new(self.file_id, span, TokenKind::Stringized(stringize_tokens)),
                        );
                    } else {
                        return Err(Diagnostic::error()
                            .with_message(format!("'#' is not followed by a macro parameter"))
                            .with_label(Label::primary(self.file_id, span)));
                    }
                }
                TokenKind::CallStart { name } => {
                    *need_rescan = true;
                    tokens.remove(i);
                    let mut args = Vec::new();
                    let mut origin = Vec::new();
                    let mut matched = true; //括号是否匹配

                    while i < tokens.len() {
                        match &tokens[i].clone().kind {
                            TokenKind::CallEnd { name: end_name } if *name == *end_name => {
                                break;
                            }
                            TokenKind::CallArgStart {
                                belong: arg_belong, ..
                            } if *arg_belong == *name => {
                                let mut arg = Vec::new();

                                tokens.remove(i);
                                while i < tokens.len() {
                                    if let TokenKind::CallArgEnd {
                                        belong: arg_end_belong,
                                        ..
                                    } = &tokens[i].kind
                                        && *arg_belong == *arg_end_belong
                                    {
                                        break;
                                    }
                                    arg.push(tokens.remove(i));
                                    origin.push(arg.last().unwrap().clone());
                                }
                                if let TokenKind::CallArgEnd {
                                    belong: arg_end_belong,
                                    ..
                                } = &tokens[i].kind
                                    && *arg_belong == *arg_end_belong
                                {
                                    tokens.remove(i);
                                    args.push(self.remove_placemarker(
                                        arg,
                                        need_rescan,
                                        expanded_macro,
                                    )?);
                                } else {
                                    matched = false;
                                }
                            }
                            _ => origin.push(tokens.remove(i)),
                        }
                    }

                    if i < tokens.len()
                        && let TokenKind::CallEnd { name: end_name } = &tokens[i].kind
                        && *name == *end_name
                    {
                        tokens.remove(i);
                    } else {
                        matched = false;
                    }

                    if matched {
                        //对参数进行宏替换
                        let mut j = 0;
                        while j < args.len() {
                            args[j] = self.replace_macro(args[j].clone(), expanded_macro)?;
                            j += 1;
                        }

                        //识别空参数
                        let mut i = 0;
                        while i < args.len() {
                            if args[i].iter().all(|x| {
                                if let TokenKind::Text {
                                    is_whitespace: true,
                                    ..
                                } = x.kind
                                {
                                    true
                                } else {
                                    false
                                }
                            }) {
                                args[i] = vec![];
                            }
                            i += 1;
                        }
                    }

                    if matched
                        && let Some(Macro {
                            replace_list,
                            kind:
                                MacroKind::Function {
                                    parameters,
                                    has_varparam,
                                },
                            ..
                        }) = &self.find_macro(&name, span)
                    {
                        //特殊处理这种情况
                        if parameters.len() == 0 && args.len() == 1 {
                            if args.len() == 1 && args[0].len() == 0 {
                                //没有参数
                                args.pop();
                            }
                        }
                        let mut j = 0;
                        while j < args.len() {
                            let arg = &mut args[j];
                            if arg.len() == 0 {
                                //为了方便之后对##的处理
                                arg.push(Token::new(
                                    self.file_id,
                                    span,
                                    TokenKind::Text {
                                        is_whitespace: false,
                                        content: "".to_string(),
                                    },
                                ));
                            }
                            j += 1;
                        }
                        if (*has_varparam && args.len() >= parameters.len())
                            || (!*has_varparam && args.len() == parameters.len())
                        {
                            tokens.splice(
                                i..i,
                                self.expand_funclike_macro(
                                    replace_list,
                                    parameters,
                                    has_varparam,
                                    &args,
                                )?,
                            );
                        }
                    } else {
                        tokens.splice(i..i, origin);
                    }
                }
                TokenKind::VaOptStart { keep } => {
                    let mut args = Vec::new();
                    tokens.remove(i);
                    while i < tokens.len() {
                        if let TokenKind::VaOptEnd = tokens[i].kind {
                            tokens.remove(i);
                            if *keep {
                                tokens.splice(
                                    i..i,
                                    self.remove_placemarker(args, need_rescan, expanded_macro)?,
                                );
                            }
                            break;
                        }
                        args.push(tokens.remove(i));
                    }
                }
                TokenKind::Contactor => {
                    if i == 0 {
                        return Err(Diagnostic::error()
                            .with_message(format!("'##' cannot appear at start of macro expansion"))
                            .with_label(Label::primary(self.file_id, span)));
                    }
                    i -= 1;
                    //下溢之后就会大于长度
                    while i < tokens.len() {
                        match &tokens[i].kind {
                            TokenKind::Text {
                                is_whitespace: true,
                                ..
                            } => {
                                if i == 0 {
                                    return Err(Diagnostic::error()
                                        .with_message(format!(
                                            "'##' cannot appear at start of macro expansion"
                                        ))
                                        .with_label(Label::primary(self.file_id, span)));
                                }
                                i -= 1;
                            }
                            TokenKind::Contactor => {
                                tokens.remove(i);
                            }
                            _ => {
                                break;
                            }
                        }
                    }
                    let left = tokens.remove(i);

                    //回到原来的位置
                    while i < tokens.len() {
                        if let TokenKind::Contactor = tokens[i].kind {
                            break;
                        }
                        tokens.remove(i);
                    }
                    tokens.remove(i);

                    while i < tokens.len() {
                        match &tokens[i].kind {
                            TokenKind::Text {
                                is_whitespace: true,
                                ..
                            }
                            | TokenKind::Contactor => {
                                tokens.remove(i);
                            }
                            _ => {
                                break;
                            }
                        }
                    }
                    if i == tokens.len() {
                        return Err(Diagnostic::error()
                            .with_message(format!("'##' cannot appear at end of macro expansion"))
                            .with_label(Label::primary(self.file_id, span)));
                    }
                    let right = tokens.remove(i);

                    tokens.insert(
                        i,
                        Token::new(
                            self.file_id,
                            span,
                            TokenKind::Contacted(Box::new(left), Box::new(right)),
                        ),
                    );
                }
                _ => i += 1,
            }
        }
        Ok(tokens)
    }
}
