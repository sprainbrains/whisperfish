use qmeta_async::with_executor;
use qmetaobject::prelude::*;
use whisperfish_traits::{ReadSettings, Settings};

#[derive(QObject)]
#[allow(non_snake_case, dead_code)]
pub struct SettingsBridge {
    base: qt_base_class!(trait QObject),

    // XXX
    // stringListSet: qt_method!(fn (&self, key: String, value: String)),
    // stringListValue: qt_method!(fn (&self, key: String, value: String)),
    avatarExists: qt_method!(fn(&self, key: String) -> bool),

    inner: Box<dyn Settings>,

    incognito: qt_property!(bool; READ get_incognito WRITE set_incognito NOTIFY incognito_changed),
    enable_notify: qt_property!(bool; READ get_enable_notify WRITE set_enable_notify NOTIFY enable_notify_changed),
    debug_mode: qt_property!(bool; READ get_debug_mode WRITE set_debug_mode NOTIFY debug_mode_changed),
    enable_typing_indicators: qt_property!(bool; READ get_enable_typing_indicators WRITE set_enable_typing_indicators NOTIFY enable_typing_indicators_changed),
    show_notify_message: qt_property!(bool; READ get_show_notify_message WRITE set_show_notify_message NOTIFY show_notify_message_changed),
    prefer_device_contacts: qt_property!(bool; READ get_prefer_device_contacts WRITE set_prefer_device_contacts NOTIFY prefer_device_contacts_changed),
    minimise_notify: qt_property!(bool; READ get_minimise_notify WRITE set_minimise_notify NOTIFY minimise_notify_changed),
    save_attachments: qt_property!(bool; READ get_save_attachments WRITE set_save_attachments NOTIFY save_attachments_changed),
    share_contacts: qt_property!(bool; READ get_share_contacts WRITE set_share_contacts NOTIFY share_contacts_changed),
    enable_enter_send: qt_property!(bool; READ get_enable_enter_send WRITE set_enable_enter_send NOTIFY enable_enter_send_changed),
    scale_image_attachments: qt_property!(bool; READ get_scale_image_attachments WRITE set_scale_image_attachments NOTIFY scale_image_attachments_changed),
    attachment_log: qt_property!(bool; READ get_attachment_log WRITE set_attachment_log NOTIFY attachment_log_changed),
    quit_on_ui_close: qt_property!(bool; READ get_quit_on_ui_close WRITE set_quit_on_ui_close NOTIFY quit_on_ui_close_changed),
    country_code: qt_property!(String; READ get_country_code WRITE set_country_code NOTIFY country_code_changed),
    avatar_dir: qt_property!(String; READ get_avatar_dir WRITE set_avatar_dir NOTIFY avatar_dir_changed),
    attachment_dir: qt_property!(String; READ get_attachment_dir WRITE set_attachment_dir NOTIFY attachment_dir_changed),
    camera_dir: qt_property!(String; READ get_camera_dir WRITE set_camera_dir NOTIFY camera_dir_changed),

    incognito_changed: qt_signal!(value: bool),
    enable_notify_changed: qt_signal!(value: bool),
    debug_mode_changed: qt_signal!(value: bool),
    enable_typing_indicators_changed: qt_signal!(value: bool),
    show_notify_message_changed: qt_signal!(value: bool),
    prefer_device_contacts_changed: qt_signal!(value: bool),
    minimise_notify_changed: qt_signal!(value: bool),
    save_attachments_changed: qt_signal!(value: bool),
    share_contacts_changed: qt_signal!(value: bool),
    enable_enter_send_changed: qt_signal!(value: bool),
    scale_image_attachments_changed: qt_signal!(value: bool),
    attachment_log_changed: qt_signal!(value: bool),
    quit_on_ui_close_changed: qt_signal!(value: bool),
    country_code_changed: qt_signal!(value: String),
    avatar_dir_changed: qt_signal!(value: String),
    attachment_dir_changed: qt_signal!(value: String),
    camera_dir_changed: qt_signal!(value: String),
}

impl ReadSettings for SettingsBridge {
    fn get_bool(&self, key: &str) -> bool {
        self.inner.get_bool(key)
    }

    fn get_string(&self, key: &str) -> String {
        self.inner.get_string(key)
    }
}

