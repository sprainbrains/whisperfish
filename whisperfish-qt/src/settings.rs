use cpp::{cpp, cpp_class};
use qttypes::QString;
use whisperfish_traits::{ReadSettings, Settings};

cpp! {{
    #include <QtCore/QSettings>
    #include <QtCore/QStandardPaths>
    #include <QtCore/QFile>
}}

cpp_class! (
    unsafe struct QSettings as "QSettings"
);

pub struct SettingsQt {
    inner: *mut QSettings,
}

impl Default for SettingsQt {
    fn default() -> Self {
        Self {
            inner: unsafe {
                cpp!([] -> *mut QSettings as "QSettings *" {
                    QString settingsFile = QStandardPaths::writableLocation(QStandardPaths::ConfigLocation)
                                                + "/be.rubdos/harbour-whisperfish/harbour-whisperfish.conf";

                    QSettings* settings = new QSettings(settingsFile, QSettings::NativeFormat);

                    QStringList path_keys;
                    path_keys << "attachment_dir" << "camera_dir";
                    QString old_path = ".local/share/harbour-whisperfish";
                    QString new_path = ".local/share/be.rubdos/harbour-whisperfish";

                    foreach(const QString &key, path_keys) {
                        if(settings->contains(key) && settings->value(key).toString().contains(old_path)) {
                            settings->setValue(key, settings->value(key).toString().replace(old_path, new_path));
                        }
                    }

                    return settings;
                })
            },
        }
    }
}

impl Drop for SettingsQt {
    fn drop(&mut self) {
        let settings = self.inner;
        unsafe {
            cpp!([settings as "QSettings *"] {
                delete settings;
            })
        }
    }
}

impl ReadSettings for SettingsQt {
    fn get_bool(&self, key: &str) -> bool {
        self.value_bool(key)
    }

    fn get_string(&self, key: &str) -> String {
        self.value_string(key)
    }
}

impl Settings for SettingsQt {
    fn set_bool(&mut self, key: &str, value: bool) {
        let key = QString::from(key);
        let settings = self.inner;
        unsafe {
            cpp!([settings as "QSettings *", key as "QString", value as "bool"] {
                settings->setValue(key, value);
            })
        };
    }

    fn set_bool_if_unset(&mut self, key: &str, value: bool) {
        if !self.contains(key) {
            self.set_bool(key, value);
        }
    }

    fn set_string(&mut self, key: &str, value: &str) {
        let key = QString::from(key);
        let value = QString::from(value);
        let settings = self.inner;
        unsafe {
            cpp!([settings as "QSettings *", key as "QString", value as "QString"] {
                settings->setValue(key, value);
            })
        };
    }

    fn set_string_if_unset(&mut self, key: &str, value: &str) {
        if !self.contains(key) {
            self.set_string(key, value);
        }
    }
}

impl SettingsQt {
    fn contains(&self, key: &str) -> bool {
        let key = QString::from(key);
        let settings = self.inner;
        unsafe {
            cpp!([settings as "QSettings *", key as "QString"] -> bool as "bool" {
                return settings->contains(key);
            })
        }
    }

    fn value_bool(&self, key: &str) -> bool {
        let key = QString::from(key);
        let settings = self.inner;
        unsafe {
            cpp!([settings as "QSettings *", key as "QString"] -> bool as "bool" {
                return settings->value(key).toBool();
            })
        }
    }

    fn value_string(&self, key: &str) -> String {
        let key = QString::from(key);
        let settings = self.inner;
        let val = unsafe {
            cpp!([settings as "QSettings *", key as "QString"] -> QString as "QString" {
                return settings->value(key).toString();
            })
        };
        val.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::fs::File;
    use std::io::{Read, Write};
    use std::path::Path;

    struct SettingsDeleter<'a>(&'a Path);

    impl<'a> Drop for SettingsDeleter<'a> {
        fn drop(&mut self) {
            fs::remove_file(self.0).unwrap();
        }
    }

    #[test]
    fn settings_integration_smoke_tests() {
        // Prevent overriding the file in the test by mistake
        let config_dir = dirs::config_dir().unwrap();
        let settings_dir = config_dir.join("be.rubdos/harbour-whisperfish");
        fs::create_dir_all(&settings_dir).unwrap();

        let settings_file = settings_dir.join("harbour-whisperfish.conf");
        assert!(
            !settings_file.exists(),
            "{} exists. To make sure that tests do not override it, please back it up manually",
            settings_file.display()
        );

        // Test read a sample settings
        let _deleter = SettingsDeleter(&settings_file);

        let mut file = File::create(&settings_file).unwrap();
        file.write_all(b"[General]\n").unwrap();
        file.write_all(b"test_bool=true\n").unwrap();
        file.write_all(b"test_string=Hello world\n").unwrap();
        drop(file);

        let mut settings = SettingsQt::default();
        assert_eq!(settings.get_bool("test_bool"), true);
        assert_eq!(
            settings.get_string("test_string"),
            "Hello world".to_string()
        );

        settings.set_bool("test_bool", false);
        settings.set_string("test_string", "Hello Qt");
        drop(settings);

        let mut file = File::open(&settings_file).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        assert!(content.contains("test_bool=false"));
        assert!(content.contains("test_string=Hello Qt"));
    }
}
