#[macro_use]
extern crate cpp;

use qmetaobject::*;

mod qrc;

fn main() {
    unsafe {
        cpp! { {
            #include <QtCore/QCoreApplication>
            #include <QtCore/QString>
        }}
        cpp!{[]{
            QCoreApplication::setApplicationName(QStringLiteral("harbour-whisperfish"));
        }}
    }
    qrc::load();

    let mut engine = QmlEngine::new();
    engine.load_file("qrc:/qml/harbour-whisperfish.qml".into());
    engine.exec();
}
