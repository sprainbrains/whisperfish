macro_rules! define_model_roles {
    (RETRIEVE $obj:ident fn $fn:ident(&self) $(via $via_fn:path)*) => {{
        let field = $obj.$fn();
        $(let field = $via_fn(field);)*
        field.into()
    }};
    (RETRIEVE $obj:ident $($field:ident).+ $(via $via_fn:path)*) => {{
        let field = $obj.$($field).+.clone();
        $(let field = $via_fn(field);)*
        field.into()
    }};
    (enum $enum_name:ident for $diesel_model:ty {
     $($role:ident($($retrieval:tt)*): $name:expr),* $(,)?
    }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum $enum_name {
            $($role),*
        }

        impl $enum_name {
            #[allow(unused_assignments)]
            #[allow(dead_code)]
            fn role_names() -> HashMap<i32, qmetaobject::QByteArray> {
                let mut hm = HashMap::new();

                let mut i = 0;
                $(
                    hm.insert(i, $name.into());
                    i += 1;
                )*

                hm
            }

            fn get(&self, obj: &$diesel_model) -> qmetaobject::QVariant {
                match self {
                    $(
                        Self::$role => define_model_roles!(RETRIEVE obj $($retrieval)*),
                    )*
                }
            }

            fn from(i: i32) -> Self {
                let rm = [$(Self::$role, )*];
                rm[i as usize]
            }
        }
    };
}

pub mod attachment;
pub mod contact;
pub mod device;
pub mod messages;
pub mod sessions;

pub mod prompt;

use std::sync::Arc;
use std::sync::Weak;

use crate::store::observer::Event;
use crate::store::observer::EventObserving;
use crate::store::Storage;

pub use self::contact::*;
pub use self::device::*;
pub use self::messages::*;
pub use self::prompt::*;
pub use self::sessions::*;

use actix::prelude::*;
use chrono::prelude::*;
use qmetaobject::prelude::*;

fn qdate_from_chrono<T: TimeZone>(dt: DateTime<T>) -> QDate {
    let dt = dt.with_timezone(&Local).naive_local();
    QDate::from_y_m_d(dt.year(), dt.month() as i32, dt.day() as i32)
}

fn qdatetime_from_chrono<T: TimeZone>(dt: DateTime<T>) -> QDateTime {
    let dt = dt.with_timezone(&Local).naive_local();
    let date = QDate::from_y_m_d(dt.year(), dt.month() as i32, dt.day() as i32);
    let time = QTime::from_h_m_s_ms(
        dt.hour() as i32,
        dt.minute() as i32,
        Some(dt.second() as i32),
        None,
    );

    QDateTime::from_date_time_local_timezone(date, time)
}

fn qdatetime_from_naive_option(timestamp: Option<NaiveDateTime>) -> qmetaobject::QVariant {
    timestamp
        .map(qdatetime_from_naive)
        .map(QVariant::from)
        .unwrap_or_default()
}

fn qdatetime_from_naive(timestamp: NaiveDateTime) -> QDateTime {
    // Naive in model is Utc, naive displayed should be Local
    qdatetime_from_chrono(DateTime::<Utc>::from_utc(timestamp, Utc))
}

fn qstring_from_option(opt: Option<impl AsRef<str>>) -> QVariant {
    match opt {
        Some(s) => QString::from(s.as_ref()).into(),
        None => QVariant::default(),
    }
}

#[macro_export]
macro_rules! observing_model {
    ($vis:vis struct $model:ident($encapsulated:ty) {
        $($field:ident: $t:ty; READ $getter:ident $(WRITE $setter:ident)?),* $(,)?
    }) => {
        #[derive(QObject)]
        $vis struct $model {
            base: qt_base_class!(trait QObject),
            inner: Arc<std::cell::RefCell<$encapsulated>>,
            actor: Option<Addr<ObservingModelActor<$encapsulated>>>,

            app: qt_property!(QPointer<AppState>; WRITE set_app),

            $(
                #[allow(unused)]
                $field: qt_property!($t; READ $getter $(WRITE $setter)?),
            )*
        }

        impl Default for $model {
            fn default() -> Self {
                let inner = Arc::<std::cell::RefCell::<$encapsulated>>::default();

                Self {
                    base: Default::default(),
                    app: Default::default(),
                    inner,
                    actor: None,
                    $( $field: Default::default(), )*
                }
            }
        }

        impl $model {
            #[with_executor]
            fn set_app(&mut self, app: QPointer<AppState>) {
                self.app = app;
                self.reinit();
            }

            fn reinit(&mut self) {
                if let Some(app) = self.app.as_pinned() {
                    if let Some(mut storage) = app.borrow().storage.borrow().clone() {
                        let actor = ObservingModelActor {
                            model: Arc::downgrade(&self.inner),
                            storage: storage.clone(),
                        }
                        .start();

                        let subscriber = actor.downgrade().recipient();
                        self.actor = Some(actor);
                        storage.register_observer(<$encapsulated>::interests(), subscriber);

                        (&self.inner as &std::cell::RefCell<$encapsulated>).borrow_mut().init(storage);
                    }
                }
            }

            $(
                fn $getter(&self) -> $t {
                    self.inner.borrow().$getter()
                }

                $(
                fn $setter(&mut self, v: $t) {
                    (&self.inner as &std::cell::RefCell<$encapsulated>).borrow_mut().$setter(
                        self.app.as_pinned().and_then(|app| app.borrow().storage.borrow().clone()),
                        v,
                    )
                }
                )?
            )*
        }
    };
}

/// An actor that accompanies the [ObservingModel], responsible to dispatch events to the contained
/// model.
///
/// The contained model is a weak pointer, such that the actor will stop when the model goes out of
/// scope.
struct ObservingModelActor<T> {
    model: Weak<std::cell::RefCell<T>>,
    storage: Storage,
}

impl<T: QObject + 'static> Actor for ObservingModelActor<T> {
    type Context = Context<Self>;
}

impl<T: QObject + 'static> Handler<Event> for ObservingModelActor<T>
where
    T: EventObserving,
{
    type Result = ();

    fn handle(&mut self, event: Event, ctx: &mut Self::Context) -> Self::Result {
        match self.model.upgrade() {
            Some(model) => model.borrow_mut().observe(self.storage.clone(), event),
            None => {
                // In principle, the actor should have gotten stopped when the model got dropped,
                // because the actor's only strong reference is contained in the ObservingModel.
                log::debug!("Model got dropped, stopping actor execution.");
                // XXX What is the difference between stop and terminate?
                ctx.stop();
            }
        }
    }
}
