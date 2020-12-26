use std::{collections::HashSet, net::IpAddr};

use actix::prelude::*;
use futures::prelude::*;
use netlink_packet_route::{NetlinkMessage, RtnlMessage};
use rtnetlink::{
    constants::{RTMGRP_IPV4_ROUTE, RTMGRP_IPV6_ROUTE},
    new_connection,
    packet::NetlinkPayload,
    sys::SocketAddr,
    Handle, IpVersion,
};

#[derive(Debug, Clone, Message, PartialEq, Eq)]
#[rtype(result = "()")]
pub enum ConnectionChange {
    /// Issued when going from a disconnected state (no internet connection) to a connected state.
    Connected,
    /// Issued when some interface disconnected or connected, but other interfaces were still available.
    Reconnected,
    /// Issued when no active internet connections are left.
    Disconnected,
}

#[derive(Message)]
#[rtype(result = "()")]
struct Resync;

#[derive(Message)]
#[rtype(result = "()")]
struct InitializeGateways(Vec<IpAddr>);

#[derive(Message)]
#[rtype(result = "()")]
pub struct RequestNotifications(pub Recipient<ConnectionChange>);

pub struct ConnectionManager {
    listeners: Vec<Recipient<ConnectionChange>>,
    known_gateways: HashSet<IpAddr>,
    handle: Option<Handle>,
    sync_in_progress: bool,
}

impl ConnectionManager {
    fn send_all_recipients(&self, msg: ConnectionChange) {
        let listeners = self.listeners.clone();
        Arbiter::spawn(async move {
            for recipient in listeners {
                if recipient.send(msg.clone()).await.is_err() {
                    log::warn!("Defunct ConnectionChange recipient");
                }
            }
        })
    }

    fn add_gateway(&mut self, gateway: std::net::IpAddr, ctx: &mut Context<Self>) {
        // If gateway was not yet present ...
        if self.known_gateways.insert(gateway) {
            // ... notify the listeners
            if self.known_gateways.len() > 1 {
                self.send_all_recipients(ConnectionChange::Reconnected);
            } else {
                self.send_all_recipients(ConnectionChange::Connected);
            }
        } else {
            log::warn!("A known gateway was readded");
            ctx.notify(Resync);
        }
    }

    fn remove_gateway(&mut self, gateway: std::net::IpAddr, ctx: &mut Context<Self>) {
        if !self.known_gateways.remove(&gateway) {
            log::warn!("An unknown gateway was deleted.");
            ctx.notify(Resync);
        }

        // If we still have connections
        if self.known_gateways.len() > 0 {
            self.send_all_recipients(ConnectionChange::Reconnected);
        } else {
            self.send_all_recipients(ConnectionChange::Disconnected);
        }
    }
}

impl Actor for ConnectionManager {
    type Context = Context<Self>;
}

impl Default for ConnectionManager {
    fn default() -> Self {
        ConnectionManager {
            listeners: vec![],
            known_gateways: HashSet::new(),
            handle: None,
            sync_in_progress: false,
        }
    }
}

impl Supervised for ConnectionManager {}
impl ArbiterService for ConnectionManager {
    fn service_started(&mut self, ctx: &mut Context<Self>) {
        log::info!("ConnectionManager started");
        // Open the netlink socket
        // XXX: do we need to capture the handle and remove it on shutdown?
        let (mut connection, handle, messages) =
            new_connection().map_err(|e| format!("{}", e)).unwrap();

        // These flags specify what kinds of broadcast messages we want to listen for.
        let mgroup_flags = RTMGRP_IPV4_ROUTE | RTMGRP_IPV6_ROUTE;

        // A netlink socket address is created with said flags.
        let addr = SocketAddr::new(0, mgroup_flags);
        // Said address is bound so new conenctions and thus new message broadcasts can be received.
        connection.socket_mut().bind(&addr).expect("failed to bind");

        self.handle = Some(handle);

        Arbiter::spawn(connection);

        ctx.notify(Resync);
        ctx.add_stream(messages);
    }
}

impl Handler<InitializeGateways> for ConnectionManager {
    type Result = ();

    fn handle(
        &mut self,
        InitializeGateways(gws): InitializeGateways,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        log::trace!("(Re)initialized known gateways");
        self.sync_in_progress = false;
        self.known_gateways.clear();
        self.known_gateways.extend(gws);

        if self.known_gateways.len() > 0 {
            self.send_all_recipients(ConnectionChange::Connected);
        }
    }
}

impl Handler<Resync> for ConnectionManager {
    type Result = ();

    fn handle(&mut self, _: Resync, ctx: &mut Self::Context) -> Self::Result {
        if self.sync_in_progress {
            return;
        }

        let handle = self.handle.clone().unwrap();
        let mgr = ctx.address();
        self.sync_in_progress = true;

        // Initialize the manager and run the connection
        Arbiter::spawn(async move {
            let mut routes = vec![];
            let mut v4 = handle.route().get(IpVersion::V4).execute();
            while let Some(route) = v4.try_next().await.unwrap() {
                if let Some(gw) = route.gateway() {
                    routes.push(gw);
                }
            }
            let mut v6 = handle.route().get(IpVersion::V6).execute();
            while let Some(route) = v6.try_next().await.unwrap() {
                if let Some(gw) = route.gateway() {
                    routes.push(gw);
                }
            }

            mgr.send(InitializeGateways(routes)).await.unwrap();
        });
    }
}

impl Handler<RequestNotifications> for ConnectionManager {
    type Result = ();

    fn handle(
        &mut self,
        RequestNotifications(recipient): RequestNotifications,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.listeners.push(recipient);
    }
}

impl StreamHandler<(NetlinkMessage<RtnlMessage>, SocketAddr)> for ConnectionManager {
    fn handle(
        &mut self,
        (msg, _sock): (NetlinkMessage<RtnlMessage>, SocketAddr),
        ctx: &mut Self::Context,
    ) {
        log::trace!("NL: {:?}", msg);
        match msg.payload {
            // Only use route messages
            NetlinkPayload::InnerMessage(i) => match i {
                RtnlMessage::NewRoute(route) => {
                    if let Some(g) = route.gateway() {
                        self.add_gateway(g, ctx);
                    }
                }
                RtnlMessage::DelRoute(route) => {
                    if let Some(g) = route.gateway() {
                        self.remove_gateway(g, ctx);
                    }
                }
                _ => {
                    // nop
                }
            },
            _ => return,
        }
    }
}
