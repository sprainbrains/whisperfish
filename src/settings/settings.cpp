#include <QSettings>

#include "settings/settings.hpp"
#include "whisperfish.hpp"


// Creating QSettings is cheap, and by not having it as a field,
// we can set a default configuration on first start.

void Settings::setup() {
    auto paths = get_paths();
    auto config_path = paths.config + "/harbour-whisperfish.conf";

    if (!QFileInfo::exists(config_path)) {
        qInfo() << "Configuration not found at" << config_path;
        setDefaults();
    } else {
        qInfo() << "Configuration found.";
    }
}

bool Settings::boolValue(const QString key) const {
    QSettings settings;
    return settings.value(key).toBool();
}

void Settings::boolSet(const QString key, bool val) {
    QSettings settings;
    settings.setValue(key, val);
}

QString Settings::stringValue(const QString key) const {
    QSettings settings;
    return settings.value(key).toString();
}

void Settings::stringSet(const QString key, const QString val) {
    QSettings settings;
    settings.setValue(key, val);
}

void Settings::setDefaults() {
    qInfo() << "Generating defaults.";

    QSettings settings;

	settings.setValue("incognito", false);
	settings.setValue("enable_notify", true);
	settings.setValue("show_notify_message", false);
	settings.setValue("encrypt_database", true);
	settings.setValue("save_attachments", true);
	settings.setValue("share_contacts", true);
	settings.setValue("enable_enter_send", false);
	settings.setValue("scale_image_attachments", false);
	settings.setValue("country_code", "");
}
