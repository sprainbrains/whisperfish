use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::schema::message;
use crate::schema::sentq;
use crate::schema::session;

use diesel::prelude::*;
use failure::*;

#[derive(actix::Message, Clone)]
#[rtype(result = "()")]
pub struct StorageReady(pub Storage);

/// Session as it relates to the schema
#[derive(Queryable)]
pub struct Session {
    pub id: i64,
    pub source: String,
    pub message: String,
    pub timestamp: i64,
    pub sent: bool,
    pub received: bool,
    pub unread: bool,
    pub is_group: bool,
    pub group_members: Option<String>,
    #[allow(dead_code)]
    pub group_id: Option<String>,
    pub group_name: Option<String>,
    pub has_attachment: bool,
}

/// ID-free Session model for insertions
#[derive(Insertable)]
#[table_name = "session"]
pub struct NewSession {
    pub source: String,
    pub message: String,
    pub timestamp: i64,
    pub sent: bool,
    pub received: bool,
    pub unread: bool,
    pub is_group: bool,
    pub group_members: Option<String>,
    #[allow(dead_code)]
    pub group_id: Option<String>,
    pub group_name: Option<String>,
    pub has_attachment: bool,
}

/// Message as it relates to the schema
#[derive(Queryable)]
pub struct Message {
    pub id: i32,
    pub sid: i64,
    pub source: String,
    pub message: String,  // NOTE: "text" in schema, doesn't apparently matter
    pub timestamp: i64,
    pub sent: bool,
    pub received: bool,
    pub flags: i32,
    pub attachment: Option<String>,
    pub mimetype: Option<String>,
    pub hasattachment: bool,
    pub outgoing: bool,
    pub queued: bool,
}

/// ID-free Message model for insertions
#[derive(Insertable)]
#[table_name = "message"]
pub struct NewMessage {
    pub session_id: i64,
    pub source: String,
    pub text: String,
    pub timestamp: i64,
    pub sent: bool,
    pub received: bool,
    pub flags: i32,
    pub attachment: Option<String>,
    pub mime_type: Option<String>,
    pub has_attachment: bool,
    pub outgoing: bool,
}

/// Location of the storage.
///
/// Path is for persistent storage.
/// Memory is for running tests or 'incognito' mode.
#[cfg_attr(not(test), allow(unused))]
pub enum StorageLocation<P> {
    Path(P),
    Memory,
}

impl<'a> From<&'a Path> for StorageLocation<&'a Path> {
    fn from(p: &'a Path) -> Self {
        StorageLocation::Path(p)
    }
}

impl From<PathBuf> for StorageLocation<PathBuf> {
    fn from(p: PathBuf) -> Self {
        StorageLocation::Path(p)
    }
}

#[cfg_attr(not(test), allow(unused))]
pub fn memory() -> StorageLocation<PathBuf> {
    StorageLocation::Memory
}

pub fn default_location() -> Result<StorageLocation<PathBuf>, Error> {
    let data_dir = dirs::data_local_dir().ok_or(format_err!("Could not find data directory."))?;

    Ok(StorageLocation::Path(
        data_dir.join("harbour-whisperfish").into(),
    ))
}

impl std::ops::Deref for StorageLocation<PathBuf> {
    type Target = Path;
    fn deref(&self) -> &Path {
        match self {
            StorageLocation::Memory => unimplemented!(":memory: deref"),
            StorageLocation::Path(p) => p,
        }
    }
}

impl<P: AsRef<Path>> StorageLocation<P> {
    fn open_db(&self) -> Result<SqliteConnection, Error> {
        let database_url = match self {
            StorageLocation::Memory => ":memory:".into(),
            StorageLocation::Path(p) => p
                .as_ref()
                .join("db")
                .join("harbour-whisperfish.db")
                .to_str()
                .ok_or(format_err!(
                    "path to db contains a non-UTF8 character, please file a bug."
                ))?
                .to_string(),
        };

        Ok(SqliteConnection::establish(&database_url)?)
    }
}

#[derive(Clone)]
pub struct Storage {
    pub db: Arc<Mutex<SqliteConnection>>,
    // aesKey + macKey
    keys: Option<[u8; 16 + 20]>,
}

