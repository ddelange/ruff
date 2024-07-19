mod db;
mod module;
mod module_name;
mod path;
mod resolver;
mod state;
mod typeshed;

#[cfg(test)]
mod testing;

use crate::resolver::module_resolution_settings;
pub use db::{Db, Jar};
pub use module::{Module, ModuleKind};
pub use module_name::ModuleName;
pub use resolver::resolve_module;
use ruff_db::system::SystemPath;
pub use typeshed::{
    vendored_typeshed_stubs, TypeshedVersionsParseError, TypeshedVersionsParseErrorKind,
};

/// Returns an iterator over all search paths pointing to a system path
pub fn system_module_search_paths(db: &dyn Db) -> impl Iterator<Item = &SystemPath> {
    module_resolution_settings(db)
        .search_paths(db)
        .filter_map(|path| path.as_system_path())
        .map(|path_buf| path_buf.as_path())
}
