use std::os::raw::*;

use qmetaobject::qttypes::*;

/// Qt is not thread safe, and the engine can only be created once and in one thread.
/// So this is a guard that will be used to panic if the engine is created twice
static HAS_ENGINE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

cpp! {{
    #include <memory>
    #include <QtQuick/QtQuick>
    #include <QtCore/QDebug>
    #include <QtWidgets/QApplication>
    #include <QtQml/QQmlComponent>

    #include <sailfishapp.h>

    struct SfosSingleApplicationGuard {
        SfosSingleApplicationGuard() {
            rust!(Rust_SfosApplicationHolder_ctor[] {
                HAS_ENGINE.compare_exchange(false, true, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst)
                        .expect("There can only be one QmlEngine in the process");
            });
        }
        ~SfosSingleApplicationGuard() {
            rust!(Rust_SfosApplicationHolder_dtor[] {
                HAS_ENGINE.compare_exchange(true, false, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst)
                    .unwrap();
            });
        }
    };

    struct SfosApplicationHolder : SfosSingleApplicationGuard {
        std::unique_ptr<QGuiApplication> app;
        std::unique_ptr<QQuickView> view;

        SfosApplicationHolder(int &argc, char **argv)
            : app(SailfishApp::application(argc, argv))
            , view(SailfishApp::createView()) { }
    };
}}

cpp_class! (
    pub unsafe struct SailfishApp as "SfosApplicationHolder"
);

impl SailfishApp {
    pub fn application(name: String) -> SailfishApp {
        use std::ffi::CString;

        let mut arguments: Vec<*mut c_char> = std::iter::once(name).chain(std::env::args())
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
            cpp!([argc as "int", argv as "char**"] -> SailfishApp as "SfosApplicationHolder" {
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
                return SfosApplicationHolder(_argc, _argv);
            })
        };

        for arg in arguments {
            unsafe {
                CString::from_raw(arg);
            }
        }

        result
    }

    pub fn engine(&self) -> qmetaobject::QmlEngine {
        unimplemented!("Getting Qml engine is unimplemented")
    }

    pub fn path_to(path: QString) -> QUrl {
        unsafe {
            cpp!([path as "QString"] -> QUrl as "QUrl" {
                return SailfishApp::pathTo(path);
            })
        }
    }

    pub fn exec(&self) {
        unsafe {
            cpp!([self as "SfosApplicationHolder*"] {
                self->app->exec();
            })
        }
    }

    pub fn set_source(&mut self, src: QUrl) {
        unsafe {
            cpp!([self as "SfosApplicationHolder*", src as "QUrl"] {
                self->view->setSource(src);
            })
        }
    }

    pub fn set_title(&mut self, title: QString) {
        unsafe {
            cpp!([self as "SfosApplicationHolder*", title as "QString"] {
                self->view->setTitle(title);
            })
        }
    }

    pub fn set_application_version(&mut self, version: QString) {
        unsafe {
            cpp!([self as "SfosApplicationHolder*", version as "QString"] {
                self->app->setApplicationVersion(version);
            })
        }
    }

    pub fn install_default_translator(&mut self) {
        log::error!("Translation are unimplemented")
    //         const QString transDir = SailfishApp::pathTo(QStringLiteral("translations")).toLocalFile();
    //         const QLocale locale;
    //         if (!translator.load(locale, appName, "-", transDir, ".qm")) {
    //             qWarning() << "Failed to load translator for" << QLocale::system().uiLanguages()
    //                        << "Searched" << transDir << "for" << appName;
    //             if(!translator.load(appName, transDir)) {
    //                 qWarning() << "Could not load default translator either!";
    //             }
    //             app->installTranslator(&translator);
    //         }
    }
}
