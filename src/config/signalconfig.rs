use anyhow::Context;
use fs_extra;

/// Global Config
///
/// This struct holds the global configuration of the whisperfish app.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct SignalConfig {
    /// Our telephone number. This field is changed in threads and thus has to be Send/Sync but
    /// mutable at the same time.
    // XXX use the corresponding phonenumber::phonenumber type
    tel: std::sync::Mutex<String>,
    /// Our uuid. This field is changed in threads and thus has to be Send/Sync but mutable at the
    /// same time.
    // XXX use the uuid type here
    uuid: std::sync::Mutex<String>,
    /// Directory for persistent share files
    // XXX share dir is an ugly name, use another one
    // XXX we don't (de-)serialize this field as there is another instance that is accessing the
    // default path in `settings.rs`. As long as `settings.rs` is not accessing this struct, we
    // cannot set this path by a config file.
    #[serde(skip)]
    share_dir: std::path::PathBuf,
    /// Verbosity of the logging messages
    pub verbose: bool,
    /// Whether whisperfish was automatically started (probably systemd) or by the user. We do not
    /// want to serialize this field to the config file. This config is only set by command line
    /// arguments.
    #[serde(skip)]
    pub autostart: bool,

    pub override_captcha: Option<String>,
}

impl Default for SignalConfig {
    fn default() -> Self {
        let path =
            crate::store::default_location().expect("Could not get xdg share directory path");

        Self {
            tel: std::sync::Mutex::new(String::from("")),
            uuid: std::sync::Mutex::new(String::from("")),
            share_dir: path.to_path_buf(),
            verbose: false,
            autostart: false,
            override_captcha: None,
        }
    }
}

impl SignalConfig {
    pub fn migrate_config() -> Result<(), anyhow::Error> {
        let config_dir = dirs::config_dir().expect("No config directory found");

        let old_path = config_dir.join("harbour-whisperfish");

        let new_path = config_dir.join("be.rubdos").join("harbour-whisperfish");

        let old_file = &old_path.join("config.yml");
        let new_file = &new_path.join("config.yml");

        if new_file.exists() {
            return Ok(());
        }

        if !new_path.exists() {
            eprintln!("Creating new config path...");
            std::fs::create_dir_all(&new_path)?;
        }

        if !old_file.exists() {
            if !new_file.exists() {
                eprintln!("Creating empty config file...");
                std::fs::File::create(new_file)?;
                return Ok(());
            }
            eprintln!("Old config doesn't exist, migration not needed");
            return Ok(());
        }

        // Sailjail mounts the old and new paths separately, which makes
        // std::fs::rename fail. That means we have to use copy-and-delete.
        eprintln!("Migrating old config file...");
        std::fs::copy(old_file, new_file)?;
        std::fs::remove_file(old_file)?;
        eprintln!("Config file migrated");
        Ok(())
    }

    pub fn migrate_qsettings() -> Result<(), anyhow::Error> {
        let config_dir = dirs::config_dir().expect("No config directory found");

        let old_path = config_dir.join("harbour-whisperfish");

        let new_path = config_dir.join("be.rubdos").join("harbour-whisperfish");

        let old_file = &old_path.join("harbour-whisperfish.conf");
        let new_file = &new_path.join("harbour-whisperfish.conf");

        if new_file.exists() {
            return Ok(());
        }

        if !new_path.exists() {
            eprintln!("Creating new config path...");
            std::fs::create_dir_all(&new_path)?;
        }

        if !old_file.exists() {
            if !new_file.exists() {
                eprintln!("Creating empty QSettings file...");
                std::fs::File::create(new_file)?;
                return Ok(());
            }
            eprintln!("Old QSettings file doesn't exist, migration not needed");
            return Ok(());
        }

        // Sailjail mounts the old and new paths separately, which makes
        // std::fs::rename fail. That means we have to use copy-and-delete.
        eprintln!("Migrating old QSettings file...");
        std::fs::copy(old_file, new_file)?;
        std::fs::remove_file(old_file)?;
        eprintln!("QSettings file migrated");
        Ok(())
    }

