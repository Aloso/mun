use std::path::Path;

use crate::error::*;
use libloading::{self, Symbol};

use mun_symbols::prelude::*;

/// A wrapper for a shared library and its corresponding symbol metadata.
pub struct Library {
    inner: libloading::Library,
    symbols: &'static ModuleInfo,
}

impl Library {
    /// Loads the shared library at `path`, retrieves its symbol metadata, and constructs a library
    /// wrapper.
    pub fn new(path: &Path) -> Result<Library> {
        let library = libloading::Library::new(path)?;

        // Check whether the library has a symbols function
        let symbols_fn: Symbol<'_, fn() -> &'static ModuleInfo> =
            unsafe { library.get(b"symbols") }.map_err(Error::from)?;

        let symbols = symbols_fn();

        Ok(Library {
            inner: library,
            symbols,
        })
    }

    /// Retrieves the inner shared library.
    pub fn inner(&self) -> &libloading::Library {
        &self.inner
    }

    /// Retrieves the libraries symbol metadata.
    pub fn module_info(&self) -> &ModuleInfo {
        self.symbols
    }
}
