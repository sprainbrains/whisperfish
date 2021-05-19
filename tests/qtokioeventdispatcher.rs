use harbour_whisperfish::*;

#[test]
fn install_qeventdispatcher() {
    qmlapp::TokioQEventDispatcher::install();
}
