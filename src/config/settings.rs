use qmetaobject::prelude::*;

cpp! {{
    #include <QtCore/QSettings>
}}

cpp_class! (
    unsafe struct QSettings as "QSettings"
);

#[derive(QObject)]
#[allow(non_snake_case)]
pub struct Settings {
    base: qt_base_class!(trait QObject),

    stringSet: qt_method!(fn(&self, key: String, value: String)),
    stringValue: qt_method!(fn(&self, key: String) -> String),

    // XXX
    // stringListSet: qt_method!(fn (&self, key: String, value: String)),
    // stringListValue: qt_method!(fn (&self, key: String, value: String)),
    boolSet: qt_method!(fn(&self, key: String, value: bool)),
    boolValue: qt_method!(fn(&self, key: String) -> bool),

    defaults: qt_method!(fn(&self)),

    inner: *mut QSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            base: Default::default(),
            stringSet: Default::default(),
            stringValue: Default::default(),

            boolSet: Default::default(),
            boolValue: Default::default(),

            defaults: Default::default(),

            inner: unsafe {
                cpp!([] -> *mut QSettings as "QSettings *" {
                    return new QSettings();
                })
            },
        }
    }
}

impl Drop for Settings {
    fn drop(&mut self) {
        let settings = self.inner;
        unsafe {
            cpp!([settings as "QSettings *"] {
                delete settings;
            })
        }
    }
}

impl Settings {
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

    pub fn set_bool(&mut self, key: &str, value: bool) {
        let key = QString::from(key);
        let settings = self.inner;
        unsafe {
            cpp!([settings as "QSettings *", key as "QString", value as "bool"] {
                settings->setValue(key, value);
            })
        };
    }

    pub fn set_bool_if_unset(&mut self, key: &str, value: bool) {
        if !self.contains(key) {
            self.set_bool(key, value);
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

    pub fn set_string(&mut self, key: &str, value: &str) {
        let key = QString::from(key);
        let value = QString::from(value);
        let settings = self.inner;
        unsafe {
            cpp!([settings as "QSettings *", key as "QString", value as "QString"] {
                settings->setValue(key, value);
            })
        };
    }

    pub fn set_string_if_unset(&mut self, key: &str, value: &str) {
        if !self.contains(key) {
            self.set_string(key, value);
        }
    }

    #[allow(non_snake_case)]
    fn stringSet(&mut self, key: String, val: String) {
        log::info!("Setting string {}", key);
        self.set_string(&key, &val)
    }

    #[allow(non_snake_case)]
    fn stringValue(&self, key: String) -> String {
        self.get_string(key)
    }

    #[allow(non_snake_case)]
    fn boolSet(&mut self, key: String, val: bool) {
        log::info!("Setting bool {}", key);
        self.set_bool(&key, val)
    }

    #[allow(non_snake_case)]
    fn boolValue(&mut self, key: String) -> bool {
        self.get_bool(key)
    }

    pub fn defaults(&mut self) {
        log::info!("Setting default settings.");

        self.set_bool_if_unset("incognito", false);
        self.set_bool_if_unset("enable_notify", true);
        self.set_bool_if_unset("show_notify_message", false);
        self.set_bool_if_unset("minimise_notify", false);
        self.set_bool_if_unset("save_attachments", true);
        self.set_bool_if_unset("share_contacts", true);
        self.set_bool_if_unset("enable_enter_send", false);
        self.set_bool_if_unset("scale_image_attachments", false);
        self.set_bool_if_unset("attachment_log", false);
        self.set_bool_if_unset("quit_on_ui_close", true);
        self.set_string_if_unset(
            "attachment_dir",
            // XXX this has to be adapted to current config struct
            &crate::config::SignalConfig::default()
                .default_attachment_dir()
                .to_string_lossy(),
        );
    }

    pub fn get_string(&self, key: impl AsRef<str>) -> String {
        self.value_string(key.as_ref())
    }

    pub fn get_bool(&self, key: impl AsRef<str>) -> bool {
        self.value_bool(key.as_ref())
    }
}