// Cannot borrow password/salt because threadpool requires 'static...
async fn derive_storage_key(password: String, salt_path: PathBuf) -> Result<[u8; 16 + 20], Error> {
    use actix_threadpool::BlockingError;
    use std::io::Read;

    actix_threadpool::run(move || -> Result<_, failure::Error> {
        let mut salt_file = std::fs::File::open(salt_path)?;
        let mut salt = [0u8; 8];
        ensure!(salt_file.read(&mut salt)? == 8, "salt file not 8 bytes");

        let mut key = [0u8; 16 + 20];
        // Please don't blame me, I'm only the implementer.
        pbkdf2::pbkdf2::<hmac::Hmac<sha1::Sha1>>(password.as_bytes(), &salt, 1024, &mut key);
        log::trace!("Computed the key, salt was {:?}", salt);

        Ok(key)
    })
    .await
    .map_err(|e| match e {
        BlockingError::Canceled => format_err!("Threadpool Canceled"),
        BlockingError::Error(e) => e,
    })
}

// Cannot borrow password/salt because threadpool requires 'static...
async fn derive_db_key(password: String, salt_path: PathBuf) -> Result<[u8; 32], Error> {
    use actix_threadpool::BlockingError;
    use std::io::Read;

    actix_threadpool::run(move || -> Result<_, failure::Error> {
        let mut salt_file = std::fs::File::open(salt_path)?;
        let mut salt = [0u8; 8];
        ensure!(salt_file.read(&mut salt)? == 8, "salt file not 8 bytes");

        let params = scrypt::ScryptParams::new(14, 8, 1)?;
        let mut key = [0u8; 32];
        scrypt::scrypt(password.as_bytes(), &salt, &params, &mut key)?;
        log::trace!("Computed the key, salt was {:?}", salt);
        Ok(key)
    })
    .await
    .map_err(|e| match e {
        BlockingError::Canceled => format_err!("Threadpool Canceled"),
        BlockingError::Error(e) => e,
    })
}

impl Storage {
    pub fn open<T: AsRef<Path>>(db_path: &StorageLocation<T>) -> Result<Storage, Error> {
        let db = db_path.open_db()?;

        Ok(Storage {
            db: Arc::new(Mutex::new(db)),
            keys: None,
        })
    }

    pub async fn open_with_password<T: AsRef<Path>>(
        db_path: &StorageLocation<T>,
        password: String,
    ) -> Result<Storage, Error> {
        let db_salt_path = crate::store::default_location()
            .unwrap()
            .join("db")
            .join("salt");
        let storage_salt_path = crate::store::default_location()
            .unwrap()
            .join("storage")
            .join("salt");
        // XXX: The storage_key could already be polled while we're querying the database,
        // but we don't want to wait for it either.
        let db_key = derive_db_key(password.clone(), db_salt_path);
        let storage_key = derive_storage_key(password, storage_salt_path);

        // 1. decrypt DB
        let db = db_path.open_db()?;
        db.execute(&format!("PRAGMA key = \"x'{}'\";", hex::encode(db_key.await?)))?;
        db.execute("PRAGMA cipher_page_size = 4096;")?;

        // From the sqlcipher manual:
        // -- if this throws an error, the key was incorrect. If it succeeds and returns a numeric value, the key is correct;
        db.execute("SELECT count(*) FROM sqlite_master;")?;
        // XXX: Do we have to signal somehow that the password was wrong?
        //      Offer retries?

        // 2. decrypt storage
        let keys = Some(storage_key.await?);

        Ok(Storage {
            db: Arc::new(Mutex::new(db)),
            keys,
        })
    }

