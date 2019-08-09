#include <cassert>
#include <QtCore>

#include <openssl/evp.h>
#include <openssl/hmac.h>
#include <openssl/sha.h>

#include "store.hpp"
#include "model/prompt.hpp"

#include "whisperfish.hpp"

Store::Store() {
}

void Store::loadIdentity(Prompt *prompt) {
    auto paths = get_paths();

    storage_path = paths.data + "/storage";

    // Look for identity
    auto identity_path = storage_path + "/identity";
    if(!QFileInfo::exists(identity_path)) {
        qWarning() << "Generating identities not yet supported";
        qInfo() << "Looked for identity at" << identity_path;
        assert(false);
    }

    QFile identity_key(identity_path + "/identity_key");
    if(!identity_key.exists()) {
        qWarning() << "Generating identities not yet supported";
        assert(false);
    }
    assert(identity_key.open(QIODevice::ReadOnly));

    identity_keys = identity_key.readAll();

    switch(identity_keys.count()) {
    // Identity of 64 bytes means unencrypted.
    case 64:
        qInfo() << "Unencrypted identity";
        emit ready();
        break;
    // Identity of 128 bytes means encrypted.
    case 128:
        connect(prompt, &Prompt::receivePassword, this, &Store::supplyPassword);
        qInfo() << "Asking for password";
        prompt->promptPassword();
        break;
    default:
        qWarning() << "Key of strange length";
        assert(false);
        break;
    }

    // Look for salt
    QFile salt_file(storage_path + "/salt");
    if(salt_file.exists() && salt_file.open(QIODevice::ReadOnly)) {
        qDebug() << "Read salt file";
        salt = salt_file.readAll();
    }
}

bool Store::derive_keys(const QString password) {
    auto count = 1024;
    QByteArray password_utf8 = password.toUtf8();

    QByteArray bytes(16+20, 0);
    if(PKCS5_PBKDF2_HMAC_SHA1(password_utf8.data(), password_utf8.count(),
            (const unsigned char*) salt.data(), salt.count(), count,
            bytes.count(), (unsigned char*) bytes.data()) == 0)
        return false;
    key_material = bytes;

    return true;
}

void Store::supplyPassword(const QString password) {
    qDebug() << "Store received password";

    assert(derive_keys(password));
    qInfo() << "Derived keys. Length" << this->key_material.count();
    qInfo() << "Identity keys length" << this->identity_keys.count();

    // TODO: handle decryption failure
    if (read_identity_keys()) {
        qInfo() << "Decrypted identity keys";
    } else {
        qWarning() << "Failure decrypting identity";
        emit decryptionFailure();
    }
}

bool decrypt(const unsigned char *encryption_key,
        const unsigned char *mac_key,
        const unsigned char *ciphertext,
        size_t ciphertext_len,
        unsigned char *out) {
    // Check MAC, last 32 bytes
    const unsigned char *hmac_out = ciphertext + ciphertext_len - 32;
    // XXX: maybe reuse signal++?
    HMAC_CTX *ctx = (HMAC_CTX *)malloc(sizeof(HMAC_CTX));
    HMAC_CTX_init(ctx);
    assert(ctx);
    assert(HMAC_Init_ex(ctx, mac_key, 20, EVP_sha256(), 0) == 1);

    assert(HMAC_Update(ctx, ciphertext, ciphertext_len - 32) == 1);

    unsigned char md[EVP_MAX_MD_SIZE];
    unsigned int md_len = 0;
    HMAC_Final(ctx, md, &md_len);
    HMAC_CTX_cleanup(ctx);
    free(ctx);

    assert(md_len > 0);
    if (memcmp(md, hmac_out, md_len) != 0) return false;

    // Now do the actual decryption of ciphertext[16:-32], with iv=ciphertext[:16]

    return true;
}

bool Store::read_identity_keys() {
    QByteArray decrypted(64, 0);
    bool ret = decrypt((const unsigned char *)key_material.data(),
            (const unsigned char *)(key_material.data() + 16),
            (const unsigned char *)identity_keys.data(),
            identity_keys.count(),
            (unsigned char *)decrypted.data());

    if (ret) {
        identity_keys = decrypted;
        qInfo() << "Decrypted identity";
    }

    return ret;
}
