use qmetaobject::*;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignalConfig {
    /// Our telephone number
    pub tel: Option<String>,
    /// Our uuid
    pub uuid: Option<String>,
    /// The TextSecure server URL
    pub server: Option<String>,
    #[serde(rename = "rootCA")]
    /// The TLS signing certificate of the server we connect to
    pub root_ca: Option<String>,
    #[serde(rename = "proxy")]
    /// HTTP Proxy URL if one is being used
    pub proxy_server: Option<String>,
    /// Directory for the persistent storage
    pub storage_dir: String,
    /// Password to the storage
    pub storage_password: String,
    #[serde(rename = "loglevel")]
    /// Verbosity of the logging messages
    pub log_level: String,
    /// Override for the default HTTP User Agent header field
    pub user_agent: String,
    #[serde(rename = "alwaysTrustPeerID")]
    /// Workaround until proper handling of peer reregistering with new ID.
    pub always_trust_peer_id: bool,
}

cpp! {{
    #include <QtCore/QSettings>
}}

impl Settings {
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
}

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

        self.set_bool("incognito", false);
        self.set_bool("enable_notify", true);
        self.set_bool("show_notify_message", false);
        self.set_bool("save_attachments", true);
        self.set_bool("share_contacts", true);
        self.set_bool("enable_enter_send", false);
        self.set_bool("scale_image_attachments", false);
        self.set_bool("attachment_log", false);
        self.set_bool("quit_on_ui_close", true);

        self.set_string(
            "attachment_dir",
            crate::store::default_location()
                .expect("default location")
                .join("storage")
                .join("attachments")
                .to_str()
                .expect("utf8 path"),
        );
    }

    pub fn get_string(&self, key: impl AsRef<str>) -> String {
        self.value_string(key.as_ref())
    }

    pub fn get_bool(&self, key: impl AsRef<str>) -> bool {
        self.value_bool(key.as_ref())
    }
}