    // XXX: This probably shouldn't be in signalconfig.rs
    pub fn migrate_storage() -> Result<(), anyhow::Error> {
        let data_dir = dirs::data_local_dir().context("No data directory found")?;

        let old_path = data_dir.join("harbour-whisperfish");
        let old_db = &old_path.join("db");
        let old_storage = &old_path.join("storage");

        let new_path = data_dir.join("be.rubdos").join("harbour-whisperfish");
        let new_db = &new_path.join("db");
        let new_storage = &new_path.join("storage");

        if !new_path.exists() {
            eprintln!("Creating new storage path...");
            std::fs::create_dir_all(&new_path)?;
        }

        if new_db.exists() && new_storage.exists() {
            return Ok(());
        } else if !new_db.exists()
            && !new_storage.exists()
            && !old_db.exists()
            && !old_storage.exists()
        {
            eprintln!("Creating storage and db folders...");
            std::fs::create_dir(new_db)?;
            std::fs::create_dir(new_storage)?;
            return Ok(());
        } else if (new_db.exists() ^ new_storage.exists())
            || (old_db.exists() ^ old_storage.exists())
        {
            eprintln!("Storage state is abnormal, aborting!");
            eprintln!("new db exists: {}", new_db.exists());
            eprintln!("new storage exists: {}", new_storage.exists());
            eprintln!("old db exists: {}", old_db.exists());
            eprintln!("old storage exists: {}", old_storage.exists());
            std::process::exit(1);
        }

        // Sailjail mounts the old and new paths separately, which makes
        // std::fs::rename fail. That means we have to copy-and-delete
        // recursively instead, handled by fs_extra::dir::move_dir.
        let options = fs_extra::dir::CopyOptions::new();
        eprintln!("Migrating old db folder...");
        fs_extra::dir::move_dir(&old_db, &new_path, &options)?;
        eprintln!("Migrating old storage folder...");
        fs_extra::dir::move_dir(&old_storage, &new_path, &options)?;
        eprintln!("Storage folders migrated");
        Ok(())
    }

    pub fn read_from_file() -> Result<Self, anyhow::Error> {
        let path = dirs::config_dir()
            .context("Could not get xdg config directory path")?
            .join("be.rubdos")
            .join("harbour-whisperfish")
            .join("config.yml");

        let fd = std::fs::File::open(&path)
            .with_context(|| format!("Could not open config file: {}", &path.display()))?;
        let ret = serde_yaml::from_reader(fd)
            .with_context(|| format!("Could not read config file: {}", &path.display()))?;

        Ok(ret)
    }

    pub fn write_to_file(&self) -> Result<(), anyhow::Error> {
        let path = dirs::config_dir()
            // XXX use anyhow context here
            .expect("No config directory found")
            .join("be.rubdos")
            .join("harbour-whisperfish");

        // create config directory if it does not exist
        if !path.exists() {
            std::fs::create_dir_all(&path).with_context(|| {
                format!("Could not create config directory: {}", &path.display())
            })?;
        }

        // write to config file
        let path = path.join("config.yml");
        let fd = std::fs::File::create(&path)
            .with_context(|| format!("Could not open config file to write: {}", &path.display()))?;
        serde_yaml::to_writer(fd, &self)
            .with_context(|| format!("Could not write config file: {}", &path.display()))?;

        Ok(())
    }

    pub fn get_share_dir(&self) -> std::path::PathBuf {
        self.share_dir.to_owned()
    }

    // XXX should be deprecated / removed
    pub fn get_storage_dir(&self) -> std::path::PathBuf {
        self.share_dir.join("storage")
    }

    pub fn default_attachment_dir(&self) -> std::path::PathBuf {
        self.share_dir.join("storage").join("attachments")
    }

    pub fn default_camera_dir(&self) -> std::path::PathBuf {
        self.share_dir.join("storage").join("camera")
    }

    pub fn get_identity_dir(&self) -> std::path::PathBuf {
        self.share_dir
            .join("storage")
            .join("identity")
            .join("identity_key")
    }

    pub fn get_tel_clone(&self) -> String {
        self.tel.lock().unwrap().clone()
    }

    pub fn get_uuid_clone(&self) -> String {
        self.uuid.lock().unwrap().clone()
    }

    pub fn set_tel(&self, tel: String) {
        *self.tel.lock().unwrap() = tel;
    }

    pub fn set_uuid(&self, uuid: String) {
        *self.uuid.lock().unwrap() = uuid;
    }
}
