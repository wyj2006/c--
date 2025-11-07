///包括了与宏替换与展开相关的一些方法
use super::cmacro::{Macro, PlaceMarker, PlaceMarkerSpan};
use super::{Preprocessor, Rule};
use chrono::{Datelike, Local};
use pest::Span;
use pest::error::{Error, ErrorVariant};

impl Preprocessor {
    ///查找宏, 包括预定义的和用户定义的
    pub fn find_macro(&self, macro_name: &String, span: Span) -> Option<Macro> {
        match macro_name.as_str() {
            "__DATE__" => {
                let nowtime = Local::now();
                //留出日期的位置
                let mut timeformat: Vec<char> =
                    nowtime.format("%h    %Y").to_string().chars().collect();
                timeformat[5] = ((nowtime.day() % 10) as u8 + b'0') as char;
                if nowtime.day() >= 10 {
                    timeformat[4] = ((nowtime.day() / 10) as u8 + b'0') as char;
                }
                Some(Macro::Object {
                    name: "__DATE__".to_string(),
                    replace_list: vec![PlaceMarker::Text(
                        format!("{:?}", timeformat.into_iter().collect::<String>()),
                        false,
                        PlaceMarkerSpan::new_from_span(span),
                    )],
                })
            }
            "__FILE__" => Some(Macro::Object {
                name: "__FILE__".to_string(),
                replace_list: vec![PlaceMarker::Text(
                    format!("{:?}", self.file_path),
                    false,
                    PlaceMarkerSpan::new_from_span(span),
                )],
            }),
            "__TIME__" => Some(Macro::Object {
                name: "__TIME__".to_string(),
                replace_list: vec![PlaceMarker::Text(
                    format!("{:?}", Local::now().format("%H:%M:%S").to_string()),
                    false,
                    PlaceMarkerSpan::new_from_span(span),
                )],
            }),
            "__LINE__" => Some(Macro::Object {
                name: "__LINE__".to_string(),
                replace_list: vec![PlaceMarker::Text(
                    format!("{}", span.start_pos().line_col().0),
                    false,
                    PlaceMarkerSpan::new_from_span(span),
                )],
            }),
            "__STDC__"
            | "__STDC_EMBED_FOUND__"
            | "__STDC_HOSTED__"
            | "__STDC_UTF_16__"
            | "__STDC_UTF_32__" => Some(Macro::Object {
                name: macro_name.clone(),
                replace_list: vec![PlaceMarker::Text(
                    format!("1"),
                    false,
                    PlaceMarkerSpan::new_from_span(span),
                )],
            }),
            "__STDC_EMBED_NOT_FOUND__" => Some(Macro::Object {
                name: "__STDC_EMBED_NOT_FOUND__".to_string(),
                replace_list: vec![PlaceMarker::Text(
                    format!("0"),
                    false,
                    PlaceMarkerSpan::new_from_span(span),
                )],
            }),
            "__STDC_EMBED_EMPTY__" => Some(Macro::Object {
                name: "__STDC_EMBED_EMPTY__".to_string(),
                replace_list: vec![PlaceMarker::Text(
                    format!("2"),
                    false,
                    PlaceMarkerSpan::new_from_span(span),
                )],
            }),
            "__STDC_VERSION__" => Some(Macro::Object {
                name: "__STDC_VERSION__".to_string(),
                replace_list: vec![PlaceMarker::Text(
                    format!("202311L"),
                    false,
                    PlaceMarkerSpan::new_from_span(span),
                )],
            }),
            _ => {
                if let Some(t) = self.user_macro.get(macro_name) {
                    Some((*t).clone())
                } else {
                    None
                }
            }
        }
    }

