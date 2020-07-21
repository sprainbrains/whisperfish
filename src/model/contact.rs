use std::collections::HashMap;
use std::str::FromStr;

use crate::settings::*;
use crate::sfos::SailfishApp;

use actix::prelude::*;
use diesel::prelude::*;
use phonenumber::Mode;
use qmetaobject::*;

const DB_PATH: &str = "/home/nemo/.local/share/system/Contacts/qtcontacts-sqlite/contacts.db";

#[derive(QObject, Default)]
pub struct ContactModel {
    base: qt_base_class!(trait QAbstractListModel),
    actor: Option<Addr<ContactActor>>,

    content: Vec<Contact>,

    format: qt_method!(fn(&self, string: QString) -> QString),
    name: qt_method!(fn(&self, source: QString) -> QString),
}

pub struct ContactActor {
    inner: QObjectBox<ContactModel>,
}

#[derive(Queryable)]
pub struct Contact {
    name: String,
    tel: String,
}

impl ContactActor {
    pub fn new(app: &mut SailfishApp) -> Self {
        let inner = QObjectBox::new(ContactModel::default());
        app.set_object_property("ContactModel".into(), inner.pinned());

        Self { inner }
    }
}

impl Actor for ContactActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.inner.pinned().borrow_mut().actor = Some(ctx.address());
    }
}

define_model_roles! {
    enum ContactRoles for Contact {
        Name(name via QString::from): "name",
        Tel(tel via QString::from):   "tel"
    }
}

impl ContactModel {
    fn format_helper(&self, number: &QString, mode: Mode) -> String {
        let settings = Settings::default();

        let country_code = settings.get_string("country_code");

        let country = match phonenumber::country::Id::from_str(&country_code) {
            Ok(country) => country,
            Err(_) => return String::new(),
        };

        let number = match phonenumber::parse(Some(country), number.to_string()) {
            Ok(number) => number,
            Err(_) => return String::new(),
        };

        let is_valid = phonenumber::is_valid(&number);

        if !is_valid {
            return String::new();
        }

        String::from(number.format().mode(mode).to_string())
    }

    // The default formatter expected by QML
    fn format(&self, string: QString) -> QString {
        if string.to_string().len() == 0 {
            return QString::from("");
        }

        QString::from(self.format_helper(&string, Mode::E164))
    }

    fn name(&self, source: QString) -> QString {
        use crate::schema::contacts;
        use crate::schema::phoneNumbers;

        let db = SqliteConnection::establish(DB_PATH); // This should maybe be established only once
        let conn = db.unwrap();

        // This will ensure the format to query is ok
        let e164_source = self.format_helper(&source, Mode::E164);
        let mut national_source = self.format_helper(&source, Mode::National);
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
}

impl QAbstractListModel for ContactModel {
    fn row_count(&self) -> i32 {
        self.content.len() as i32
    }

    fn data(&self, index: QModelIndex, role: i32) -> QVariant {
        let role = ContactRoles::from(role);
        role.get(&self.content[index.row() as usize])
    }
}
