use actix::prelude::*;

use crate::store::observer::Event;
use crate::store::observer::EventObserving;
use crate::store::observer::Interest;
use crate::store::Storage;

#[macro_export]
macro_rules! observing_model {
    ($vis:vis struct $model:ident($encapsulated:ty) {
        $($property:ident: $t:ty; READ $getter:ident $(WRITE $setter:ident)?),* $(,)?
    } $(
        WITH OPTIONAL PROPERTIES FROM $field:ident WITH ROLE $role:ident {
            $($opt_property:ident $role_variant:ident),* $(,)?
        }
    )?) => {
        #[derive(QObject)]
        $vis struct $model {
            base: qt_base_class!(trait QObject),
            inner: std::sync::Arc<std::cell::RefCell<$encapsulated>>,
            actor: Option<actix::Addr<ObservingModelActor<$encapsulated>>>,

            app: qt_property!(QPointer<$crate::gui::AppState>; WRITE set_app),

            $(
                #[allow(unused)]
                $property: qt_property!($t; READ $getter $(WRITE $setter)? NOTIFY something_changed),
            )*

            $(
            $(
                #[allow(unused)]
                $opt_property: qt_property!(QVariant; READ $opt_property NOTIFY something_changed),
            )*
            )?

            something_changed: qt_signal!(),
        }

        impl Default for $model {
            fn default() -> Self {
                let inner = std::sync::Arc::<std::cell::RefCell::<$encapsulated>>::default();

                Self {
                    base: Default::default(),
                    app: Default::default(),
                    inner,
                    actor: None,
                    $( $property: Default::default(), )*
                    $($( $opt_property: Default::default(), )*)?
                    something_changed: Default::default(),
                }
            }
        }

        impl $model {
            #[qmeta_async::with_executor]
            fn set_app(&mut self, app: QPointer<$crate::gui::AppState>) {
                self.app = app;
                self.reinit();
            }

            fn reinit(&mut self) {
                use actix::prelude::*;
                if let Some(app) = self.app.as_pinned() {
                    if let Some(mut storage) = app.borrow().storage.borrow().clone() {
                        let actor = ObservingModelActor {
                            model: std::sync::Arc::downgrade(&self.inner),
                            storage: storage.clone(),
                        }
                        .start();

                        let subscriber = actor.downgrade().recipient();
                        self.actor = Some(actor);
                        storage.register_observer($crate::store::observer::EventObserving::interests(&*self.inner.borrow()), subscriber);

                        (&self.inner as &std::cell::RefCell<$encapsulated>).borrow_mut().init(storage);
                        self.something_changed();
                    }
                }
            }

            $(
            $(
                fn $opt_property(&self) -> qmetaobject::QVariant {
                    match self.inner.borrow().$field.as_ref() {
                        Some(x) => {
                            ($role::$role_variant).get(x)
                        }
                        None => qmetaobject::QVariant::default()
                    }
                }
            )*
            )?
            $(
                fn $getter(&self) -> $t {
                    self.inner.borrow().$getter()
                }

                $(
                fn $setter(&mut self, v: $t) {
                    (&self.inner as &std::cell::RefCell<$encapsulated>).borrow_mut().$setter(
                        self.app.as_pinned().and_then(|app| app.borrow().storage.borrow().clone()),
                        v,
                    );
                    self.something_changed();
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
pub struct ObservingModelActor<T> {
    pub(super) model: std::sync::Weak<std::cell::RefCell<T>>,
    pub(super) storage: Storage,
}

impl<T: 'static> actix::Actor for ObservingModelActor<T> {
    type Context = actix::Context<Self>;
}

impl<T: 'static> actix::Handler<Event> for ObservingModelActor<T>
where
    T: EventObserving,
{
    type Result = Vec<Interest>;

    fn handle(&mut self, event: Event, ctx: &mut Self::Context) -> Self::Result {
        match self.model.upgrade() {
            Some(model) => {
                let mut model = model.borrow_mut();
                model.observe(self.storage.clone(), event);
                model.interests()
            }
            None => {
                // In principle, the actor should have gotten stopped when the model got dropped,
                // because the actor's only strong reference is contained in the ObservingModel.
                log::debug!("Model got dropped, stopping actor execution.");
                // XXX What is the difference between stop and terminate?
                ctx.stop();
                Vec::new()
            }
        }
    }
}
