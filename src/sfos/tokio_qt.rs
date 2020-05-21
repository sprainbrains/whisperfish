use std::os::unix::io::RawFd;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};
use std::sync::Mutex;
use std::cell::RefCell;

use futures::prelude::*;
use pin_utils::unsafe_unpinned;
use tokio::io::Registration;

cpp_class!(
    pub unsafe struct QSocketNotifier as "QSocketNotifier"
);

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
enum QSocketNotifierType {
    Read = 0,
    Write = 1,
    Exception = 2,
}

impl Into<mio::Ready> for QSocketNotifierType {
    fn into(self) -> mio::Ready {
        use QSocketNotifierType::*;
        match self {
            Read => mio::Ready::readable(),
            Write => mio::Ready::writable(),
            Exception => *mio::unix::UnixReady::error(),
        }
    }
}

impl QSocketNotifier {
    fn socket(&self) -> RawFd {
        unsafe {
            cpp!( [self as "QSocketNotifier *"] -> RawFd as "int" {
                return self->socket();
            })
        }
    }

    fn notifier_type(&self) -> QSocketNotifierType {
        unsafe {
            cpp!( [self as "QSocketNotifier *"] -> QSocketNotifierType as "int" {
                return self->type();
            })
        }
    }
}

#[derive(Debug, Clone)]
struct TimerSpec {
    timer_id: i32,
    interval: u32,
    obj: *mut std::os::raw::c_void,
}

impl Eq for TimerSpec {}
impl PartialEq for TimerSpec {
    fn eq(&self, rhs: &Self) -> bool {
        self.timer_id == rhs.timer_id
    }
}

impl Ord for TimerSpec {
    fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
        self.timer_id.cmp(&rhs.timer_id)
    }
}
impl PartialOrd for TimerSpec {
    fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
        self.timer_id.partial_cmp(&rhs.timer_id)
    }
}

struct Timer {
    spec: TimerSpec,
    interval: tokio::time::Interval,
}

impl Eq for Timer {}
impl PartialEq for Timer {
    fn eq(&self, rhs: &Self) -> bool {
        self.spec == rhs.spec
    }
}

impl Ord for Timer {
    fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
        self.spec.cmp(&rhs.spec)
    }
}
impl PartialOrd for Timer {
    fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
        self.spec.partial_cmp(&rhs.spec)
    }
}

impl Stream for Timer {
    type Item = TimerSpec;