impl Settings for SettingsBridge {
    fn set_bool(&mut self, key: &str, value: bool) {
        self.inner.set_bool(key, value)
    }

    fn set_bool_if_unset(&mut self, key: &str, value: bool) {
        self.inner.set_bool_if_unset(key, value)
    }

    fn set_string(&mut self, key: &str, value: &str) {
        self.inner.set_string(key, value)
    }

    fn set_string_if_unset(&mut self, key: &str, value: &str) {
        self.inner.set_string_if_unset(key, value)
    }
}

impl SettingsBridge {
    pub fn new(inner: Box<dyn Settings>) -> Self {
        Self {
            base: Default::default(),

            avatarExists: Default::default(),

            inner,

            incognito: false,
            enable_notify: true,
            debug_mode: false,
            enable_typing_indicators: false,
            show_notify_message: false,
            prefer_device_contacts: false,
            minimise_notify: false,
            save_attachments: true,
            share_contacts: true,
            enable_enter_send: false,
            scale_image_attachments: false,
            attachment_log: false,
            quit_on_ui_close: true,
            country_code: Default::default(),
            avatar_dir: Default::default(),
            attachment_dir: Default::default(),
            camera_dir: Default::default(),

            incognito_changed: Default::default(),
            enable_notify_changed: Default::default(),
            debug_mode_changed: Default::default(),
            enable_typing_indicators_changed: Default::default(),
            show_notify_message_changed: Default::default(),
            prefer_device_contacts_changed: Default::default(),
            minimise_notify_changed: Default::default(),
            save_attachments_changed: Default::default(),
            share_contacts_changed: Default::default(),
            enable_enter_send_changed: Default::default(),
            scale_image_attachments_changed: Default::default(),
            attachment_log_changed: Default::default(),
            quit_on_ui_close_changed: Default::default(),
            country_code_changed: Default::default(),
            avatar_dir_changed: Default::default(),
            attachment_dir_changed: Default::default(),
            camera_dir_changed: Default::default(),
        }
    }

    pub fn get_incognito(&mut self) -> bool {
        self.get_bool("incognito")
    }

    pub fn get_enable_notify(&self) -> bool {
        self.get_bool("enable_notify")
    }

    pub fn get_debug_mode(&self) -> bool {
        self.get_bool("debug_mode")
    }

    pub fn get_enable_typing_indicators(&self) -> bool {
        self.get_bool("enable_typing_indicators")
    }

    pub fn get_show_notify_message(&self) -> bool {
        self.get_bool("show_notify_message")
    }

    pub fn get_prefer_device_contacts(&self) -> bool {
        self.get_bool("prefer_device_contacts")
    }

    pub fn get_minimise_notify(&self) -> bool {
        self.get_bool("minimise_notify")
    }

    pub fn get_save_attachments(&self) -> bool {
        self.get_bool("save_attachments")
    }

    pub fn get_share_contacts(&self) -> bool {
        self.get_bool("share_contacts")
    }

    pub fn get_enable_enter_send(&self) -> bool {
        self.get_bool("enable_enter_send")
    }

    pub fn get_scale_image_attachments(&self) -> bool {
        self.get_bool("scale_image_attachments")
    }

    pub fn get_attachment_log(&self) -> bool {
        self.get_bool("attachment_log")
    }

    pub fn get_quit_on_ui_close(&self) -> bool {
        self.get_bool("quit_on_ui_close")
    }

    pub fn get_avatar_dir(&self) -> String {
        self.get_string("avatar_dir")
    }

    pub fn get_country_code(&self) -> String {
        self.get_string("country_code")
    }

    pub fn get_attachment_dir(&self) -> String {
        self.get_string("attachment_dir")
    }

    pub fn get_camera_dir(&self) -> String {
        self.get_string("camera_dir")
    }

    pub fn set_incognito(&mut self, value: bool) {
        self.set_bool("incognito", value);
        self.incognito_changed(value);
    }

    pub fn set_enable_notify(&mut self, value: bool) {
        self.set_bool("enable_notify", value);
        self.enable_notify_changed(value);
    }

    pub fn set_debug_mode(&mut self, value: bool) {
        self.set_bool("debug_mode", value);
        self.debug_mode_changed(value);
    }

