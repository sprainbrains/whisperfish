use std::collections::HashMap;
use std::str::FromStr;

use crate::settings::*;

use diesel::prelude::*;
use phonenumber::Mode;
use qmetaobject::*;

/// `/home/nemo/.local/share/system/Contacts/qtcontacts-sqlite/contacts.db`
///
/// Contains only the part after `share`.
const DB_PATH: &str = "system/Contacts/qtcontacts-sqlite/contacts.db";

#[derive(QObject, Default)]
pub struct ContactModel {
    base: qt_base_class!(trait QAbstractListModel),
    // actor: Option<Addr<ContactActor>>,
    content: Vec<Contact>,

    refresh: qt_method!(fn(&mut self)),
    format: qt_method!(fn(&self, string: QString) -> QString),
    name: qt_method!(fn(&self, source: QString) -> QString),

    total: qt_property!(i32; NOTIFY contacts_changed READ total),
    count: qt_property!(i32; NOTIFY contacts_changed READ row_count),

    contacts_changed: qt_signal!(),
}

#[derive(Queryable, Clone, Debug)]
pub struct Contact {
    name: String,
    tel: String,
}

define_model_roles! {
    enum ContactRoles for Contact {
        Name(name via QString::from): "name",
        Tel(tel via QString::from):   "tel"
    }
}

impl ContactModel {
    fn format_helper(&self, number: &str, mode: Mode) -> Option<String> {
        let settings = Settings::default();

        let country_code = settings.get_string("country_code");

        let country = match phonenumber::country::Id::from_str(&country_code) {
            Ok(country) => country,
            Err(_) => return None,
        };

        let number = match phonenumber::parse(Some(country), number) {
            Ok(number) => number,
            Err(_) => return None,
        };

        if !phonenumber::is_valid(&number) {
            return None;
        }

        Some(number.format().mode(mode).to_string())
    }

    // The default formatter expected by QML
    fn format(&self, string: QString) -> QString {
        let string = string.to_string();
        let string = string.trim();
        if string.is_empty() {
            return QString::from("");
        }

        let string_with_plus = format!("+{}", string);

        if let Some(number) = self.format_helper(string, Mode::E164) {
            QString::from(number)
        } else if string.starts_with('+') {
            QString::from("")
        } else if let Some(number) = self.format_helper(&string_with_plus, Mode::E164) {
            QString::from(number)
        } else {
            QString::from("")
        }
    }

    fn db(&self) -> SqliteConnection {
        let path = dirs::data_local_dir().expect("find data directory");
        SqliteConnection::establish(path.join(DB_PATH).to_str().expect("UTF-8 path"))
            .expect("open contact database")
    }

    fn name(&self, source: QString) -> QString {
        use crate::schema::contacts;
        use crate::schema::phoneNumbers;

        let source = source.to_string();
        let source = source.trim();

        let conn = self.db(); // This should maybe be established only once

        // This will ensure the format to query is ok
        let e164_source = self
            .format_helper(&source, Mode::E164)
            .unwrap_or_else(|| "".into());
        let mut national_source = self
            .format_helper(&source, Mode::National)
            .unwrap_or_else(|| "".into());
        national_source.retain(|c| c != ' '); // At least FI numbers had spaces after parsing
        let source = source.to_string();

        let (name, _phone_number): (String, String) = contacts::table
            .inner_join(phoneNumbers::table)
            .select((contacts::displayLabel, phoneNumbers::phoneNumber))
            .filter(phoneNumbers::phoneNumber.like(&e164_source))
            .or_filter(phoneNumbers::phoneNumber.like(&national_source))
            .get_result(&conn)
            .unwrap_or((source.clone(), source));

        QString::from(name)
    }

    pub fn refresh(&mut self) {
        log::info!("Refreshing contacts");
        use crate::schema::contacts;
        use crate::schema::phoneNumbers;

        let settings = crate::settings::Settings::default();
        let country_code = settings.get_string("country_code");
        let db = self.db();

        let country = match phonenumber::country::Id::from_str(&country_code) {
            Ok(country) => Some(country),
            Err(()) => {
                log::warn!("Please set country in settings!");
                None
            }
        };

        let contacts: Result<Vec<Contact>, _> = contacts::table
            .inner_join(phoneNumbers::table)
            .select((contacts::displayLabel, phoneNumbers::phoneNumber))
            .order_by(contacts::displayLabel.asc())
            .get_results(&db);

        (self as &mut dyn QAbstractListModel).begin_reset_model();

        match contacts {
            Ok(contacts) => {
                log::info!("Found {} contacts", contacts.len());
                self.content = contacts;
            }
            Err(e) => {
                log::error!("Refreshing contacts {}", e);
                return;
            }
        }

        for contact in self.content.iter_mut() {
            let number = match phonenumber::parse(country, &contact.tel) {
                Ok(number) => number,
                Err(e) => {
                    log::warn!("Could not format phone number: {}", e);
                    continue;
                }
            };
            contact.tel = number.format().mode(phonenumber::Mode::E164).to_string();
        }
        (self as &mut dyn QAbstractListModel).end_reset_model();

        self.contacts_changed();
    }

    fn total(&self) -> i32 {
        // XXX: this should in fact be the amount of *registered* contacts.
        self.row_count()
    }
}

impl QAbstractListModel for ContactModel {
    fn row_count(&self) -> i32 {
        log::trace!("ContactModel::row_count");
        self.content.len() as i32
    }

    fn data(&self, index: QModelIndex, role: i32) -> QVariant {
        log::trace!("ContactModel::data(role={})", role);
        let role = ContactRoles::from(role);
        role.get(&self.content[index.row() as usize])
    }

    fn role_names(&self) -> HashMap<i32, QByteArray> {
        log::trace!("ContactModel::role_names");
        ContactRoles::role_names()
    }
}
