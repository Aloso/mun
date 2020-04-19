use std::io;
use std::path::{Path, PathBuf};

use crate::DispatchTable;
use abi::AssemblyInfo;
use libloading::Symbol;

mod temp_library;

use self::temp_library::TempLibrary;
use crate::garbage_collector::{GarbageCollector, RawTypeInfo};
use memory::{diff::diff, mapping::MemoryMapper};
use std::{collections::HashSet, sync::Arc};

/// An assembly is a hot reloadable compilation unit, consisting of one or more Mun modules.
pub struct Assembly {
    library_path: PathBuf,
    _library: Option<TempLibrary>,
    info: AssemblyInfo,
    allocator: Arc<GarbageCollector>,
}

impl Assembly {
    /// Loads an assembly and its information for the shared library at `library_path`. The
    /// resulting `Assembly` is ensured to be linkable.
    pub fn load(
        library_path: &Path,
        gc: Arc<GarbageCollector>,
        runtime_dispatch_table: &DispatchTable,
    ) -> Result<Self, failure::Error> {
        let library = TempLibrary::new(library_path)?;

        // Check whether the library has a symbols function
        let get_info: Symbol<'_, extern "C" fn() -> AssemblyInfo> =
            unsafe { library.library().get(b"get_info") }?;

        let set_allocator_handle: Symbol<'_, extern "C" fn(*mut std::ffi::c_void)> =
            unsafe { library.library().get(b"set_allocator_handle") }?;

        let allocator_ptr = Arc::into_raw(gc.clone()) as *mut std::ffi::c_void;
        set_allocator_handle(allocator_ptr);

        let info = get_info();
        let assembly = Assembly {
            library_path: library_path.to_path_buf(),
            _library: Some(library),
            info,
            allocator: gc,
        };

        assembly.ensure_linkable(runtime_dispatch_table)?;
        Ok(assembly)
    }

    /// Verifies that the `Assembly` resolves all dependencies in the `DispatchTable`.
    fn ensure_linkable(&self, runtime_dispatch_table: &DispatchTable) -> Result<(), io::Error> {
        if let Some(dependencies) = runtime_dispatch_table
            .fn_dependencies
            .get(self.info.symbols.path())
        {
            let fn_names: HashSet<&str> = self
                .info
                .symbols
                .functions()
                .iter()
                .map(|f| f.signature.name())
                .collect();

            for fn_name in dependencies.keys() {
                if !fn_names.contains(&fn_name.as_str()) {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("Failed to link: function `{}` is missing.", fn_name),
                    ));
                }
            }

            for fn_info in self.info.symbols.functions().iter() {
                let (fn_sig, _) = dependencies
                    .get(fn_info.signature.name())
                    .expect("The dependency must exist after the previous check.");

                // TODO: This is a hack
                if fn_info.signature.return_type() != fn_sig.return_type()
                    || fn_info.signature.arg_types().len() != fn_sig.arg_types().len()
                    || !fn_info
                        .signature
                        .arg_types()
                        .iter()
                        .zip(fn_sig.arg_types().iter())
                        .all(|(a, b)| PartialEq::eq(a, b))
                {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("Failed to link: function '{}' is missing. A function with the same name does exist, but the signatures do not match (expected: {}, found: {}).", fn_sig.name(), fn_sig, fn_info.signature),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Links the assembly using the runtime's dispatch table.
    pub fn link(&mut self, runtime_dispatch_table: &mut DispatchTable) {
        // Fill the runtime's `DispatchTable`
        for function in self.info.symbols.functions() {
            runtime_dispatch_table.insert_fn(function.signature.name(), function.clone());
        }

        for (dispatch_ptr, fn_signature) in self.info.dispatch_table.iter_mut() {
            let fn_ptr = runtime_dispatch_table
                .get_fn(fn_signature.name())
                .unwrap_or_else(|| panic!("Function '{}' is expected to exist.", fn_signature))
                .fn_ptr;

            *dispatch_ptr = fn_ptr;
        }
    }

    /// Swaps the assembly's shared library and its information for the library at `library_path`.
    pub fn swap(
        &mut self,
        library_path: &Path,
        runtime_dispatch_table: &mut DispatchTable,
    ) -> Result<(), failure::Error> {
        let mut new_assembly =
            Assembly::load(library_path, self.allocator.clone(), runtime_dispatch_table)?;

        let old_types: Vec<RawTypeInfo> = self
            .info
            .symbols
            .types()
            .iter()
            .map(|ty| (*ty as *const abi::TypeInfo).into())
            .collect();

        let new_types: Vec<RawTypeInfo> = new_assembly
            .info
            .symbols
            .types()
            .iter()
            .map(|ty| (*ty as *const abi::TypeInfo).into())
            .collect();

        self.allocator
            .map_memory(&old_types, &new_types, &diff(&old_types, &new_types));

        // Remove the old assembly's functions
        for function in self.info.symbols.functions() {
            runtime_dispatch_table.remove_fn(function.signature.name());
        }

        new_assembly.link(runtime_dispatch_table);
        *self = new_assembly;
        Ok(())
    }

    /// Returns the assembly's information.
    pub fn info(&self) -> &AssemblyInfo {
        &self.info
    }

    /// Returns the path corresponding to the assembly's library.
    pub fn library_path(&self) -> &Path {
        self.library_path.as_path()
    }
}
