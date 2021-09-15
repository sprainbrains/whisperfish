#[cfg(feature = "sailfish")]
mod native;

#[cfg(feature = "sailfish")]
pub use native::*;

#[cfg(not(feature = "sailfish"))]
pub type QmlApp = qmetaobject::QmlEngine;
