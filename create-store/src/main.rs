use libsignal_protocol::DeviceId;
use std::path::PathBuf;
use whisperfish::store;

const HELP: &str = "Initializes a storage, meant for creating storage migration tests.

USAGE:
  create-store [OPTIONS] [--password <password>] --path <path> [--fill-dummy]

FLAGS:
  -h, --help
        Prints help information

OPTIONS:
  --password <password>
        Whisperfish storage password

  --path <path>
        Path where the storage will be created

  --fill-dummy
        Whether to fill the storage with dummy data
";
#[derive(Debug)]
struct Opts {
    password: Option<String>,
    path: PathBuf,
    fill_dummy: bool,
}

fn parse_args() -> Result<Opts, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    // Help has a higher priority and should be handled separately.
    if pargs.contains(["-h", "--help"]) {
        println!("{}", HELP);
        std::process::exit(0);
    }

    let args = Opts {
        password: pargs.opt_value_from_str("--password")?,
        path: {
            let path_arg: String = pargs.value_from_str("--path")?;
            PathBuf::from(path_arg)
        },
        fill_dummy: pargs.contains("--fill-dummy"),
    };

    let remaining = pargs.finish();
    if !remaining.is_empty() {
        eprintln!("Error: unused arguments: {:?}.", remaining);
        std::process::exit(1);
    }

    Ok(args)
}

async fn create_storage(
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

    store::Storage::new(
        &path,
        storage_password,
        regid,
        &password,
        signaling_key,
        None,
    )
    .await
    .unwrap()
}

async fn add_dummy_data(storage: &mut store::Storage) {
    use libsignal_service::prelude::protocol::{IdentityKeyStore, SessionStore};
    use std::str::FromStr;
    let mut rng = rand::thread_rng();

    // Invent two users with devices
    let user_id = uuid::Uuid::from_str("5844fce4-4407-401a-9dbc-fc86c6def4e6").unwrap();
    let device_id = 1;
    let addr_1 = libsignal_service::prelude::protocol::ProtocolAddress::new(
        user_id.to_string(),
        DeviceId::from(device_id),
    );

    let user_id = uuid::Uuid::from_str("7bec59e1-140d-4b53-98f1-dc8fd2c011c8").unwrap();
    let device_id = 2;
    let addr_2 = libsignal_service::prelude::protocol::ProtocolAddress::new(
        user_id.to_string(),
        DeviceId::from(device_id),
    );

    let device_id = 3;
    let addr_3 = libsignal_service::prelude::protocol::ProtocolAddress::new(
        "+32412345678".into(),
        DeviceId::from(device_id),
    );

    // Create two identities and two sessions
    let key_1 = libsignal_service::prelude::protocol::IdentityKeyPair::generate(&mut rng);
    let key_2 = libsignal_service::prelude::protocol::IdentityKeyPair::generate(&mut rng);
    let key_3 = libsignal_service::prelude::protocol::IdentityKeyPair::generate(&mut rng);

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

    let session_1 = libsignal_service::prelude::protocol::SessionRecord::new_fresh();
    let session_2 = libsignal_service::prelude::protocol::SessionRecord::new_fresh();
    let session_3 = libsignal_service::prelude::protocol::SessionRecord::new_fresh();
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

    let opt = match parse_args() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}.", e);
            std::process::exit(1);
        }
    };

    let path = opt.path;
    let mut store = create_storage(opt.password.as_deref(), path.into()).await;

    if opt.fill_dummy {
        add_dummy_data(&mut store).await;
    }

    Ok(())
}
