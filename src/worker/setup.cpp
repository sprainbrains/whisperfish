#include "whisperfish.hpp"
#include "worker/setup.hpp"

SetupWorker::SetupWorker(QObject *parent): QObject(parent) {
}

bool SetupWorker::registered() const {
    auto paths = get_paths();

    auto storage_path = paths.data + "/storage";

    // Look for identity
    auto identity_path = storage_path + "/identity";
    QFile identity_key(identity_path + "/identity_key");
    return identity_key.exists();
}

QString SetupWorker::phoneNumber() const {
    // TODO
    return "Your phone number";
}

QString SetupWorker::identity() const {
    // TODO
    return "Your identity";
}
