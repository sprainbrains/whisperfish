use libsignal_service::protocol::*;
use std::{path::PathBuf, sync::Arc};
use structopt::StructOpt;
use whisperfish::{config::SignalConfig, store};

/// Initializes a storage, meant for creating storage migration tests.
#[derive(StructOpt, Debug)]
#[structopt(name = "create-store")]
struct Opt {
    /// Whisperfish storage password
    #[structopt(short, long)]
    password: Option<String>,

    /// Path where the storage will be created
    #[structopt(parse(from_os_str))]
    path: PathBuf,

    /// Whether to fill the storage with dummy data
    #[structopt(short, long)]
    fill_dummy: bool,
}

async fn create_storage(
    config: Arc<SignalConfig>,
    storage_password: Option<&str>,
    path: store::StorageLocation<PathBuf>,
) -> store::Storage {
    use rand::{Rng, RngCore};
    let rng = rand::thread_rng();

    // Signaling password for REST API
    let password: String = rng
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(24)
        .collect();

    // Signaling key that decrypts the incoming Signal messages
    let mut rng = rand::thread_rng();
    let mut signaling_key = [0u8; 52];
    rng.fill_bytes(&mut signaling_key);
    let signaling_key = signaling_key;

    // Registration ID
    let regid: u32 = 12345;
    let pni_regid: u32 = 12346;

    store::Storage::new(
        config,
        &path,
        storage_password,
        regid,
        pni_regid,
        &password,
        signaling_key,
        None,
        None,
    )
    .await
    .unwrap()
}

async fn add_dummy_data(storage: &mut store::Storage) {
    use std::str::FromStr;
    let mut rng = rand::thread_rng();

    // Invent two users with devices
    let user_id = uuid::Uuid::from_str("5844fce4-4407-401a-9dbc-fc86c6def4e6").unwrap();
    let device_id = 1;
    let addr_1 = ProtocolAddress::new(user_id.to_string(), DeviceId::from(device_id));

    let user_id = uuid::Uuid::from_str("7bec59e1-140d-4b53-98f1-dc8fd2c011c8").unwrap();
    let device_id = 2;
    let addr_2 = ProtocolAddress::new(user_id.to_string(), DeviceId::from(device_id));

    let device_id = 3;
    let addr_3 = ProtocolAddress::new("+32412345678".into(), DeviceId::from(device_id));

    // Create two identities and two sessions
    let key_1 = IdentityKeyPair::generate(&mut rng);
    let key_2 = IdentityKeyPair::generate(&mut rng);
    let key_3 = IdentityKeyPair::generate(&mut rng);

    storage
        .save_identity(&addr_1, key_1.identity_key(), None)
        .await
        .unwrap();
    storage
        .save_identity(&addr_2, key_2.identity_key(), None)
        .await
        .unwrap();
    storage
        .save_identity(&addr_3, key_3.identity_key(), None)
        .await
        .unwrap();

    let session_1 = SessionRecord::new_fresh();
    let session_2 = SessionRecord::new_fresh();
    let session_3 = SessionRecord::new_fresh();
    storage
        .store_session(&addr_1, &session_1, None)
        .await
        .unwrap();
    storage
        .store_session(&addr_2, &session_2, None)
        .await
        .unwrap();
    storage
        .store_session(&addr_3, &session_3, None)
        .await
        .unwrap();
}

#[actix_rt::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    let opt = Opt::from_args();

    // TODO: probably source more config flags, see harbour-whisperfish main.rs
    let config = match whisperfish::config::SignalConfig::read_from_file() {
        Ok(x) => x,
        Err(e) => {
            eprintln!("Config file not found: {}", e);
            whisperfish::config::SignalConfig::default()
        }
    };
    let config = Arc::new(config);

    let path = opt.path;
    let mut store = create_storage(config, opt.password.as_deref(), path.into()).await;

    if opt.fill_dummy {
        add_dummy_data(&mut store).await;
    }

    Ok(())
}
