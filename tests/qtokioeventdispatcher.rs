use harbour_whisperfish::*;

#[test]
fn install_qeventdispatcher() {
    sfos::TokioQEventDispatcher::install();
}
