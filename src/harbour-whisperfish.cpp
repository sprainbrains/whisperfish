#include <sstream>
#include <string>

#include <sailfishapp.h>
#include <QTranslator>
#include <QGuiApplication>
#include <QQuickView>
#include <QtQml>
#include <QtCore>

#include "whisperfish.hpp"
#include "model/contact.hpp"
#include "model/device.hpp"
#include "model/filepicker.hpp"
#include "model/message.hpp"
#include "model/prompt.hpp"
#include "model/session.hpp"
#include "settings/settings.hpp"
#include "worker/setup.hpp"
#include "worker/client.hpp"
#include "worker/send.hpp"

static void register_types(QQmlEngine* engine, const char* uri, Version v)
{
}

Version get_version() {
    std::stringstream ss(APP_VERSION);
    int v1, v2, v3;
    ss >> v1; ss.get();
    ss >> v2; ss.get();
    ss >> v3;
    return Version { v1, v2, v3 };
}

Paths get_paths() {
    const QString appName = qApp->applicationName();

    auto data_paths = QStandardPaths::writableLocation(QStandardPaths::GenericDataLocation) + "/" + appName;
    auto config_paths = QStandardPaths::writableLocation(QStandardPaths::ConfigLocation) + "/" + appName;
    qInfo() << "Data should be at" << data_paths;
    qInfo() << "Config should be at" << config_paths;

    return Paths { data_paths, config_paths };
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

    get_paths();

    Settings settings;
    SetupWorker setupWorker;
    FilePicker filePicker;
    ContactModel contactModel;
    Prompt prompt;
    SessionModel sessionModel;
    MessageModel messageModel;
    DeviceModel deviceModel;
    ClientWorker clientWorker;
    SendWorker sendWorker;

    // Start GUI
    QQmlEngine* engine = view->engine();
    register_types(engine, "harbour.whisperfish", version);

    QQmlContext* root = view->rootContext();
    root->setContextProperty("AppVersion", APP_VERSION);

    root->setContextProperty("Prompt", &prompt);
    root->setContextProperty("SettingsBridge", &settings);
    root->setContextProperty("FilePicker", &filePicker);
    root->setContextProperty("SessionModel", &sessionModel);
    root->setContextProperty("MessageModel", &messageModel);
    root->setContextProperty("ContactModel", &contactModel);
    root->setContextProperty("DeviceModel", &deviceModel);
    root->setContextProperty("SetupWorker", &setupWorker);
    root->setContextProperty("ClientWorker", &clientWorker);
    root->setContextProperty("SendWorker", &sendWorker);

    view->setSource(SailfishApp::pathTo("qml/harbour-whisperfish.qml"));
    view->setTitle("Whisperfish");
    view->showFullScreen();
    return app->exec();
}