    /// Asynchronously loads the signal HTTP password from storage and decrypts it.
    pub async fn signal_password(&self) -> Result<String, Error> {
        let http_password_path = crate::store::default_location()
            .unwrap()
            .join("storage")
            .join("identity")
            .join("http_password");
        // XXX: unencrypted storage.
        let keys = self.keys.unwrap();

        let password = actix_threadpool::run(move || {
            use std::io::Read;

            // XXX This is *full* of bad practices.
            // Let's try to migrate to nacl or something alike in the future.

            let mut iv = [0u8; 16];
            let mut password = [0u8; 32];
            let mut mac = [0u8; 32];

            let mut f = std::fs::File::open(http_password_path)?;
            ensure!(f.read(&mut iv)? == 16, "IV not 16 bytes");
            ensure!(f.read(&mut password)? == 32, "password not 32 bytes");
            ensure!(f.read(&mut mac)? == 32, "mac not 32 bytes");

            {
                use hmac::{Hmac, Mac, NewMac};
                use sha2::Sha256;
                // Verify HMAC SHA256, 32 last bytes
                let mut verifier = Hmac::<Sha256>::new_varkey(&keys[16..])
                    .map_err(|_| format_err!("MAC keylength error"))?;
                verifier.update(&iv);
                verifier.update(&password);
                verifier
                    .verify(&mac)
                    .map_err(|_| format_err!("MAC error"))?;
            }

            let password = {
                use aes::Aes128;
                use block_modes::block_padding::Pkcs7;
                use block_modes::{BlockMode, Cbc};
                // Decrypt password
                let cipher = Cbc::<Aes128, Pkcs7>::new_var(&keys[0..16], &iv)
                    .map_err(|_| format_err!("CBC initialization error"))?;
                cipher
                    .decrypt(&mut password)
                    .map_err(|_| format_err!("AES CBC decryption error"))?
            };
            let password = std::str::from_utf8(password)?.to_owned();
            Ok(password)
        })
        .await?;

        Ok(password)
    }

    /// Process message and store in database and update or create a session
    /// TODO: textsecure Group not implemented
    pub fn process_message(&self, mut new_message: NewMessage, is_unread: bool) {
        let group = false;  // TODO: textsecure Group not implemented

        let db_session_res = if !group {
            self.fetch_session_by_source(&new_message.source)
        } else {
            panic!("textsecure Group not implemented: fetch_session_by_group")
        };

        // Initialize the session data to work with, modify it in case of a group
        let mut session_data = NewSession {
            source: new_message.source.clone(),
            message: new_message.text.clone(),
            timestamp: new_message.timestamp,
            sent: new_message.sent,
            received: new_message.received,
            unread: is_unread,
            has_attachment: new_message.has_attachment,
            is_group: false,
            group_id: None,
            group_name: None,
            group_members: None,
        };

        if group {
            session_data.is_group = true;
            session_data.group_id = Some(String::from("TODO"));
            session_data.group_name = Some(String::from("TODO"));
            session_data.group_members = Some(String::from("TODO"));
        }

        let db_session: Session = if db_session_res.is_some() {
            let db_session = db_session_res.unwrap();
            self.update_session(&db_session, &session_data, is_unread);
            db_session
        } else {
            self.create_session(&session_data)
                .expect("Unable to create session yet create_session() did not panic")
        };

        new_message.session_id = db_session.id;

        // With the prepared new_message in hand, see if it's an update or a new one
        let update_msg_res = self.update_message_if_needed(&new_message);

        if update_msg_res.is_none() {
            self.create_message(&new_message);
        };
    }

    /// Create a new session. This was transparent within SaveSession in Go.
    ///
    /// It needs to be locked from the outside because sqlite sucks.
    pub fn create_session(&self, new_session: &NewSession) -> Option<Session> {
        use crate::schema::session::dsl as schema_dsl;

        let db = self.db.lock();
        let conn = db.unwrap();

        log::trace!("Called create_session()");

        let query = diesel::insert_into(schema_dsl::session).values(new_session);

        let res = query.execute(&*conn).expect("inserting a session");

        // Then see if the session was inserted ok and what it was
        drop(conn);  // Connection must be dropped because everyone wants a lock here
        let latest_session_res = self.fetch_latest_session();

        if res != 1 || latest_session_res.is_none() {
            panic!("Non-error non-insert!")
        }

        let latest_session = latest_session_res.unwrap();

        // XXX: This is checking that we got the latest one we expect,
        //      because sqlite sucks and some other thread might have inserted
        if latest_session.timestamp != new_session.timestamp ||
            latest_session.source != new_session.source {
                panic!("Could not match latest session to this one!
                       latest.source {} == new.source {} | latest.tstamp {} == new.timestamp {}",
                       latest_session.source, new_session.source,
                       latest_session.timestamp, new_session.timestamp);
            }

        // Better hope something panicked before now if something went wrong
        Some(latest_session)
    }

