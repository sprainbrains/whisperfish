use qmetaobject::*;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignalConfig {
    /// Our telephone number
    pub tel: String,
    /// Our uuid
    pub uuid: String,
    /// The TextSecure server URL
    pub server: String,
    #[serde(rename = "rootCA")]
    /// The TLS signing certificate of the server we connect to
    pub root_ca: String,
    #[serde(rename = "proxy")]
    /// HTTP Proxy URL if one is being used
    pub proxy_server: String,
    /// Code verification method during registration (SMS/VOICE/DEV)
    pub verification_type: String,
    /// Directory for the persistent storage
    pub storage_dir: String,
    /// Whether to store plaintext keys and session state (only for development)
    pub unencrypted_storage: bool,
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
        unsafe {
            cpp!([key as "QString"] -> bool as "bool" {
                QSettings settings;
                return settings.value(key).toBool();
            })
        }
    }

    fn set_bool(&mut self, key: &str, value: bool) {
        let key = QString::from(key);
        unsafe {
            cpp!([key as "QString", value as "bool"] {
                QSettings settings;
                settings.setValue(key, value);
            })
        };
    }

    fn value_string(&self, key: &str) -> String {
        let key = QString::from(key);
        let val = unsafe {
            cpp!([key as "QString"] -> QString as "QString" {
                QSettings settings;
                return settings.value(key).toString();
            })
        };
        val.into()
    }

    fn set_string(&mut self, key: &str, value: &str) {
        let key = QString::from(key);
        let value = QString::from(value);
        unsafe {
            cpp!([ key as "QString", value as "QString"] {
                QSettings settings;
                settings.setValue(key, value);
            })
        };
    }
}

#[derive(QObject, Default)]
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
        self.set_bool("encrypt_database", true);
        self.set_bool("save_attachments", true);
        self.set_bool("share_contacts", true);
        self.set_bool("enable_enter_send", false);
        self.set_bool("scale_image_attachments", false);
        self.set_bool("attachment_log", false);
    }

    pub fn get_string(&self, key: impl AsRef<str>) -> String {
        self.value_string(key.as_ref())
    }

    pub fn get_bool(&self, key: impl AsRef<str>) -> bool {
        self.value_bool(key.as_ref())
    }
}