    pub fn replace_macro(
        &self,
        mut placemarkers: Vec<PlaceMarker>,
        expanded_macro: &mut Vec<String>, //已经展开过的宏
    ) -> Result<Vec<PlaceMarker>, Error<Rule>> {
        let mut need_rescan = true;
        while need_rescan {
            need_rescan = false;

            let mut macro_call_name = Vec::new(); //正在解析的marco_call的名称
            let mut paren_count = Vec::new(); //匹配的paran数量, 每一个嵌套对应一个元素
            let mut arg_num = Vec::new(); //当前匹配到的参数数量

            let placemarkers_clone = placemarkers.clone();
            placemarkers.clear();
            for placemarker in placemarkers_clone {
                match &placemarker {
                    PlaceMarker::Identifier(macro_name, placemarker_span)
                        if !expanded_macro.contains(macro_name) =>
                    {
                        if let Some(Macro::Function {
                            name: _,
                            parameters: _,
                            has_varparam: _,
                            replace_list: _,
                        }) = self.find_macro(macro_name, placemarker_span.to_span().unwrap())
                        {
                            //手动解析macro call
                            placemarkers.push(PlaceMarker::CallStart(
                                macro_name.clone(),
                                placemarker_span.clone(),
                            ));
                            placemarkers.push(PlaceMarker::Text(
                                macro_name.clone(),
                                false,
                                placemarker_span.clone(),
                            ));
                            macro_call_name.push(macro_name.clone());
                            paren_count.push(0);
                            arg_num.push(0);
                            expanded_macro.push(macro_name.clone());
                        } else if let Some(Macro::Object {
                            name: _,
                            replace_list,
                        }) =
                            self.find_macro(macro_name, placemarker_span.to_span().unwrap())
                        {
                            need_rescan = true;
                            placemarkers.extend(replace_list.to_vec());
                            expanded_macro.push(macro_name.clone());
                        } else {
                            placemarkers.push(PlaceMarker::Text(
                                macro_name.clone(),
                                false,
                                placemarker_span.clone(),
                            ));
                        }
                    }
                    PlaceMarker::Text(t, _, placemarker_span)
                        if t == "(" && macro_call_name.len() > 0 =>
                    {
                        *paren_count.last_mut().unwrap() += 1;
                        placemarkers.push(placemarker.clone());

                        if *paren_count.last().unwrap() == 1 {
                            placemarkers.push(PlaceMarker::CallArgStart(
                                macro_call_name.last().unwrap().to_string(),
                                *arg_num.last().unwrap(),
                                placemarker_span.clone(),
                            ));
                        }
                    }
                    PlaceMarker::Text(t, _, placemarker_span)
                        if t == ")" && macro_call_name.len() > 0 =>
                    {
                        *paren_count.last_mut().unwrap() -= 1;
                        if *paren_count.last().unwrap() == 0 {
                            placemarkers.push(PlaceMarker::CallArgEnd(
                                macro_call_name.last().unwrap().to_string(),
                                *arg_num.last().unwrap(),
                                placemarker_span.clone(),
                            ));

                            paren_count.pop();
                            arg_num.pop();

                            placemarkers.push(placemarker.clone());
                            placemarkers.push(PlaceMarker::CallEnd(
                                macro_call_name.pop().unwrap(),
                                placemarker_span.clone(),
                            ));
                        } else {
                            placemarkers.push(placemarker.clone());
                        }
                    }
                    PlaceMarker::Text(t, _, placemarker_span)
                        if t == "," && macro_call_name.len() > 0 =>
                    {
                        if *paren_count.last().unwrap() > 1 {
                            //这个','是在参数内部的括号中
                            placemarkers.push(placemarker.clone());
                        } else {
                            placemarkers.push(PlaceMarker::CallArgEnd(
                                macro_call_name.last().unwrap().to_string(),
                                *arg_num.last().unwrap(),
                                placemarker_span.clone(),
                            ));

                            placemarkers.push(placemarker.clone());
                            *arg_num.last_mut().unwrap() += 1;

                            placemarkers.push(PlaceMarker::CallArgStart(
                                macro_call_name.last().unwrap().to_string(),
                                *arg_num.last().unwrap(),
                                placemarker_span.clone(),
                            ));
                        }
                    }
                    _ => placemarkers.push(placemarker),
                }
            }

            placemarkers =
                self.remove_placemarker(placemarkers, &mut need_rescan, expanded_macro)?;
        }

        Ok(placemarkers)
    }

