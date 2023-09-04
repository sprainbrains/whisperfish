use log::{log, Level};
use qmetaobject::{log::*, prelude::*, QMessageLogContext, QtMsgType};

static QLEVEL: &[Level] = &[
    Level::Debug, // 0 = QDebug
    Level::Warn,  // 1 = QWarning
    Level::Error, // 2 = QCritical
    Level::Error, // 3 = QFatal
    Level::Info,  // 4 = QInfo
    Level::Error, // 5 = QSystem
    Level::Error, // 6 = _
];

const FILE_START: &str = "file:///usr/share/be.rubdos.harbour.whisperfish/";

#[no_mangle]
pub extern "C" fn log_qt(msg_type: QtMsgType, msg_context: &QMessageLogContext, msg: &QString) {
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

pub fn enable() -> QtMessageHandler {
    install_message_handler(Some(log_qt))
}

pub fn disable() -> QtMessageHandler {
    install_message_handler(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qml_to_rust_logging() {
        let handler_a = enable();
        assert!(handler_a.is_some());

        let handler_b = disable();
        assert!(handler_b.is_some());

        assert_ne!(handler_a.unwrap() as usize, handler_b.unwrap() as usize);

        let handler_b = enable();
        assert!(handler_b.is_some());

        assert_eq!(handler_a.unwrap() as usize, handler_b.unwrap() as usize);
    }
}
