pub use self::vendored::vendored_typeshed_stubs;
pub(crate) use self::versions::{LazyTypeshedVersions, TypeshedVersionsQueryResult};
pub use self::versions::{TypeshedVersionsParseError, TypeshedVersionsParseErrorKind};

mod vendored;
mod versions;
