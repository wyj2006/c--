use codespan_reporting::files::{Error, Files, SimpleFile};
use std::ops::Range;

pub struct FileMap {
    pub files: Vec<SimpleFile<String, String>>,
}

impl FileMap {
    pub fn new() -> FileMap {
        FileMap { files: Vec::new() }
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