    fn poll_next(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<TimerSpec>> {
        match Stream::poll_next(Pin::new(&mut self.interval), ctx) {
            Poll::Ready(Some(_instant)) => Poll::Ready(Some(self.spec.clone())),
            Poll::Ready(None) => {
                log::warn!("Unexpected end of Interval stream. Please file a bug report.");
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl From<TimerSpec> for Timer {
    fn from(spec: TimerSpec) -> Timer {
        let duration = std::time::Duration::from_millis(spec.interval as u64);
        let start = std::time::Instant::now() + duration;
        Timer {
            spec,
            interval: tokio::time::interval_at(start.into(), duration),
        }
    }
}

pub struct TokioQEventDispatcherPriv {
    socket_registrations: Vec<(*mut QSocketNotifier, Registration)>,
    timers: Vec<Timer>,
    waker: Mutex<RefCell<Option<Waker>>>,
    _unpin: std::marker::PhantomPinned,
}

impl Default for TokioQEventDispatcherPriv {
    fn default() -> Self {
        TokioQEventDispatcherPriv {
            socket_registrations: Default::default(),
            timers: Default::default(),
            waker: Default::default(),
            _unpin: std::marker::PhantomPinned,
        }
    }
}

impl TokioQEventDispatcherPriv {
    unsafe_unpinned!(timers: Vec<Timer>);
    unsafe_unpinned!(socket_registrations: Vec<(*mut QSocketNotifier, Registration)>);

    fn register_socket_notifier(self: Pin<&mut Self>, raw_notifier: *mut QSocketNotifier) {
        let notifier = unsafe { raw_notifier.as_mut() }.unwrap();
        let fd = notifier.socket();
        log::debug!("registerSocketNotifier: fd={} for {:?} on thread {:?}", fd, notifier.notifier_type(), std::thread::current().id());

        let reg = Registration::new_with_ready(
            &mio::unix::EventedFd(&fd),
            notifier.notifier_type().into(),
        ).unwrap(); // XXX unwrap

        self.socket_registrations().push((
            raw_notifier,
            reg
        ));
    }

    fn register_timer(
        mut self: Pin<&mut Self>,
        timer_id: i32,
        interval: u32,
        obj: *mut std::os::raw::c_void,
    ) {
        let timers = self.as_mut().timers();
        for timer in timers.iter() {
            if timer.spec.timer_id == timer_id {
                log::warn!("Registering duplicate timer");
                return;
            }
        }

        timers.push(
            TimerSpec {
                timer_id,
                interval,
                obj,
            }
            .into(),
        );

        self.wake_up();
    }

    fn unregister_timer(mut self: Pin<&mut Self>, timer_id: i32) -> bool {
        let timers = self.as_mut().timers();
        let (idx, timer) = match timers
            .iter_mut()
            .enumerate()
            .find(|(_id, t)| t.spec.timer_id == timer_id)
        {
            Some(v) => v,
            None => return false,
        };

        timers.remove(idx);

        for (i, timer) in timers.iter().enumerate() {
            log::trace!("- {}: {}ms", i, timer.spec.interval);
        }

        self.wake_up();

        true
    }

    fn wake_up(self: Pin<&mut Self>) {
        if let Some(waker) = self.waker.lock().unwrap().borrow().clone() {
            // XXX: Consider waking by value,
            //      maybe store wakers in thread-local storage or something.
            waker.wake();
        } else {
            log::trace!("Already awaken");
        }
    }

    fn set_waker(self: Pin<&mut Self>, w: &Waker) {
        drop(self.waker.lock().unwrap().replace(Some(w.clone())));
    }

    fn poll_sockets(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<()> {
        let mut events = Vec::new();

        for (notifier, registration) in &self.socket_registrations {
            let notifier: *mut QSocketNotifier = *notifier;

            let read = match registration.poll_read_ready(ctx) {
                Poll::Ready(Ok(_readiness)) => {
                    true
                }
                Poll::Ready(Err(e)) => {
                    log::error!(
                        "Something wrong with registration {:?}: {:?}",
                        registration,
                        e
                    );
                    false
                }
                Poll::Pending => false,
            };

            let write = match registration.poll_write_ready(ctx) {
                Poll::Ready(Ok(_readiness)) => {
                    true
                }
                Poll::Ready(Err(e)) => {
                    log::error!(
                        "Something wrong with registration {:?}: {:?}",
                        registration,
                        e
                    );
                    false
                }
                Poll::Pending => false,
            };

            if read || write {
                events.push(notifier);
            }
        }

        if events.len() > 0 {
            self.as_mut().wake_up();
        }

        // Drop the &mut reference to self,
        // since Qt may obtain one from now on.
        // Not sure how this *should* be handled, however.
        drop(self);

        for notifier in events {
            let result = unsafe {
                cpp!([notifier as "QSocketNotifier *"] -> bool as "bool" {
                    QEvent ev(QEvent::SockAct);
                    return QCoreApplication::sendEvent(notifier, &ev);
                })
            };
            if ! result {
                log::warn!("Socket ready, sendEvent returned false.");
            }
        }

        Poll::Pending
    }

    fn poll_timers(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<()> {
        let events = {
            let mut events = Vec::new();

            let timers = self.as_mut().timers();
            let mut stream = stream::select_all(timers.iter_mut());

            while let Poll::Ready(Some(spec)) = Stream::poll_next(Pin::new(&mut stream), ctx) {
                events.push((spec.obj, spec.timer_id))
            }
            events
        };

        drop(self);

        for (obj, id) in events {
            let ev = unsafe {
                cpp! ( [obj as "QObject *", id as "int"] -> bool as "bool" {
                    QTimerEvent e(id);
                    return QCoreApplication::sendEvent(obj, &e);
                })
            };

            if !ev {
                log::error!("Sending timer event for {} responded to with false", id);
            }
        }

        Poll::Pending
    }
}

cpp! {{
    #include <QtCore/QAbstractEventDispatcher>

    class TokioQEventDispatcher : public QAbstractEventDispatcher {
    public:
        // m_priv is a *mut Pin<Box<TokioQEventDispatcherPriv>>
        // Yes this is unhealthy.
        void *m_priv;

        TokioQEventDispatcher(void *d) : m_priv(d) { }

        bool processEvents(QEventLoop::ProcessEventsFlags flags) override {
            emit awake();
            QCoreApplication::sendPostedEvents();
            return true;
        }

        bool hasPendingEvents(void) override {
            rust!(hasPendingEvents_r [] {
                log::warn!("hasPendingEvents called, untested");
            });

            extern uint qGlobalPostedEventsCount();
            return qGlobalPostedEventsCount() > 0;
        }

        void registerSocketNotifier(QSocketNotifier* notifier) override {
            rust!(registerSocketNotifier_r [m_priv: &mut Pin<Box<TokioQEventDispatcherPriv>> as "void *", notifier: *mut QSocketNotifier as "QSocketNotifier *"] {
                m_priv.as_mut().register_socket_notifier(notifier);
            });
            wakeUp();
        }

        void unregisterSocketNotifier(QSocketNotifier* notifier) override {
            int fd = notifier->socket();
            rust!(unregisterSocketNotifier_r [m_priv: &mut Pin<Box<TokioQEventDispatcherPriv>> as "void *", notifier: *mut QSocketNotifier as "QSocketNotifier *", fd: isize as "int"] {
                log::error!("unregisterSocketNotifier: fd={}", fd);
            });
        }

        void registerTimer(
            int timerId,
            int interval,
            Qt::TimerType timerType,
            QObject* object
        ) override {
            // XXX: respect TimerType
            rust!(registerTimer_r [m_priv: &mut Pin<Box<TokioQEventDispatcherPriv>> as "void *", timerId: std::os::raw::c_int as "int", interval: std::os::raw::c_int as "int", object: *mut std::os::raw::c_void as "QObject *"] {
                m_priv.as_mut().register_timer(timerId, interval as u32, object);
            });
        }

        bool unregisterTimer(int timerId) override {
            return rust!(unregisterTimer_r [m_priv: &mut Pin<Box<TokioQEventDispatcherPriv>> as "void *", timerId: std::os::raw::c_int as "int"] -> bool as "bool" {
                m_priv.as_mut().unregister_timer(timerId as i32)
            });
        }

        bool unregisterTimers(QObject* object) override {
            rust!(unregisterTimers_r [] {
                log::error!("unregisterTimers");
            });
            return false;
        }

        QList<QAbstractEventDispatcher::TimerInfo> registeredTimers(QObject *obj) const override {
            QList<QPair<int, int>> list;
            size_t amount = rust!(countTimers [m_priv: &mut Pin<Box<TokioQEventDispatcherPriv>> as "void *"] -> usize as "size_t"{
                m_priv.timers.len()
            });

            for (size_t i = 0; i < amount; ++i) {
                int id = rust!(registeredTimers_id_r [m_priv: &mut Pin<Box<TokioQEventDispatcherPriv>> as "void *", obj: *mut std::os::raw::c_void as "QObject *", i: usize as "size_t"] -> std::os::raw::c_int as "int" {
                    let entry = &m_priv.timers[i].spec;
                    if entry.obj == obj {
                        entry.timer_id
                    } else {
                        -1
                    }
                });
                int interval = rust!(registeredTimers_interval_r [m_priv: &mut Pin<Box<TokioQEventDispatcherPriv>> as "void *", i: usize as "size_t"] -> std::os::raw::c_int as "int" {
                    m_priv.timers[i].spec.interval as i32
                });
                if (id >= 0) {
                    list << QPair<int, int>(id, interval);
                }
            }

            return QList<QAbstractEventDispatcher::TimerInfo>();
        }

        int remainingTime(int) override {
            rust!(remainingTime_r [] {
                unimplemented!("remainingTime");
            });
            return -1;
        }

        void wakeUp() override {
            rust!(wakeUp_r [m_priv: &mut Pin<Box<TokioQEventDispatcherPriv>> as "void *"] {
                m_priv.as_mut().wake_up();
            });
        }
        void interrupt() override {
            rust!(interrupt_r [] {
                unimplemented!("interrupt");
            });

            wakeUp();
        }
        void flush() override {
            rust!(flush_r [] {
                unimplemented!("flush");
            });
        }
    };
}}

cpp_class! (
    pub unsafe struct TokioQEventDispatcher as "TokioQEventDispatcher"
);

impl TokioQEventDispatcher {
    pub fn poll(&mut self, ctx: &mut Context<'_>) -> Poll<()> {
        unsafe {
            cpp!([self as "TokioQEventDispatcher*"] {
                self->processEvents(QEventLoop::AllEvents);
            })
        }

        self.m_priv_mut().set_waker(ctx.waker());

        if let Poll::Ready(_) = self.m_priv_mut().poll_sockets(ctx) {
            return Poll::Ready(());
        }

        if let Poll::Ready(_) = self.m_priv_mut().poll_timers(ctx) {
            return Poll::Ready(());
        }

        let interrupted = false; // XXX
        if interrupted {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }

    pub fn m_priv_mut(&mut self) -> Pin<&mut TokioQEventDispatcherPriv> {
        unsafe {cpp!([self as "TokioQEventDispatcher *"] -> *mut Pin<Box<TokioQEventDispatcherPriv>> as "void *" {
            return self->m_priv;
        }).as_mut()}.unwrap().as_mut()
    }

    pub fn install() {
        let p = Box::new(TokioQEventDispatcherPriv::default());
        let p: &mut Pin<Box<TokioQEventDispatcherPriv>> =
            Box::leak(Box::new(unsafe { Pin::new_unchecked(p) }));
        unsafe {
            cpp!([p as "void *"] {
                TokioQEventDispatcher *dispatch = new TokioQEventDispatcher(p);
                QCoreApplication::setEventDispatcher(dispatch);
            })
        }
    }
}
