use std::rc::Rc;

pub trait ReadSettings {
    fn get_bool(&self, key: &str) -> bool;
    fn get_string(&self, key: &str) -> String;
}

pub trait Settings: ReadSettings {
    fn set_bool(&mut self, key: &str, value: bool);
    fn set_bool_if_unset(&mut self, key: &str, value: bool);
    fn set_string(&mut self, key: &str, value: &str);
    fn set_string_if_unset(&mut self, key: &str, value: &str);
}

pub trait Traits {
    fn new_read_settings(&self) -> Box<dyn ReadSettings>;
    fn new_settings(&self) -> Box<dyn Settings>;
}

pub type SharedTraits = Rc<Box<dyn Traits>>;
