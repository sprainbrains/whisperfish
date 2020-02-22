#![recursion_limit="256"]

#[macro_use]
extern crate cpp;

use qmetaobject::*;

mod qrc;

fn main() {
    qrc::load();

    unsafe {
        cpp! { {
            #include <QtCore/QCoreApplication>
            #include <QtCore/QString>

            #include <QtGui/QGuiApplication>
            #include <QtQuick/QQuickView>
            #include <QtQml/QtQml>
            #include <QtCore/QtCore>

            #include <sailfishapp.h>
        }}
        cpp!{[]{
            int argc = 1;
            char *_appName = "harbour-whisperfish";
            char **argv = &_appName;
            QScopedPointer<QGuiApplication> app(SailfishApp::application(argc, argv));
            qApp->setApplicationVersion("0.6.0");

            QTranslator translator(qApp);
            const QString appName = qApp->applicationName();
            const QString transDir = SailfishApp::pathTo(QStringLiteral("translations")).toLocalFile();
            const QLocale locale;
            if (!translator.load(locale, appName, "-", transDir, ".qm")) {
                qWarning() << "Failed to load translator for" << QLocale::system().uiLanguages()
                           << "Searched" << transDir << "for" << appName;
                if(!translator.load(appName, transDir)) {
                    qWarning() << "Could not load default translator either!";
                }
                app->installTranslator(&translator);
            }

            QScopedPointer<QQuickView> view(SailfishApp::createView());

            view->setSource(SailfishApp::pathTo("qml/harbour-whisperfish.qml"));
            app->exec();
        }}
    }

    // let mut engine = QmlEngine::new();
    // engine.load_file("qrc:/qml/harbour-whisperfish.qml".into());
    // engine.exec();
}
