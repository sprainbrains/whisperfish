#include <sstream>
#include <string>

#include <sailfishapp.h>
#include <QTranslator>
#include <QGuiApplication>
#include <QQuickView>
#include <QtQml>

#include "whisperfish.hpp"

static void register_types(QQmlEngine* engine, const char* uri, Version v)
{
}

static const Version get_version() {
    std::stringstream ss(APP_VERSION);
    int v1, v2, v3;
    ss >> v1; ss.get();
    ss >> v2; ss.get();
    ss >> v3;
    return Version { v1, v2, v3 };
}

int main(int argc, char *argv[])
{
    QScopedPointer<QGuiApplication> app(SailfishApp::application(argc, argv));
    qApp->setApplicationVersion(QString(APP_VERSION));

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

    auto version = get_version();
    qInfo() << "Whisperfish version "
            << version.v1
            << "." << version.v2
            << "." << version.v3;

    QQmlEngine* engine = view->engine();
    register_types(engine, "harbour.whisperfish", version);

    QQmlContext* root = view->rootContext();
    root->setContextProperty("AppVersion", APP_VERSION);

    view->setSource(SailfishApp::pathTo("qml/harbour-whisperfish.qml"));
    view->setTitle("Whisperfish");
    view->showFullScreen();
    return app->exec();
}
