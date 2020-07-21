use actix::prelude::*;
use qmetaobject::*;

use crate::gui::StorageReady;
use crate::sfos::SailfishApp;
use crate::store::Storage;

use libsignal_protocol::Context;
use libsignal_service::content::ContentBody;
use libsignal_service::prelude::*;
use libsignal_service_actix::prelude::*;

const SERVICE_URL: &str = "https://textsecure-service.whispersystems.org/";
const ROOT_CA: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", "rootCA.crt"));

#[derive(QObject, Default)]
#[allow(non_snake_case)]
pub struct ClientWorker {
    base: qt_base_class!(trait QObject),
    messageReceived: qt_signal!(sid: i64, mid: i32),
    messageReceipt: qt_signal!(),
    notifyMessage: qt_signal!(sid: i64, source: QString, message: QString, is_group: bool),
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
    fn handle(&mut self, msg: Result<Envelope, ServiceError>, _ctx: &mut Self::Context) {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                // XXX: we might want to dispatch on this error.
                log::error!("MessagePipe pushed an error: {:?}", e);
                return;
            }
        };

        // XXX: figure out edge cases in which these are *not* initialized.
        let _service = self.service.as_mut().expect("service running");
        let storage = self.storage.as_mut().expect("storage initialized");
        let cipher = self.cipher.as_mut().expect("cipher initialized");

        let Content { body, metadata } = match cipher.open_envelope(msg) {
            Ok(Some(content)) => content,
            Ok(None) => {
                log::info!("Empty envelope");
                return;
            }
            Err(e) => {
                log::error!("Error opening envelope: {:?}", e);
                return;
            }
        };

        log::trace!(
            "Opened envelope Content {{ body: {:?}, metadata: {:?} }}",
            body,
            metadata
        );

        let notify = match body {
            ContentBody::DataMessage(message) => {
                let msg = crate::store::NewMessage {
                    session_id: None,
                    source: metadata.sender.e164.clone(),
                    text: message.body().to_string(),
                    timestamp: metadata.timestamp as i64,
                    sent: false,
                    received: false,
                    flags: message.flags() as i32,
                    attachment: None, // FIXME
                    mime_type: None,  // FIXME
                    has_attachment: false,
                    outgoing: false,
                };
                Some(storage.process_message(msg, &None, true))
            }
            ContentBody::SynchronizeMessage(message) => {
                if let Some(sent) = message.sent {
                    log::trace!("Sync sent message");
                    // These are messages sent through a paired device.

                    let message = sent.message.expect("sync sent with message");
                    let msg = crate::store::NewMessage {
                        session_id: None,
                        source: metadata.sender.e164.clone(),
                        text: message.body().to_string(),
                        timestamp: metadata.timestamp as i64,
                        sent: true,
                        received: false,
                        flags: message.flags() as i32,
                        attachment: None, // FIXME
                        mime_type: None,  // FIXME
                        has_attachment: false,
                        outgoing: true,
                    };
                    Some(storage.process_message(msg, &None, false))
                } else if let Some(_request) = message.request {
                    log::trace!("Sync request message");
                    None
                } else if message.read.len() > 0 {
                    log::trace!("Sync read message");
                    None
                } else {
                    log::warn!("Sync message without known sync type");
                    None
                }
            }
            ContentBody::TypingMessage(_typing) => {
                log::info!("{} is typing.", metadata.sender.e164);
                None
            }
            ContentBody::ReceiptMessage(_receipt) => {
                log::info!("{} received a message.", metadata.sender.e164);
                None
            }
            ContentBody::CallMessage(_call) => {
                log::info!("{} is calling.", metadata.sender.e164);
                None
            }
        };

        if let Some((message, session)) = notify {
            self.inner
                .pinned()
                .borrow_mut()
                .messageReceived(session.id, message.id);
            self.inner.pinned().borrow_mut().notifyMessage(
                session.id,
                session
                    .group_name
                    .as_deref()
                    .unwrap_or(&session.source)
                    .into(),
                message.message.into(),
                session.group_id.is_some(),
            );
        }
    }
}
