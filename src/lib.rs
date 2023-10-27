pub mod error;
#[cfg(any(feature = "dbus", feature = "tests"))]
pub mod impls;
pub mod property;
pub mod schema;
#[cfg(any(feature = "dbus", feature = "tests"))]
pub mod storage_backend;
pub mod value;
pub use anyhow::Result;
pub use serde;
pub use zvariant;
#[cfg(any(feature = "dbus", feature = "tests"))]
pub mod storage;
pub mod trigger;
e
#[macro_export]
macro_rules! builder_get {
    ($id:ident, $var:ident, $val:expr, $builder:expr, $context:expr) => {
        $id.$var.ok_or($crate::error::BuilderError::unwrap_failed(
            $val, $builder, $context,
        ))?
    };
}
