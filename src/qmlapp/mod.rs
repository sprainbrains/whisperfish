#[cfg(feature = "sailfish")]
pub use sailors::sailfishapp::QmlApp;

#[cfg(not(feature = "sailfish"))]
pub type QmlApp = qmetaobject::QmlEngine;
