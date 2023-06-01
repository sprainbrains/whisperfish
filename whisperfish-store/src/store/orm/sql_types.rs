use diesel::{
    backend, deserialize, serialize,
    sql_types::{Integer, Nullable, Text},
};
use phonenumber::PhoneNumber;
use uuid::Uuid;

use super::UnidentifiedAccessMode;

impl<DB> deserialize::FromSql<Integer, DB> for UnidentifiedAccessMode
where
    DB: backend::Backend,
    i32: deserialize::FromSql<Integer, DB>,
{
    fn from_sql(bytes: backend::RawValue<DB>) -> deserialize::Result<Self> {
        match i32::from_sql(bytes)? {
            0 => Ok(UnidentifiedAccessMode::Unknown),
            1 => Ok(UnidentifiedAccessMode::Disabled),
            2 => Ok(UnidentifiedAccessMode::Enabled),
            3 => Ok(UnidentifiedAccessMode::Unrestricted),
            x => Err(format!("Unrecognized variant {}", x).into()),
        }
    }
}

impl serialize::ToSql<Integer, diesel::sqlite::Sqlite> for UnidentifiedAccessMode
where
    i32: serialize::ToSql<Integer, diesel::sqlite::Sqlite>,
{
    fn to_sql<'b>(
        &'b self,
        out: &mut serialize::Output<'b, '_, diesel::sqlite::Sqlite>,
    ) -> serialize::Result {
        out.set_value(*self as i32);
        Ok(serialize::IsNull::No)
    }
}

pub struct OptionUuidString(Option<Uuid>);
pub struct UuidString(Uuid);

impl<DB> deserialize::Queryable<Nullable<Text>, DB> for OptionUuidString
where
    DB: backend::Backend,
    Option<String>: deserialize::FromSql<Nullable<Text>, DB>,
{
    type Row = Option<String>;

    fn build(s: Option<String>) -> diesel::deserialize::Result<Self> {
        log::trace!("OptionUuidString: deserializing {:?}", s);
        let uuid = s
            .as_deref()
            .filter(|x| !x.is_empty())
            .map(Uuid::parse_str)
            .transpose()?;
        Ok(OptionUuidString(uuid))
    }
}

impl<DB> deserialize::Queryable<Text, DB> for UuidString
where
    DB: backend::Backend,
    String: deserialize::FromSql<Text, DB>,
{
    type Row = String;

    fn build(s: String) -> diesel::deserialize::Result<Self> {
        log::trace!("UuidString: deserializing {}", s);
        let uuid = Uuid::parse_str(&s)?;
        Ok(UuidString(uuid))
    }
}

impl From<UuidString> for Uuid {
    fn from(val: UuidString) -> Self {
        val.0
    }
}

impl From<OptionUuidString> for Option<Uuid> {
    fn from(val: OptionUuidString) -> Self {
        val.0
    }
}

pub struct OptionPhoneNumberString(Option<PhoneNumber>);
pub struct PhoneNumberString(PhoneNumber);

impl<DB> deserialize::Queryable<Nullable<Text>, DB> for OptionPhoneNumberString
where
    DB: backend::Backend,
    Option<String>: deserialize::FromSql<Nullable<Text>, DB>,
{
    type Row = Option<String>;

    fn build(s: Option<String>) -> diesel::deserialize::Result<Self> {
        log::trace!("OptionPhoneNumberString: deserializing {:?}", s);
        let phonenumber = s
            .as_deref()
            // XXX: a migration should be made to set these to NULL instead in the db.
            .filter(|x| !x.is_empty())
            .map(|s| phonenumber::parse(None, s))
            .transpose()?;
        Ok(OptionPhoneNumberString(phonenumber))
    }
}

impl<DB> deserialize::Queryable<Text, DB> for PhoneNumberString
where
    DB: backend::Backend,
    String: deserialize::FromSql<Text, DB>,
{
    type Row = String;

    fn build(s: String) -> diesel::deserialize::Result<Self> {
        log::trace!("PhoneNumberString: deserializing {}", s);
        let uuid = phonenumber::parse(None, s)?;
        Ok(PhoneNumberString(uuid))
    }
}

impl From<PhoneNumberString> for PhoneNumber {
    fn from(val: PhoneNumberString) -> Self {
        val.0
    }
}

impl From<OptionPhoneNumberString> for Option<PhoneNumber> {
    fn from(val: OptionPhoneNumberString) -> Self {
        val.0
    }
}
