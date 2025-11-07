///包括了与宏替换相关的一些方法
use super::cmacro::{Macro, PlaceMarker};
use super::{Preprocessor, Rule};
use chrono::{Datelike, Local};
use pest::error::Error;

impl Preprocessor {
    ///查找宏, 包括预定义的和用户定义的
    pub fn find_macro(&self, macro_name: &String) -> Option<Macro> {
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
                    )],
                })
            }
            "__FILE__" => Some(Macro::Object {
                name: "__FILE__".to_string(),
                replace_list: vec![PlaceMarker::Text(format!("{:?}", self.file_path), false)],
            }),
            "__TIME__" => Some(Macro::Object {
                name: "__TIME__".to_string(),
                replace_list: vec![PlaceMarker::Text(
                    format!("{:?}", Local::now().format("%H:%M:%S").to_string()),
                    false,
                )],
            }),
            //TODO __LINE__
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
                    PlaceMarker::Identifier(macro_name) => {
                        if let Some(Macro::Function {
                            name: _,
                            parameters: _,
                            has_varparam: _,
                            replace_list: _,
                        }) = self.find_macro(macro_name)
                        {
                            //手动解析macro call
                            placemarkers.push(PlaceMarker::CallStart(macro_name.clone()));
                            placemarkers.push(PlaceMarker::Text(macro_name.clone(), false));
                            macro_call_name.push(macro_name.clone());
                            paren_count.push(0);
                            arg_num.push(0);
                        } else if let Some(Macro::Object {
                            name: _,
                            replace_list,
                        }) = self.find_macro(macro_name)
                        {
                            need_rescan = true;
                            placemarkers.extend(replace_list.to_vec());
                        } else {
                            placemarkers.push(PlaceMarker::Text(macro_name.clone(), false));
                        }
                    }
                    PlaceMarker::Text(t, _) if t == "(" && macro_call_name.len() > 0 => {
                        *paren_count.last_mut().unwrap() += 1;
                        placemarkers.push(placemarker.clone());

                        if *paren_count.last().unwrap() == 1 {
                            placemarkers.push(PlaceMarker::CallArgStart(
                                macro_call_name.last().unwrap().to_string(),
                                *arg_num.last().unwrap(),
                            ));
                        }
                    }
                    PlaceMarker::Text(t, _) if t == ")" && macro_call_name.len() > 0 => {
                        *paren_count.last_mut().unwrap() -= 1;
                        if *paren_count.last().unwrap() == 0 {
                            placemarkers.push(PlaceMarker::CallArgEnd(
                                macro_call_name.last().unwrap().to_string(),
                                *arg_num.last().unwrap(),
                            ));

                            paren_count.pop();
                            arg_num.pop();

                            placemarkers.push(placemarker.clone());
                            placemarkers.push(PlaceMarker::CallEnd(macro_call_name.pop().unwrap()));
                        } else {
                            placemarkers.push(placemarker.clone());
                        }
                    }
                    PlaceMarker::Text(t, _) if t == "," && macro_call_name.len() > 0 => {
                        if *paren_count.last().unwrap() > 1 {
                            //这个','是在参数内部的括号中
                            placemarkers.push(placemarker.clone());
                        } else {
                            placemarkers.push(PlaceMarker::CallArgEnd(
                                macro_call_name.last().unwrap().to_string(),
                                *arg_num.last().unwrap(),
                            ));

                            placemarkers.push(placemarker.clone());
                            *arg_num.last_mut().unwrap() += 1;

                            placemarkers.push(PlaceMarker::CallArgStart(
                                macro_call_name.last().unwrap().to_string(),
                                *arg_num.last().unwrap(),
                            ));
                        }
                    }
                    _ => placemarkers.push(placemarker),
                }
            }

            placemarkers = self.remove_placemarker(placemarkers, &mut need_rescan)?;
        }

        Ok(placemarkers)
    }

    ///用在替换类函数宏的替换列表时
    pub fn replace_macro_with_args(
        &self,
        replace_list: &Vec<PlaceMarker>,
        parameters: &Vec<String>,
        has_varparam: &bool,
        args: &Vec<Vec<PlaceMarker>>,
    ) -> Result<Vec<PlaceMarker>, Error<Rule>> {
        let mut placemarkers = replace_list.clone();
        let mut unmatch_stringize = 0; //已经存在的StringizeStart的数量
        let mut in_parse_vaopt = false; //是否正在处理__VA_OPT__
        let mut va_opt_paren = 0; //当前__VA_OPT__匹配的括号

        let mut i = 0;
        while i < placemarkers.len() {
            let placemarker = &placemarkers[i];
            match placemarker {
                PlaceMarker::Identifier(name) if name == "__VA_ARGS__" && *has_varparam => {
                    placemarkers.remove(i);

                    let mut vaargs = Vec::new();
                    for (j, arg) in args[parameters.len()..].iter().enumerate() {
                        if j != 0 {
                            //解析的时候已经包含了空白字符, 所以这里不需要考虑
                            vaargs.push(PlaceMarker::Text(",".to_string(), false));
                        }
                        //对参数的替换在之前就已经完成
                        vaargs.extend(arg.to_vec());
                    }

                    if unmatch_stringize > 0 {
                        vaargs.push(PlaceMarker::StringizeEnd);
                        unmatch_stringize -= 1;
                    }

                    placemarkers.splice(i..i, vaargs);
                }
                PlaceMarker::Identifier(name) if name == "__VA_OPT__" && *has_varparam => {
                    in_parse_vaopt = true;
                    placemarkers[i] = PlaceMarker::VaOptStart(args.len() > parameters.len());
                }
                PlaceMarker::Text(t, _) if t == "(" && in_parse_vaopt => {
                    va_opt_paren += 1;
                    if va_opt_paren == 1 {
                        //这是__VA_OPT__最外层的括号
                        placemarkers.remove(i);
                    } else {
                        i += 1;
                    }
                }
                PlaceMarker::Text(t, _) if t == ")" && in_parse_vaopt => {
                    va_opt_paren -= 1;
                    if va_opt_paren == 0 {
                        in_parse_vaopt = false;
                        placemarkers[i] = PlaceMarker::VaOptEnd;
                        i += 1;
                        if unmatch_stringize > 0 {
                            placemarkers.insert(i, PlaceMarker::StringizeEnd);
                            unmatch_stringize -= 1;
                            i += 1;
                        }
                    } else {
                        i += 1;
                    }
                }
                PlaceMarker::Identifier(name) if parameters.contains(name) => {
                    //对参数的替换在之前就已经完成
                    let mut replace_arg =
                        args[parameters.iter().position(|x| x == name).unwrap()].to_vec();

                    placemarkers.remove(i);

                    if unmatch_stringize > 0 {
                        replace_arg.push(PlaceMarker::StringizeEnd);
                        unmatch_stringize -= 1;
                    }

                    placemarkers.splice(i..i, replace_arg);
                }
                PlaceMarker::Text(text, _) if text == "#" => {
                    placemarkers[i] = PlaceMarker::StringizeStart;
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
    ) -> Result<Vec<PlaceMarker>, Error<Rule>> {
        let mut i = 0;
        while i < placemarkers.len() {
            if let PlaceMarker::StringizeStart = placemarkers[i] {
                //参与字符串化的placemarker
                let mut args = Vec::new();
                placemarkers.remove(i);
                while i < placemarkers.len() {
                    if let PlaceMarker::StringizeEnd = placemarkers[i] {
                        placemarkers.remove(i);

                        let mut stringize_placemarker =
                            self.remove_placemarker(args, need_rescan)?;
                        let mut j = 1;
                        //将中间的多个空白字符变为一个
                        //两边的空白字符在下面用trim消除
                        while j < stringize_placemarker.len() {
                            if let PlaceMarker::Text(_, true) = &stringize_placemarker[j]
                                && let PlaceMarker::Text(_, true) = &stringize_placemarker[j - 1]
                            {
                                stringize_placemarker.remove(j);
                            } else {
                                j += 1;
                            }
                        }

                        placemarkers.insert(
                            i,
                            PlaceMarker::Text(
                                format!(
                                    "{:?}",
                                    PlaceMarker::vec_tostring(stringize_placemarker).trim()
                                ),
                                false,
                            ),
                        );
                        break;
                    }
                    args.push(placemarkers.remove(i));
                }
            } else if let PlaceMarker::CallStart(name) = placemarkers[i].clone() {
                *need_rescan = true;
                placemarkers.remove(i);
                let mut args = Vec::new();
                let mut origin = Vec::new();

                while i < placemarkers.len() {
                    if let PlaceMarker::CallEnd(end_name) = placemarkers[i].clone()
                        && name == end_name
                    {
                        placemarkers.remove(i);
                        break;
                    } else if let PlaceMarker::CallArgStart(arg_belong, _index) =
                        placemarkers[i].clone()
                        && arg_belong == name
                    {
                        let mut arg = Vec::new();

                        placemarkers.remove(i);
                        while i < placemarkers.len() {
                            if let PlaceMarker::CallArgEnd(arg_end_belong, _index) =
                                &placemarkers[i]
                                && arg_belong == *arg_end_belong
                            {
                                placemarkers.remove(i);
                                break;
                            }
                            arg.push(placemarkers.remove(i));
                            origin.push(arg.last().unwrap().clone());
                        }
                        args.push(self.remove_placemarker(arg, need_rescan)?);
                    } else {
                        origin.push(placemarkers.remove(i));
                    }
                }

                //对参数进行宏替换
                let mut j = 0;
                while j < args.len() {
                    args[j] = self.replace_macro(args[j].clone())?;
                    j += 1;
                }

                if args.len() == 1 && args[0].len() == 0 {
                    //没有参数
                    args.pop();
                } else {
                    for arg in args.iter_mut() {
                        if arg.len() == 0 {
                            //为了方便之后对##的处理
                            arg.push(PlaceMarker::Text(String::new(), false));
                        }
                    }
                }

                if let Some(Macro::Function {
                    name: _,
                    parameters,
                    has_varparam,
                    replace_list,
                }) = &self.find_macro(&name)
                    && ((*has_varparam && args.len() >= parameters.len())
                        || (!*has_varparam && args.len() == parameters.len()))
                {
                    placemarkers.splice(
                        i..i,
                        self.replace_macro_with_args(
                            replace_list,
                            parameters,
                            has_varparam,
                            &args,
                        )?,
                    );
                } else {
                    placemarkers.splice(i..i, origin);
                }
            } else if let PlaceMarker::VaOptStart(keep) = placemarkers[i] {
                let mut args = Vec::new();
                placemarkers.remove(i);
                while i < placemarkers.len() {
                    if let PlaceMarker::VaOptEnd = placemarkers[i] {
                        placemarkers.remove(i);
                        if keep {
                            placemarkers.splice(i..i, self.remove_placemarker(args, need_rescan)?);
                        }
                        break;
                    }
                    args.push(placemarkers.remove(i));
                }
            } else if let PlaceMarker::Contactor = placemarkers[i] {
                /*
                Contactor只会在replace_macro_with_args中出现,
                而对macoo call的处理在replace_macro中,
                所以Contactor前后不会碰到与macro call相关的placemarker
                */
                i -= 1;
                //下溢之后就会大于长度
                while i < placemarkers.len() {
                    //找到preprocessing token
                    match &placemarkers[i] {
                        PlaceMarker::Text(_, false) => {
                            break;
                        }
                        _ => {
                            i -= 1;
                        }
                    }
                }
                let left = placemarkers.remove(i);

                //回到原来的位置
                while i < placemarkers.len() {
                    if let PlaceMarker::Contactor = placemarkers[i] {
                        break;
                    }
                    placemarkers.remove(i);
                }
                placemarkers.remove(i);

                while i < placemarkers.len() {
                    //找到preprocessing token
                    match &placemarkers[i] {
                        PlaceMarker::Text(_, false) => {
                            break;
                        }
                        _ => {
                            placemarkers.remove(i);
                        }
                    }
                }
                let right = placemarkers.remove(i);

                placemarkers.insert(i, PlaceMarker::Text(format!("{left}{right}"), false));
            } else {
                i += 1;
            }
        }
        Ok(placemarkers)
    }
}
