mod settings;

use std::rc::Rc;
use whisperfish_traits::{ReadSettings, Settings, SharedTraits, Traits};

struct QtTraits;

impl Traits for QtTraits {
    fn new_read_settings(&self) -> Box<dyn ReadSettings> {
        Box::new(settings::SettingsQt::default())
    }

    fn new_settings(&self) -> Box<dyn Settings> {
        Box::new(settings::SettingsQt::default())
    }
}

pub fn new_traits() -> SharedTraits {
    Rc::new(Box::new(QtTraits))
}