    ///用在展开类函数宏的替换列表时
    pub fn expand_funclike_macro(
        &self,
        replace_list: &Vec<PlaceMarker>,
        parameters: &Vec<String>,
        has_varparam: &bool,
        args: &Vec<Vec<PlaceMarker>>,
    ) -> Result<Vec<PlaceMarker>, Error<Rule>> {
        let mut placemarkers = replace_list.clone();
        let mut unmatch_stringize = 0; //已经存在的StringizeStart的数量
        let mut stringize_param = String::new(); //'#'后面的参数名, 用于确保添加StringizeEnd的是正确的参数
        let mut in_parse_vaopt = false; //是否正在处理__VA_OPT__
        let mut va_opt_paren = 0; //当前__VA_OPT__匹配的括号

        let mut i = 0;
        while i < placemarkers.len() {
            let placemarker = &placemarkers[i];
            match placemarker {
                PlaceMarker::Identifier(name, placemarker_span) if name == "__VA_ARGS__" => {
                    if !*has_varparam {
                        return Err(Error::new_from_span(
                            ErrorVariant::CustomError {
                                message: format!(
                                    "'__VA_ARGS__' can only appear in the expansion of a variadic macro"
                                ),
                            },
                            placemarker_span.to_span().unwrap(),
                        ));
                    }

                    let placemarker_span = placemarker_span.clone();
                    placemarkers.remove(i);

                    let mut vaargs: Vec<PlaceMarker> = Vec::new();
                    for (j, arg) in args[parameters.len()..].iter().enumerate() {
                        if j != 0 {
                            //解析的时候已经包含了空白字符, 所以这里不需要考虑
                            vaargs.push(PlaceMarker::Text(
                                ",".to_string(),
                                false,
                                match vaargs.last() {
                                    Some(t) => t.span().clone(),
                                    None => placemarker_span.clone(),
                                },
                            ));
                        }
                        //对参数的替换在之前就已经完成
                        vaargs.extend(arg.to_vec());
                    }

                    if unmatch_stringize > 0 && stringize_param == "" {
                        vaargs.push(PlaceMarker::StringizeEnd(match vaargs.last() {
                            Some(t) => t.span().clone(),
                            None => placemarker_span.clone(),
                        }));
                        unmatch_stringize -= 1;
                    }

                    placemarkers.splice(i..i, vaargs);
                }
                PlaceMarker::Identifier(name, placemarker_span) if name == "__VA_OPT__" => {
                    if !*has_varparam {
                        return Err(Error::new_from_span(
                            ErrorVariant::CustomError {
                                message: format!(
                                    "'__VA_OPT__' can only appear in the expansion of a variadic macro"
                                ),
                            },
                            placemarker_span.to_span().unwrap(),
                        ));
                    }

                    if in_parse_vaopt {
                        return Err(Error::new_from_span(
                            ErrorVariant::CustomError {
                                message: format!("'__VA_OPT__' may not appear in a '__VA_OPT__'"),
                            },
                            placemarker_span.to_span().unwrap(),
                        ));
                    }

                    if unmatch_stringize > 0 && stringize_param == "" {
                        stringize_param = "__VA_OPT__".to_string();
                    }

                    in_parse_vaopt = true;
                    placemarkers[i] = PlaceMarker::VaOptStart(
                        args.len() > parameters.len(),
                        placemarker_span.clone(),
                    );
                }
                PlaceMarker::Text(t, _, _) if t == "(" && in_parse_vaopt => {
                    va_opt_paren += 1;
                    if va_opt_paren == 1 {
                        //这是__VA_OPT__最外层的括号
                        placemarkers.remove(i);
                    } else {
                        i += 1;
                    }
                }
                PlaceMarker::Text(t, _, placemarker_span) if t == ")" && in_parse_vaopt => {
                    va_opt_paren -= 1;
                    if va_opt_paren == 0 {
                        in_parse_vaopt = false;

                        let placemarker_span = placemarker_span.clone();
                        placemarkers[i] = PlaceMarker::VaOptEnd(placemarker_span.clone());

                        i += 1;
                        if unmatch_stringize > 0 && stringize_param == "__VA_OPT__" {
                            placemarkers
                                .insert(i, PlaceMarker::StringizeEnd(placemarker_span.clone()));
                            unmatch_stringize -= 1;
                            i += 1;
                        }
                    } else {
                        i += 1;
                    }
                }
                PlaceMarker::Identifier(name, placemarker_span) if parameters.contains(name) => {
                    //对参数的替换在之前就已经完成
                    let mut replace_arg =
                        args[parameters.iter().position(|x| x == name).unwrap()].to_vec();

                    let placemarker_span = placemarker_span.clone();
                    placemarkers.remove(i);

                    if unmatch_stringize > 0 && stringize_param == "" {
                        replace_arg.push(PlaceMarker::StringizeEnd(match replace_arg.last() {
                            Some(t) => t.span().clone(),
                            None => placemarker_span.clone(),
                        }));
                        unmatch_stringize -= 1;
                    }

                    placemarkers.splice(i..i, replace_arg);
                }
                PlaceMarker::Text(text, _, placemarker_span) if text == "#" => {
                    placemarkers[i] = PlaceMarker::StringizeStart(placemarker_span.clone());
                    stringize_param = String::new(); //交给参数自己确定
                    unmatch_stringize += 1;
                    i += 1;
                }
                _ => {
                    i += 1;
                }
            }
        }
        Ok(placemarkers)
    }

