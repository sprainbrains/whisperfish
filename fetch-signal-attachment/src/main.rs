use futures::io::AsyncReadExt;
use libsignal_service::configuration::SignalServers;
use libsignal_service::prelude::*;
use libsignal_service_actix::prelude::*;
use mime_classifier::{ApacheBugFlag, LoadContext, MimeClassifier, NoSniffFlag};
use std::path::Path;
use whisperfish::store::{self, Storage};

const HELP: &str = "Signal attachment downloader for Whisperfish

USAGE:
  fetch-signal-attachment [OPTIONS] --cdn-key <cdn-key> --cdn-number <cdn-number> --ext <ext> --key <key> --message-id <message-id> --mime-type <mime-type>

FLAGS:
  -h, --help
        Prints help information

  -V, --version
        Prints version information


OPTIONS:
  --cdn-key <cdn-key>
        AttachmentPointer CdnKey or CdnId

  --cdn-number <cdn-number>
        CDN number (normally either 0 or 2)

  --ext <ext>
        Extension for file

  --key <key>
        Key of AttachmentPointer

  --message-id <message-id>
        Message will be found by ID.

        Specify either this or `timestamp`

  --mime-type <mime-type>
        Mime-type for file

  --password <password>
        Whisperfish storage password";

/// Signal attachment downloader for Whisperfish
#[derive(Debug)]
struct Opts {
    password: Option<String>,
    cdn_number: u32,
    cdn_key: String,
    key: String,
    message_id: i32,
    ext: String,
    mime_type: String,
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
        cdn_number: pargs.value_from_fn("--cdn-number", to_u32)?,
        cdn_key: pargs.value_from_str("--cdn-key")?,
        key: pargs.value_from_str("--key")?,
        message_id: pargs.value_from_fn("--message-id", to_i32)?,
        ext: pargs.value_from_str("--ext")?,
        mime_type: pargs.value_from_str("--mime-type")?,
    };

    let remaining = pargs.finish();
    if !remaining.is_empty() {
        eprintln!("Error: unused arguments: {:?}.", remaining);
        std::process::exit(1);
    }

    Ok(args)
}

fn to_u32(s: &str) -> Result<u32, &'static str> {
    s.parse().map_err(|_| "not a number")
}

fn to_i32(s: &str) -> Result<i32, &'static str> {
    s.parse().map_err(|_| "not a number")
}

#[actix_rt::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    let mut opt = match parse_args() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}.", e);
            std::process::exit(1);
        }
    };

    let config = whisperfish::config::SignalConfig::read_from_file()?;
    let settings = whisperfish::config::SettingsBridge::default();
    let dir = settings.get_string("attachment_dir");
    let dest = Path::new(&dir);

    let mut storage = Storage::open(&store::default_location()?, opt.password).await?;

    let key_material = hex::decode(opt.key)?;
    anyhow::ensure!(
        key_material.len() == 64,
        "Attachment key should have 64 bytes"
    );

    // Check whether we can find the message that this attachment should be linked to.
    let mid = opt.message_id;
    let msg = storage
        .fetch_message_by_id(mid)
        .expect("find message by mid");
    anyhow::ensure!(
        msg.id == mid,
        "unreachable: Fetched message ID does not equal supplied mid"
    );

    // Connection details for OWS servers
    // XXX: https://gitlab.com/whisperfish/whisperfish/-/issues/80
    let phonenumber = phonenumber::parse(None, config.get_tel_clone()).unwrap();
    let uuid = uuid::Uuid::parse_str(&config.get_uuid_clone()).ok();
    let device_id = config.get_device_id();
    let e164 = phonenumber
        .format()
        .mode(phonenumber::Mode::E164)
        .to_string();
    log::info!("E164: {}", e164);
    let signaling_key = Some(storage.signaling_key().await.unwrap());
    let credentials = ServiceCredentials {
        uuid,
        phonenumber,
        password: None,
        signaling_key,
        device_id: Some(device_id.into()),
    };

    // Connect to OWS
    let mut service = AwcPushService::new(
        SignalServers::Production,
        Some(credentials.clone()),
        whisperfish::user_agent(),
    );

    // Download the attachment
    let mut stream = service
        .get_attachment_by_id(&opt.cdn_key, opt.cdn_number)
        .await?;
    log::info!("Downloading attachment");

    // We need the whole file for the crypto to check out 😢
    let mut ciphertext = Vec::new();
    let len = stream
        .read_to_end(&mut ciphertext)
        .await
        .expect("streamed attachment");

    log::info!("Downloaded {} bytes", len);

    let mut key = [0u8; 64];
    key.copy_from_slice(&key_material);
    libsignal_service::attachment_cipher::decrypt_in_place(key, &mut ciphertext)
        .expect("attachment decryption");

    // Signal Desktop sometimes sends a JPEG image with .png extension,
    // so double check the received .png image, and rename it if necessary.
    opt.ext = opt.ext.to_lowercase();
    if opt.ext == "png" {
        log::trace!("Checking for JPEG with .png extension...");
        let classifier = MimeClassifier::new();
        let computed_type = classifier.classify(
            LoadContext::Image,
            NoSniffFlag::Off,
            ApacheBugFlag::Off,
            &None,
            &ciphertext as &[u8],
        );
        if computed_type == mime::IMAGE_JPEG {
            log::info!("Received JPEG file with .png suffix, fixing suffix");
            opt.ext = String::from("jpg");
        }
    }

    let attachment_path = storage
        .save_attachment(dest, &opt.ext, &ciphertext)
        .await
        .unwrap();

    log::info!("Attachment stored at {:?}", attachment_path);

    storage.register_attachment(
        mid,
        // Reconstruct attachment pointer
        AttachmentPointer {
            content_type: Some(opt.mime_type),
            ..Default::default()
        },
        attachment_path
            .canonicalize()
            .unwrap()
            .to_str()
            .expect("attachment path utf-8"),
    );
    log::info!("Attachment registered with message {:?}", msg);
    Ok(())
}