    /// Update an existing session. This was transparent within SaveSession in Go.
    ///
    /// It needs to be locked from the outside because sqlite sucks.
    /// Also with better schema design this whole thing would be moot!
    pub fn update_session(&self, db_session: &Session, new_session: &NewSession, is_unread: bool) {
        let db = self.db.lock();
        let conn = db.unwrap();

        log::trace!("Called update_session()");

        let query = diesel::update(session::table.filter(session::id.eq(db_session.id))).set((
            session::message.eq(&new_session.message),
            session::timestamp.eq(new_session.timestamp),
            session::unread.eq(is_unread),
            session::sent.eq(new_session.sent),
            session::received.eq(new_session.received),
            session::has_attachment.eq(new_session.has_attachment),
        ));
        query.execute(&*conn).expect("updating session");
    }

    /// This was implicit in Go, which probably didn't use threads.
    ///
    /// It needs to be locked from the outside because sqlite sucks.
    pub fn fetch_latest_session(&self) -> Option<Session> {
        let db = self.db.lock();
        let conn = db.unwrap();

        log::trace!("Called fetch_latest_session()");
        session::table.order_by(session::columns::id.desc())
                      .first(&*conn)
                      .ok()
    }

    pub fn fetch_session(&self, sid: i64) -> Option<Session> {
        let db = self.db.lock();
        let conn = db.unwrap();

        log::trace!("Called fetch_session({})", sid);
        session::table.filter(session::columns::id.eq(sid))
                      .first(&*conn)
                      .ok()
    }

    pub fn fetch_session_by_source(&self, source: &str) -> Option<Session> {
        let db = self.db.lock();
        let conn = db.unwrap();

        log::trace!("Called fetch_session_by_source({})", source);
        session::table.filter(session::columns::source.eq(source))
                      .first(&*conn)
                      .ok()
    }

    /// Check if message exists and explicitly update it if required
    ///
    /// This is because during development messages may come in partially
    fn update_message_if_needed(&self, new_message: &NewMessage) -> Option<Message> {
        let db = self.db.lock();
        let conn = db.unwrap();

        use diesel::expression::sql_literal::sql;

        log::trace!("Called update_message_if_needed({})", new_message.session_id);

        let mut msg: Message = message::table.left_join(sentq::table)
                                             .select((message::columns::id, message::columns::session_id, message::columns::source,
                                                      message::columns::text, message::columns::timestamp, message::columns::sent,
                                                      message::columns::received, message::columns::flags, message::columns::attachment,
                                                      message::columns::mime_type, message::columns::has_attachment, message::columns::outgoing,
                                             sql::<diesel::sql_types::Bool>("CASE WHEN sentq.message_id > 0 THEN 1 ELSE 0 END AS queued")))
                                             .filter(message::columns::session_id.eq(new_message.session_id))
                                             .filter(message::columns::timestamp.eq(new_message.timestamp))
                                             .filter(message::columns::text.eq(&new_message.text))
                                             .order_by(message::columns::id.desc())
                                             .first(&*conn)
                                             .ok()?;

        // Do not update `(session_id, timestamp, message)` because that's considered unique
        // nor `source` which is correlated with `session_id`
        if msg.sent != new_message.sent ||
            msg.received != new_message.received ||
            msg.flags != new_message.flags ||
            msg.attachment != new_message.attachment ||
            msg.mimetype != new_message.mime_type ||
            msg.hasattachment != new_message.has_attachment ||
            msg.outgoing != new_message.outgoing {
                let query = diesel::update(message::table.filter(message::id.eq(msg.id))).set((
                    message::sent.eq(new_message.sent),
                    message::received.eq(new_message.received),
                    message::flags.eq(new_message.flags),
                    message::attachment.eq(&new_message.attachment),
                    message::mime_type.eq(&new_message.mime_type),
                    message::has_attachment.eq(new_message.has_attachment),
                    message::outgoing.eq(new_message.outgoing),
                ));

                query.execute(&*conn).expect("updating message");

                // Also update the message we got from the db to match what was updated
                msg.sent = new_message.sent;
                msg.received = new_message.received;
                msg.flags = new_message.flags;
                msg.attachment = new_message.attachment.clone();
                msg.mimetype = new_message.mime_type.clone();
                msg.hasattachment = new_message.has_attachment;
                msg.outgoing = new_message.outgoing;
            }

        Some(msg)
    }

