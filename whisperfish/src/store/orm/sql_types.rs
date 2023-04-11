use diesel::{
    backend, deserialize, serialize,
    sql_types::{Integer, Nullable, Text},
};
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
        let uuid = s.as_deref().map(Uuid::parse_str).transpose()?;
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
        let uuid = Uuid::parse_str(&s)?;
        Ok(UuidString(uuid))
    }
}

impl Into<Uuid> for UuidString {
    fn into(self) -> Uuid {
        self.0
    }
}

impl Into<Option<Uuid>> for OptionUuidString {
    fn into(self) -> Option<Uuid> {
        self.0
    }
}
