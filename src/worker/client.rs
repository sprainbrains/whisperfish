use actix::prelude::*;
use qmetaobject::*;

use crate::gui::StorageReady;
use crate::sfos::SailfishApp;
use crate::store::Storage;

use libsignal_protocol::Context;
use libsignal_service::prelude::*;
use libsignal_service_actix::prelude::*;

const SERVICE_URL: &str = "https://textsecure-service.whispersystems.org/";
const ROOT_CA: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", "rootCA.crt"));

#[derive(QObject, Default)]
#[allow(non_snake_case)]
pub struct ClientWorker {
    base: qt_base_class!(trait QObject),
    messageReceived: qt_signal!(),
    messageReceipt: qt_signal!(),
    notifyMessage: qt_signal!(),
    promptResetPeerIdentity: qt_signal!(),

    actor: Option<Addr<ClientActor>>,
}

/// ClientActor keeps track of the connection state.
pub struct ClientActor {
    inner: QObjectBox<ClientWorker>,

    /// Some(Service) when connected, otherwise None
    service: Option<AwcPushService>,
    storage: Option<Storage>,
    cipher: Option<ServiceCipher>,
    context: Context,
}

impl ClientActor {
    pub fn new(app: &mut SailfishApp) -> Result<Self, failure::Error> {
        let inner = QObjectBox::new(ClientWorker::default());
        app.set_object_property("ClientWorker".into(), inner.pinned());

        let crypto = libsignal_protocol::crypto::DefaultCrypto::default();

        Ok(Self {
            inner,
            service: None,
            storage: None,
            cipher: None,
            context: Context::new(crypto)?,
        })
    }
}

impl Actor for ClientActor {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.inner.pinned().borrow_mut().actor = Some(ctx.address());
    }
}

impl Handler<StorageReady> for ClientActor {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(
        &mut self,
        StorageReady(storage, config): StorageReady,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.storage = Some(storage.clone());

        let useragent = format!("Whisperfish-{}", env!("CARGO_PKG_VERSION"));
        let service_cfg = ServiceConfiguration {
            service_urls: vec![SERVICE_URL.to_string()],
            cdn_urls: vec![],
            contact_discovery_url: vec![],
        };

        let context = self.context.clone();

        Box::pin(
            async move {
                // Web socket
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

                let service =
                    AwcPushService::new(service_cfg, credentials.clone(), &useragent, &ROOT_CA);
                let mut receiver = MessageReceiver::new(service.clone());

                let pipe = receiver.create_message_pipe(credentials).await.unwrap();
                let stream = pipe.stream();
                // end web socket

                // Signal service context
                let store_context = libsignal_protocol::store_context(
                    &context,
                    // Storage is a pointer-to-shared-storage
                    storage.clone(),
                    storage.clone(),
                    storage.clone(),
                    storage.clone(),
                )
                .expect("initialized storage");
                let local_addr = ServiceAddress {
                    uuid: None,
                    e164,
                    relay: None,
                };
                let cipher = ServiceCipher::from_context(context, local_addr, store_context);
                // end signal service context

                (cipher, service, stream)
            }
            .into_actor(self)
            .map(move |(cipher, service, pipe), act, ctx| {
                act.service = Some(service);
                act.cipher = Some(cipher);
                ctx.add_stream(pipe);
            }),
        )
    }
}

impl StreamHandler<Result<Envelope, ServiceError>> for ClientActor {
    fn handle(&mut self, msg: Result<Envelope, ServiceError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                // XXX: we might want to dispatch on this error.
                log::error!("MessagePipe pushed an error: {:?}", e);
                ctx.stop();
                return;
            }
        };

        // XXX: figure out edge cases in which these are *not* initialized.
        let _service = self.service.as_mut().expect("service running");
        let _storage = self.storage.as_mut().expect("storage initialized");
        let cipher = self.cipher.as_mut().expect("cipher initialized");

        log::trace!("Opening envelope");
        let content = cipher.open_envelope(msg).expect("Content");
        log::trace!("Opened envelope {:?}", content);
    }
}
