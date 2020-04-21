use qmetaobject::{qrc, qrc_internal};

include!(concat!(env!("OUT_DIR"), "/qml.rs"));

qrc!(other_resources,
);

pub fn load() {
    other_resources();
    qml_resources();
}
