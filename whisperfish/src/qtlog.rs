use cpp::cpp;
use log::{log, Level};
use qmetaobject::{prelude::*, QMessageLogContext};

cpp! {{
    #include <QtGlobal>
    #include <QtCore/QString>
}}

static QLEVEL: &[Level] = &[
    Level::Debug, // 0 = QDebug
    Level::Warn,  // 1 = QWarning
    Level::Error, // 2 = QCritical
    Level::Error, // 3 = QFatal
    Level::Info,  // 4 = QInfo
    Level::Error, // 5 = QSystem
    Level::Error, // 6 = _
];

const FILE_START: &str = "file:///usr/share/harbour-whisperfish/";

#[no_mangle]
pub extern "C" fn log_qt(msg_type: i32, msg_context: &QMessageLogContext, msg: &QString) {
    // QML may have prepended the message with the file information (so shorten it a bit),
    // or QMessageLogContext may provide it to us.
    let mut new_msg = msg.to_string();

    if new_msg.contains(FILE_START) {
        new_msg = new_msg.replace(FILE_START, "");
    } else if !msg_context.file().is_empty() {
        new_msg = format!(
            "{}:{}:{}(): {}",
            msg_context.file().replace(FILE_START, ""),
            msg_context.line(),
            msg_context.function(),
            msg
        );
    }

    let level = QLEVEL.get(msg_type as usize).unwrap_or(&QLEVEL[6]);
    log!(*level, "{}", new_msg);
}

cpp! {{
    extern "C" {
        void log_qt(QtMsgType msgType, const QMessageLogContext &msgContext, const QString msg);
    };

    void qDebugToRust(QtMsgType msgType, const QMessageLogContext &msgContext, const QString &msg)
    {
        log_qt(msgType, msgContext, msg);
        if (msgType == QtFatalMsg) {
            abort();
        }
    };
}}

pub fn install_message_handler() {
    unsafe {
        cpp!([] {
            qInstallMessageHandler(qDebugToRust);
        })
    };
}

#[cfg(test)]
mod tests {
    use cpp::cpp;

    #[test]
    fn qml_to_rust_logging() {
        cpp! {{
            #include <QDebug>
        }};

        let mut _logged = false;
        _logged = unsafe {
            cpp!([] -> bool as "bool" {
                qInstallMessageHandler(nullptr);
                qDebug() << "stderr";
                return true;
            })
        };
        assert!(_logged);

        _logged = false;
        _logged = unsafe {
            cpp!([] -> bool as "bool" {
                qInstallMessageHandler(qDebugToRust);
                qDebug() << "qInstallMessageHandler";
                return true;
            })
        };
        assert!(_logged);
    }
}