    ///以从内到外的顺序移除部分类型的placemarker
    pub fn remove_placemarker(
        &self,
        mut placemarkers: Vec<PlaceMarker>,
        need_rescan: &mut bool,
        expanded_macro: &mut Vec<String>,
    ) -> Result<Vec<PlaceMarker>, Error<Rule>> {
        let mut i = 0;
        while i < placemarkers.len() {
            if let PlaceMarker::StringizeStart(placemarker_span) = &placemarkers[i] {
                //参与字符串化的placemarker
                let mut args = Vec::new();

                let placemarker_span = placemarker_span.clone();
                placemarkers.remove(i);

                while i < placemarkers.len() {
                    if let PlaceMarker::StringizeEnd(_) = placemarkers[i] {
                        break;
                    }
                    args.push(placemarkers.remove(i));
                }

                if let PlaceMarker::StringizeEnd(_) = placemarkers[i] {
                    placemarkers.remove(i);

                    let mut stringize_placemarker =
                        self.remove_placemarker(args, need_rescan, expanded_macro)?;
                    let mut j = 0;

                    //移除空的placemarker, 这会影响对后面空白字符的判断
                    while j < stringize_placemarker.len() {
                        if stringize_placemarker[j].to_string() == "" {
                            stringize_placemarker.remove(j);
                        } else {
                            j += 1;
                        }
                    }

                    j = 0;
                    //将中间的多个空白字符变为一个空格
                    while j < stringize_placemarker.len() {
                        if j >= 1
                            && let PlaceMarker::Text(_, true, _) = &stringize_placemarker[j]
                            && let PlaceMarker::Text(_, true, _) = &stringize_placemarker[j - 1]
                        {
                            stringize_placemarker.remove(j);
                        }
                        if let PlaceMarker::Text(_, true, placemarker_span) =
                            &stringize_placemarker[j]
                        {
                            stringize_placemarker[j] =
                                PlaceMarker::Text(" ".to_string(), true, placemarker_span.clone());
                            j += 1;
                        } else {
                            j += 1;
                        }
                    }

                    //移除两边的空白字符
                    while stringize_placemarker.len() > 0 {
                        if let PlaceMarker::Text(_, true, _) = stringize_placemarker[0] {
                            stringize_placemarker.remove(0);
                        } else {
                            break;
                        }
                    }
                    while stringize_placemarker.len() > 0 {
                        if let PlaceMarker::Text(_, true, _) = stringize_placemarker.last().unwrap()
                        {
                            stringize_placemarker.pop();
                        } else {
                            break;
                        }
                    }

                    placemarkers.insert(
                        i,
                        PlaceMarker::Stringized(stringize_placemarker, placemarker_span.clone()),
                    );
                } else {
                    return Err(Error::new_from_span(
                        ErrorVariant::CustomError {
                            message: format!("'#' is not followed by a macro parameter"),
                        },
                        placemarker_span.to_span().unwrap(),
                    ));
                }
            } else if let PlaceMarker::CallStart(name, placemarker_span) = placemarkers[i].clone() {
                *need_rescan = true;
                placemarkers.remove(i);
                let mut args = Vec::new();
                let mut origin = Vec::new();
                let mut matched = true; //括号是否匹配

                while i < placemarkers.len() {
                    if let PlaceMarker::CallEnd(end_name, _) = placemarkers[i].clone()
                        && name == end_name
                    {
                        break;
                    } else if let PlaceMarker::CallArgStart(arg_belong, _index, _) =
                        placemarkers[i].clone()
                        && arg_belong == name
                    {
                        let mut arg = Vec::new();

                        placemarkers.remove(i);
                        while i < placemarkers.len() {
                            if let PlaceMarker::CallArgEnd(arg_end_belong, _index, _) =
                                &placemarkers[i]
                                && arg_belong == *arg_end_belong
                            {
                                break;
                            }
                            arg.push(placemarkers.remove(i));
                            origin.push(arg.last().unwrap().clone());
                        }
                        if let PlaceMarker::CallArgEnd(arg_end_belong, _, _) = &placemarkers[i]
                            && arg_belong == *arg_end_belong
                        {
                            placemarkers.remove(i);
                            args.push(self.remove_placemarker(arg, need_rescan, expanded_macro)?);
                        } else {
                            matched = false;
                        }
                    } else {
                        origin.push(placemarkers.remove(i));
                    }
                }

                if let PlaceMarker::CallEnd(end_name, _) = placemarkers[i].clone()
                    && name == end_name
                {
                    placemarkers.remove(i);
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
                    j = 0;
                    while j < args.len() {
                        if args[j].iter().all(|x| {
                            if let PlaceMarker::Text(_, true, _) = x {
                                true
                            } else {
                                false
                            }
                        }) {
                            args[j] = vec![];
                        }
                        j += 1;
                    }

                    if args.len() == 1 && args[0].len() == 0 {
                        //没有参数
                        args.pop();
                    } else {
                        let mut j = 0;
                        while j < args.len() {
                            let placemarker_span = if j == 0 {
                                placemarker_span.clone()
                            } else {
                                match args[j - 1].last() {
                                    Some(t) => t.span().clone(),
                                    None => placemarker_span.clone(),
                                }
                            };
                            let arg = &mut args[j];
                            if arg.len() == 0 {
                                //为了方便之后对##的处理
                                arg.push(PlaceMarker::Text(String::new(), false, placemarker_span));
                            }
                            j += 1;
                        }
                    }
                }

                if matched
                    && let Some(Macro::Function {
                        name: _,
                        parameters,
                        has_varparam,
                        replace_list,
                    }) = &self.find_macro(&name, placemarker_span.to_span().unwrap())
                    && ((*has_varparam && args.len() >= parameters.len())
                        || (!*has_varparam && args.len() == parameters.len()))
                {
                    placemarkers.splice(
                        i..i,
                        self.expand_funclike_macro(replace_list, parameters, has_varparam, &args)?,
                    );
                } else {
                    placemarkers.splice(i..i, origin);
                }
            } else if let PlaceMarker::VaOptStart(keep, _) = placemarkers[i] {
                let mut args = Vec::new();
                placemarkers.remove(i);
                while i < placemarkers.len() {
                    if let PlaceMarker::VaOptEnd(_) = placemarkers[i] {
                        placemarkers.remove(i);
                        if keep {
                            placemarkers.splice(
                                i..i,
                                self.remove_placemarker(args, need_rescan, expanded_macro)?,
                            );
                        }
                        break;
                    }
                    args.push(placemarkers.remove(i));
                }
            } else if let PlaceMarker::Contactor(placemarker_span) = &placemarkers[i] {
                /*
                Contactor只会在replace_macro_with_args中出现,
                而对macoo call的处理在replace_macro中,
                所以Contactor前后不会碰到与macro call相关的placemarker
                */
                let placemarker_span = placemarker_span.clone();
                i -= 1;
                //下溢之后就会大于长度
                while i < placemarkers.len() {
                    //找到preprocessing token
                    match &placemarkers[i] {
                        PlaceMarker::Text(_, true, _) => {
                            if i == 0 {
                                return Err(Error::new_from_span(
                                    ErrorVariant::CustomError {
                                        message: format!(
                                            "'##' cannot appear at start of macro expansion"
                                        ),
                                    },
                                    placemarker_span.to_span().unwrap(),
                                ));
                            }
                            i -= 1;
                        }
                        _ => {
                            break;
                        }
                    }
                }
                let left = placemarkers.remove(i);

                //回到原来的位置
                while i < placemarkers.len() {
                    if let PlaceMarker::Contactor(_) = placemarkers[i] {
                        break;
                    }
                    placemarkers.remove(i);
                }
                placemarkers.remove(i);

                while i < placemarkers.len() {
                    //找到preprocessing token
                    match &placemarkers[i] {
                        PlaceMarker::Text(_, true, _) => {
                            placemarkers.remove(i);
                        }
                        _ => {
                            break;
                        }
                    }
                }
                if placemarkers.len() == 0 {
                    return Err(Error::new_from_span(
                        ErrorVariant::CustomError {
                            message: format!("'##' cannot appear at end of macro expansion"),
                        },
                        placemarker_span.to_span().unwrap(),
                    ));
                }
                let right = placemarkers.remove(i);

                placemarkers.insert(
                    i,
                    PlaceMarker::Contacted(
                        Box::new(left),
                        Box::new(right),
                        placemarker_span.clone(),
                    ),
                );
            } else {
                i += 1;
            }
        }
        Ok(placemarkers)
    }
}
