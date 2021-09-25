use std::os::raw::*;

use qmetaobject::qttypes::*;
use qmetaobject::{QObject, QObjectPinned};

/// Qt is not thread safe, and the engine can only be created once and in one thread.
/// So this is a guard that will be used to panic if the engine is created twice
static HAS_ENGINE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

cpp! {{
    #include <memory>
    #include <QtQuick/QtQuick>
    #include <QtCore/QDebug>
    #include <QtWidgets/QApplication>
    #include <QtQml/QQmlComponent>
    #include <QtQuick/QQuickWindow>

    #include <sailfishapp.h>

    struct QmlSingleApplicationGuard {
        QmlSingleApplicationGuard() {
            rust!(Rust_QmlApplicationHolder_ctor[] {
                HAS_ENGINE.compare_exchange(false, true, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst)
                        .expect("There can only be one QmlEngine in the process");
            });
        }
        ~QmlSingleApplicationGuard() {
            rust!(Rust_QmlApplicationHolder_dtor[] {
                HAS_ENGINE.compare_exchange(true, false, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst)
                    .unwrap();
            });
        }
    };

    struct QmlApplicationHolder : QmlSingleApplicationGuard {
        std::unique_ptr<QGuiApplication> app;
        std::unique_ptr<QQuickView> view;

        QmlApplicationHolder(int &argc, char **argv)
            : app(SailfishApp::application(argc, argv))
            , view(SailfishApp::createView()) {
                QObject::connect(
                    view->engine(),
                    &QQmlEngine::quit,
                    view.get(),
                    &QWindow::close,
                    Qt::QueuedConnection
                );
            }
    };
}}

cpp_class! (
    pub unsafe struct QmlApp as "QmlApplicationHolder"
);

impl QmlApp {
    pub fn application(name: String) -> QmlApp {
        use std::ffi::CString;

        let mut arguments: Vec<*mut c_char> = std::iter::once(name)
            .chain(std::env::args())
            .map(|arg| CString::new(arg.into_bytes()).expect("argument contains invalid c-string!"))
            .map(|arg| arg.into_raw())
            .collect();
        let argc: i32 = arguments.len() as i32 - 1;
        let argv: *mut *mut c_char = arguments.as_mut_ptr();

        let result = unsafe {
            cpp! { {
                #include <QtCore/QCoreApplication>
                #include <QtCore/QString>

                #include <QtGui/QGuiApplication>
                #include <QtQuick/QQuickView>
                #include <QtQml/QtQml>
                #include <QtCore/QtCore>

                #include <sailfishapp.h>
            }}
            cpp!([argc as "int", argv as "char**"] -> QmlApp as "QmlApplicationHolder" {
                static int _argc  = argc;
                static char **_argv = nullptr;
                if (_argv == nullptr) {
                    // copy the arguments
                    _argv = new char*[argc + 1];
                    // argv should be null terminated
                    _argv[argc] = nullptr;
                    for (int i=0; i<argc; ++i) {
                        _argv[i] = new char[strlen(argv[i]) + 1];
                        strcpy(_argv[i], argv[i]);
                    }
                }
                return QmlApplicationHolder(_argc, _argv);
            })
        };

        for arg in arguments {
            unsafe {
                CString::from_raw(arg);
            }
        }

        result
    }

    // TODO: these methods come directly from `qmetaobject::QmlEngine`.  Some form of attribution
    // is necessary, and some form casting into QmlEngine.  impl Deref<Target=QmlEngine> would be
    // ideal.
    /// Sets a property for this QML context (calls QQmlEngine::rootContext()->setContextProperty)
    pub fn set_property(&mut self, name: QString, value: QVariant) {
        unsafe {
            cpp!([self as "QmlApplicationHolder*", name as "QString", value as "QVariant"] {
                self->view->engine()->rootContext()->setContextProperty(name, value);
            })
        }
    }

    /// Sets a property for this QML context (calls QQmlEngine::rootContext()->setContextProperty)
    ///
    // (TODO: consider making the lifetime the one of the engine, instead of static)
    pub fn set_object_property<T: QObject + Sized>(
        &mut self,
        name: QString,
        obj: QObjectPinned<'_, T>,
    ) {
        let obj_ptr = obj.get_or_create_cpp_object();
        cpp!(unsafe [self as "QmlApplicationHolder*", name as "QString", obj_ptr as "QObject*"] {
            self->view->engine()->rootContext()->setContextProperty(name, obj_ptr);
        })
    }

    pub fn path_to(path: QString) -> QUrl {
        unsafe {
            cpp!([path as "QString"] -> QUrl as "QUrl" {
                return SailfishApp::pathTo(path);
            })
        }
    }

    #[allow(dead_code)]
    pub fn exec(&self) {
        unsafe {
            cpp!([self as "QmlApplicationHolder*"] {
                self->app->exec();
            })
        }
    }

    pub fn set_source(&mut self, src: QUrl) {
        unsafe {
            cpp!([self as "QmlApplicationHolder*", src as "QUrl"] {
                self->view->setSource(src);
            })
        }
    }

    pub fn set_title(&mut self, title: QString) {
        unsafe {
            cpp!([self as "QmlApplicationHolder*", title as "QString"] {
                self->view->setTitle(title);
            })
        }
    }

    pub fn set_application_version(&mut self, version: QString) {
        unsafe {
            cpp!([self as "QmlApplicationHolder*", version as "QString"] {
                self->app->setApplicationVersion(version);
            })
        }
    }

    pub fn install_default_translator(&mut self) -> Result<(), anyhow::Error> {
        let result = unsafe {
            cpp!([self as "QmlApplicationHolder*"] -> u32 as "int" {
                const QString transDir = SailfishApp::pathTo(QStringLiteral("translations")).toLocalFile();
                const QString appName = qApp->applicationName();
                QTranslator translator(qApp);
                int result = 0;
                if (!translator.load(QLocale(), appName, "-", transDir)) {
                    qWarning() << "Failed to load translator for" << QLocale::system().uiLanguages()
                               << "Searched" << transDir << "for" << appName;
                    result = 1;
                    if(!translator.load(appName, transDir)) {
                        qWarning() << "Could not load default translator either!";
                        result = 2;
                    }
                }
                self->app->installTranslator(&translator);
                return result;
            })
        };
        match result {
            0 => Ok(()),
            1 => {
                log::info!("Default translator loaded.");
                Ok(())
            }
            2 => anyhow::bail!("No translators found"),
            _ => unreachable!("Impossible return code from C++"),
        }
    }

    pub fn show(&self) {
        unsafe {
            cpp!([self as "QmlApplicationHolder*"] {
                self->view->show();
            })
        }
    }

    pub fn show_full_screen(&self) {
        unsafe {
            cpp!([self as "QmlApplicationHolder*"] {
                self->view->showFullScreen();
            })
        }
    }
}
