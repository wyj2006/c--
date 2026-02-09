use codespan::Span;
use codespan_reporting::files::{Error, Files, SimpleFile};
use indexmap::IndexMap;
use std::{cell::RefCell, cmp::min, ops::Range, rc::Rc};

use crate::{
    ast::expr::{Expr, ExprKind},
    files,
    preprocessor::token::{Token, to_string},
};

pub struct FileMap {
    pub files: Vec<SimpleFile<String, String>>,
    pub mappings: IndexMap<(usize, Span), (usize, Span)>,
}

impl FileMap {
    pub fn new() -> FileMap {
        FileMap {
            files: Vec::new(),
            mappings: IndexMap::new(),
        }
    }

    pub fn add(&mut self, name: String, source: String) -> usize {
        let file_id = self.files.len();
        self.files.push(SimpleFile::new(name, source));
        file_id
    }

    pub fn get(&self, file_id: usize) -> Result<&SimpleFile<String, String>, Error> {
        self.files.get(file_id).ok_or(Error::FileMissing)
    }
}

impl Files<'_> for FileMap {
    type FileId = usize;
    type Name = String;
    type Source = String;

    fn name(&self, file_id: usize) -> Result<String, Error> {
        Ok(self.get(file_id)?.name().clone())
    }

    fn source(&self, file_id: usize) -> Result<String, Error> {
        Ok(self.get(file_id)?.source().clone())
    }

    fn line_index(&self, file_id: usize, byte_index: usize) -> Result<usize, Error> {
        self.get(file_id)?.line_index((), byte_index)
    }

    fn line_range(&self, file_id: usize, line_index: usize) -> Result<Range<usize>, Error> {
        self.get(file_id)?.line_range((), line_index)
    }
}

pub trait CorrectSpan {
    fn correct(&self, offset: usize);
}

impl CorrectSpan for Rc<RefCell<Expr>> {
    fn correct(&self, offset: usize) {
        let span = self.borrow().span;
        self.borrow_mut().span = Span::new(
            (span.start().to_usize() + offset) as u32,
            (span.end().to_usize() + offset) as u32,
        );
        match &self.borrow().kind {
            ExprKind::BinOp { left, right, .. } => {
                left.correct(offset);
                right.correct(offset);
            }
            ExprKind::Cast { target, .. } => target.correct(offset),
            ExprKind::Conditional {
                condition,
                true_expr,
                false_expr,
            } => {
                condition.correct(offset);
                true_expr.correct(offset);
                false_expr.correct(offset);
            }
            ExprKind::FunctionCall { target, arguments } => {
                target.correct(offset);
                for argument in arguments {
                    argument.correct(offset);
                }
            }
            ExprKind::GenericSelection { control_expr, .. } => control_expr.correct(offset),
            ExprKind::MemberAccess { target, .. } => target.correct(offset),
            ExprKind::SizeOf {
                expr: Some(expr), ..
            } => expr.correct(offset),
            ExprKind::Subscript { target, index } => {
                target.correct(offset);
                index.correct(offset);
            }
            ExprKind::UnaryOp { operand, .. } => operand.correct(offset),
            _ => {}
        }
    }
}

pub fn source_map(file_path: String, tokens: &Vec<Token>) -> usize {
    let file_map = &mut files.lock().unwrap();
    let file_id = file_map.add(file_path, to_string(&tokens));
    let mut start = 0;
    for token in tokens {
        let end = start + token.to_string().len() as u32;
        file_map.mappings.insert(
            (file_id, Span::new(start, end)),
            (token.file_id, token.span),
        );
        start = end;
    }
    file_id
}

pub fn source_lookup(file_id: usize, span: Span) -> (usize, Span) {
    let file_map = &files.lock().unwrap();
    let start = span.start().to_usize();
    let end = span.end().to_usize();
    for ((key_file_id, key_span), (val_file_id, val_span)) in &file_map.mappings {
        if *key_file_id != file_id {
            continue;
        }
        if span.disjoint(*key_span) {
            continue;
        }
        let new_start = val_span.start().to_usize() + start - key_span.start().to_usize();
        let new_end = min(new_start + end - start, val_span.end().to_usize());
        return (*val_file_id, Span::new(new_start as u32, new_end as u32));
    }
    (file_id, span)
}
