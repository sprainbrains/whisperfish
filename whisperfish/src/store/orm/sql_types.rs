use diesel::{backend, deserialize, serialize, sql_types::Integer};

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