    pub fn set_enable_typing_indicators(&mut self, value: bool) {
        self.set_bool("enable_typing_indicators", value);
        self.enable_typing_indicators_changed(value);
    }

    pub fn set_show_notify_message(&mut self, value: bool) {
        self.set_bool("show_notify_message", value);
        self.show_notify_message_changed(value);
    }

    pub fn set_prefer_device_contacts(&mut self, value: bool) {
        self.set_bool("prefer_device_contacts", value);
        self.prefer_device_contacts_changed(value);
    }

    pub fn set_minimise_notify(&mut self, value: bool) {
        self.set_bool("minimise_notify", value);
        self.minimise_notify_changed(value);
    }

    pub fn set_save_attachments(&mut self, value: bool) {
        self.set_bool("save_attachments", value);
        self.save_attachments_changed(value);
    }

    pub fn set_share_contacts(&mut self, value: bool) {
        self.set_bool("share_contacts", value);
        self.share_contacts_changed(value);
    }

    pub fn set_enable_enter_send(&mut self, value: bool) {
        self.set_bool("enable_enter_send", value);
        self.enable_enter_send_changed(value);
    }

    pub fn set_scale_image_attachments(&mut self, value: bool) {
        self.set_bool("scale_image_attachments", value);
        self.scale_image_attachments_changed(value);
    }

    pub fn set_attachment_log(&mut self, value: bool) {
        self.set_bool("attachment_log", value);
        self.attachment_log_changed(value);
    }

    pub fn set_quit_on_ui_close(&mut self, value: bool) {
        self.set_bool("quit_on_ui_close", value);
        self.quit_on_ui_close_changed(value);
    }

    pub fn set_country_code(&mut self, value: String) {
        self.set_string("country_code", &value);
        self.country_code_changed(value);
    }

    pub fn set_avatar_dir(&mut self, value: String) {
        self.set_string("avatar_dir", &value);
        self.avatar_dir_changed(value);
    }

    pub fn set_attachment_dir(&mut self, value: String) {
        self.set_string("attachment_dir", &value);
        self.attachment_dir_changed(value);
    }

    pub fn set_camera_dir(&mut self, value: String) {
        self.set_string("camera_dir", &value);
        self.camera_dir_changed(value);
    }

    #[allow(non_snake_case)]
    #[with_executor]
    fn avatarExists(&mut self, uuid: String) -> bool {
        self.avatar_exists(uuid)
    }

    pub fn defaults(&mut self) {
        log::info!("Setting default settings.");

        self.set_bool_if_unset("incognito", false);
        self.set_bool_if_unset("debug_mode", false);
        self.set_bool_if_unset("enable_notify", true);
        self.set_bool_if_unset("enable_typing_indicators", false);
        self.set_bool_if_unset("show_notify_message", false);
        self.set_bool_if_unset("prefer_device_contacts", false);
        self.set_bool_if_unset("minimise_notify", false);
        self.set_bool_if_unset("save_attachments", true);
        self.set_bool_if_unset("share_contacts", true);
        self.set_bool_if_unset("enable_enter_send", false);
        self.set_bool_if_unset("scale_image_attachments", false);
        self.set_bool_if_unset("attachment_log", false);
        self.set_bool_if_unset("quit_on_ui_close", true);
        self.set_string_if_unset("country_code", "");
        self.set_string_if_unset(
            "avatar_dir",
            // XXX this has to be adapted to current config struct
            &crate::config::SignalConfig::default()
                .get_avatar_dir()
                .to_string_lossy(),
        );
        self.set_string_if_unset(
            "attachment_dir",
            // XXX this has to be adapted to current config struct
            &crate::config::SignalConfig::default()
                .default_attachment_dir()
                .to_string_lossy(),
        );
        self.set_string_if_unset(
            "camera_dir",
            // XXX this has to be adapted to current config struct
            &crate::config::SignalConfig::default()
                .default_camera_dir()
                .to_string_lossy(),
        );
    }

    pub fn avatar_exists(&self, uuid: impl AsRef<str>) -> bool {
        crate::config::SignalConfig::default()
            .get_avatar_dir()
            .join(uuid.as_ref())
            .exists()
    }
}
