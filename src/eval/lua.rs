use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::default::Default;
use std::fs;
use std::path::{Path, PathBuf};
use comemo::{Prehashed, track};
use elsa::FrozenVec;
use once_cell::sync::OnceCell;
use rlua::UserData;
use crate::diag::{FileError, FileResult};
use crate::eval::Library;
use crate::font::{Font, FontBook};
use crate::model::Content;
use crate::syntax::{Source, SourceId};
use crate::util::{Buffer, PathExt};
use crate::World;

/// Holds canonical data for all paths pointing to the same entity.
#[derive(Default)]
struct PathSlot {
    source: OnceCell<FileResult<SourceId>>,
    buffer: OnceCell<FileResult<Buffer>>,
}

pub struct LuaWorld {
    library: Prehashed<Library>,
    book: Prehashed<FontBook>,
    fonts: Vec<Font>,
    sources: FrozenVec<Box<Source>>,
    main: SourceId,
    paths: RefCell<HashMap<PathBuf, PathSlot>>,
}

impl LuaWorld {
    fn slot(&self, path: &Path) -> RefMut<PathSlot> {
        RefMut::map(self.paths.borrow_mut(), |paths| {
            paths.entry(path.normalize()).or_default()
        })
    }

    fn insert(&self, path: &Path, text: String) -> SourceId {
        let id = SourceId::from_u16(self.sources.len() as u16);
        let source = Source::new(id, path, text);
        self.sources.push(Box::new(source));
        id
    }
}

impl LuaWorld {
    pub fn new(library: Prehashed<Library>) -> Self {
        Self {
            library,
            book: Default::default(),
            fonts: Default::default(),
            sources: Default::default(),
            main: SourceId::detached(),
            paths: Default::default()
        }
    }
}

impl World for LuaWorld {
    fn library(&self) -> &Prehashed<Library> {
        &self.library
    }

    fn main(&self) -> &Source {
        self.source(self.main)
    }

    fn resolve(&self, path: &Path) -> FileResult<SourceId> {
        self.slot(path)
            .source
            .get_or_init(|| {
                let buf = read(path)?;
                let text = String::from_utf8(buf)?;
                Ok(self.insert(path, text))
            })
            .clone()
    }

    fn source(&self, id: SourceId) -> &Source {
        &self.sources[id.into_u16() as usize]
    }

    fn book(&self) -> &Prehashed<FontBook> {
        &self.book
    }

    fn font(&self, id: usize) -> Option<Font> {
        Some(self.fonts[id].clone())
    }

    fn file(&self, path: &Path) -> FileResult<Buffer> {
        self.slot(path)
            .buffer
            .get_or_init(|| read(path).map(Buffer::from))
            .clone()
    }
}

/// Read a file.
fn read(path: &Path) -> FileResult<Vec<u8>> {
    let f = |e| FileError::from_io(e, path);
    if fs::metadata(path).map_err(f)?.is_dir() {
        Err(FileError::IsDirectory)
    } else {
        fs::read(path).map_err(f)
    }
}

impl UserData for Content {

}