    /// Create a new message. This was transparent within SaveMessage in Go.
    pub fn create_message(&self, new_message: &NewMessage) -> Message {
        use crate::schema::message::dsl as schema_dsl;

        let db = self.db.lock();
        let conn = db.unwrap();

        log::trace!("Called create_message()");

        let query = diesel::insert_into(schema_dsl::message).values(new_message);

        let res = query.execute(&*conn).expect("inserting a message");

        // Then see if the message was inserted ok and what it was
        drop(conn);  // Connection must be dropped because everyone wants a lock here
        let latest_message_res = self.fetch_latest_message();

        if res != 1 || latest_message_res.is_none() {
            panic!("Non-error non-insert!")
        }

        let latest_message = latest_message_res.unwrap();

        // XXX: This is checking that we got the latest one we expect,
        //      because sqlite sucks and some other thread might have inserted

        if latest_message.timestamp != new_message.timestamp ||
            latest_message.source != new_message.source {
                panic!("Could not match latest message to this one!
                       latest.source {} == new.source {} | latest.tstamp {} == new.timestamp {}",
                       latest_message.source, new_message.source,
                       latest_message.timestamp, new_message.timestamp);
            }


        log::trace!("Inserted message id {}", latest_message.id);
        latest_message
    }

    /// This was implicit in Go, which probably didn't use threads.
    ///
    /// It needs to be locked from the outside because sqlite sucks.
    pub fn fetch_latest_message(&self) -> Option<Message> {
        let db = self.db.lock();
        let conn = db.unwrap();

        use diesel::expression::sql_literal::sql;

        log::trace!("Called fetch_latest_message()");
        message::table.left_join(sentq::table)
                      .select((message::columns::id, message::columns::session_id, message::columns::source,
                               message::columns::text, message::columns::timestamp, message::columns::sent,
                               message::columns::received, message::columns::flags, message::columns::attachment,
                               message::columns::mime_type, message::columns::has_attachment, message::columns::outgoing,
                      sql::<diesel::sql_types::Bool>("CASE WHEN sentq.message_id > 0 THEN 1 ELSE 0 END AS queued")))
                      .order_by(message::columns::id.desc())
                      .first(&*conn)
                      .ok()
    }

    pub fn fetch_all_messages(&self, sid: i64) -> Option<Vec<Message>> {
        let db = self.db.lock();
        let conn = db.unwrap();

        use diesel::expression::sql_literal::sql;
        use diesel::debug_query;

        log::trace!("Called fetch_all_messages({})", sid);
        let query = message::table.left_join(sentq::table)
                            .select((message::columns::id, message::columns::session_id, message::columns::source,
                                     message::columns::text, message::columns::timestamp, message::columns::sent,
                                     message::columns::received, message::columns::flags, message::columns::attachment,
                                     message::columns::mime_type, message::columns::has_attachment, message::columns::outgoing,
                            sql::<diesel::sql_types::Bool>("CASE WHEN sentq.message_id > 0 THEN 1 ELSE 0 END AS queued")))
                            .filter(message::columns::session_id.eq(sid))
                            .order_by(message::columns::id.desc());

        let debug = debug_query::<diesel::sqlite::Sqlite, _>(&query);
        log::trace!("{}", debug.to_string());

        query.load::<Message>(&*conn).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_memory_db() -> Result<(), Error> {
        let _storage = Storage::open(&memory())?;

        Ok(())
    }
}
