use std::path::Path;

use failure::{ensure, format_err, Error};
use futures::io::AsyncReadExt;
use harbour_whisperfish::settings::SignalConfig;
use harbour_whisperfish::store::{self, Storage};
use structopt::StructOpt;

use libsignal_service::prelude::*;
use libsignal_service_actix::prelude::*;

/// Signal attachment downloader for Whisperfish
#[derive(StructOpt, Debug)]
#[structopt(name = "fetch-signal-attachment")]
struct Opt {
    /// Whisperfish storage password
    #[structopt(short, long)]
    password: String,

    /// AttachmentPointer ID
    #[structopt(short, long)]
    id: u64,

    /// Key of AttachmentPointer
    #[structopt(short, long)]
    key: String,

    /// Message will be found by timestamp.
    #[structopt(short, long)]
    timestamp: u64,

    /// Extension for file
    #[structopt(short, long)]
    ext: String,
}

fn read_config() -> Result<SignalConfig, Error> {
    // XXX non-existing file?
    let conf_dir = dirs::config_dir().ok_or(format_err!("Could not find config directory."))?;
    let signal_config_file = conf_dir.join("harbour-whisperfish").join("config.yml");
    let signal_config_file = std::fs::File::open(signal_config_file)?;

    Ok(serde_yaml::from_reader(signal_config_file)?)
}

const SERVICE_URL: &str = "https://textsecure-service.whispersystems.org/";
const CDN_URL: &str = "https://cdn.signal.org";
const CDN2_URL: &str = "https://cdn.signal.org";
const ROOT_CA: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", "rootCA.crt"));

#[actix_rt::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    let opt = Opt::from_args();

    let config = read_config()?;
    let settings = harbour_whisperfish::settings::Settings::default();
    let dir = settings.get_string("attachment_dir");
    let dest = Path::new(&dir);

    let mut storage =
        Storage::open_with_password(&store::default_location()?, opt.password).await?;

    let key_material = hex::decode(opt.key)?;
    ensure!(
        key_material.len() == 64,
        "Attachment key should have 64 bytes"
    );

    // Check whether we can find the message that this attachment should be linked to.
    let msg = storage
        .fetch_message_by_timestamp(opt.timestamp)
        .expect("find message with ts");
    let mid = msg.id;

    // Connection details for OWS servers
    let phonenumber = phonenumber::parse(None, config.tel).unwrap();
    let e164 = phonenumber
        .format()
        .mode(phonenumber::Mode::E164)
        .to_string();
    log::info!("E164: {}", e164);
    let password = Some(storage.signal_password().await.unwrap());
    let signaling_key = storage.signaling_key().await.unwrap();
    let credentials = Credentials {
        uuid: None,
        e164: e164.clone(),
        password,
        signaling_key,
    };

    let service_cfg = ServiceConfiguration {
        service_urls: vec![SERVICE_URL.to_string()],
        cdn_urls: vec![CDN_URL.to_string(), CDN2_URL.to_string()],
        contact_discovery_url: vec![],
    };

    // Connect to OWS
    let useragent = format!("Whisperfish-{}", env!("CARGO_PKG_VERSION"));
    let mut service = AwcPushService::new(service_cfg, credentials.clone(), &useragent, &ROOT_CA);

    // Download the attachment
    let mut stream = service.get_attachment_by_id(opt.id).await?;
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

    let attachment_path =
        crate::store::save_attachment(&dest, &opt.ext, futures::io::Cursor::new(ciphertext)).await;

    log::info!("Attachment stored at {:?}", attachment_path);

    storage.register_attachment(
        mid,
        attachment_path.to_str().expect("attachment path utf-8"),
    );
    log::info!("Attachment registred with message {:?}", msg);
    Ok(())
}
