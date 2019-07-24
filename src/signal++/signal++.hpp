#pragma once

#include <mutex>
#include <cassert>

#include "signal_protocol.h"

#include <openssl/opensslv.h>
#include <openssl/evp.h>
#include <openssl/hmac.h>
#include <openssl/rand.h>
#include <openssl/sha.h>

class SignalContext;

extern "C" {
    void __lock(void *);
    void __unlock(void *);
    int __hmac_sha256_init(
            void **hmac_context,
            const uint8_t *key,
            size_t key_len,
            void *user_data);
    int __hmac_sha256_update(
            void *hmac_context,
            const uint8_t *data,
            size_t data_len,
            void *user_data);
    int __hmac_sha256_final(
            void *hmac_context,
            signal_buffer **output,
            void *user_data);
    void __hmac_sha256_cleanup(
            void *hmac_context,
            void *user_data);
}

class SignalContext {
    signal_context *ctx;
    signal_crypto_provider crypto_provider;

    std::recursive_mutex signal_mutex;

public:
    SignalContext() {
        assert(signal_context_create(&ctx, this) == 0);

        crypto_provider.hmac_sha256_init_func = &__hmac_sha256_init;
        crypto_provider.hmac_sha256_update_func = &__hmac_sha256_update;
        crypto_provider.hmac_sha256_final_func = &__hmac_sha256_final;
        crypto_provider.hmac_sha256_cleanup_func = &__hmac_sha256_cleanup;

        assert(signal_context_set_crypto_provider(ctx, &crypto_provider) == 0);
        assert(signal_context_set_locking_functions(ctx, &__lock, &__unlock) == 0);

#ifdef QT_VERSION
        qDebug() << "SignalContext opened";
#endif
    }

    SignalContext(const SignalContext&) = delete;
    SignalContext& operator=(const SignalContext&) = delete;

    virtual ~SignalContext() {
        signal_context_destroy(ctx);

#ifdef QT_VERSION
        qDebug() << "SignalContext shut down";
#endif
    }

    void lock() {
        signal_mutex.lock();
    }

    void unlock() {
        signal_mutex.unlock();
    }
};

// Function implementations for callbacks from signal-c
extern "C" {
    // Signal promises us never to deadlock iff we have reentrant locks.
    void __lock(void *user) {
        SignalContext *ctx = (SignalContext *)user;
        ctx->lock();
    }
    void __unlock(void *user) {
        SignalContext *ctx = (SignalContext *)user;
        ctx->unlock();
    }

    // Crypto provider
    // Implementation taken from tests/test_common_openssl.c, GPLv3
    int __hmac_sha256_init(void **hmac_context, const uint8_t *key, size_t key_len, __attribute__((unused)) void *_user_data)
    {
#if OPENSSL_VERSION_NUMBER >= 0x1010000fL
        HMAC_CTX *ctx = HMAC_CTX_new();
        if(!ctx) {
            return SG_ERR_NOMEM;
        }
#else
        HMAC_CTX *ctx = (HMAC_CTX *)malloc(sizeof(HMAC_CTX));
        if(!ctx) {
            return SG_ERR_NOMEM;
        }
        HMAC_CTX_init(ctx);
#endif

        *hmac_context = (void *)ctx;

        if(HMAC_Init_ex(ctx, key, key_len, EVP_sha256(), 0) != 1) {
            return SG_ERR_UNKNOWN;
        }

        return 0;
    }

    int __hmac_sha256_update(void *hmac_context, const uint8_t *data, size_t data_len, __attribute__((unused)) void *user_data)
    {
        HMAC_CTX *ctx = (HMAC_CTX *)hmac_context;
        int result = HMAC_Update(ctx, data, data_len);
        return (result == 1) ? 0 : -1;
    }

    int __hmac_sha256_final(void *hmac_context, signal_buffer **output, __attribute__((unused)) void *user_data)
    {
        int result = 0;
        unsigned char md[EVP_MAX_MD_SIZE];
        unsigned int len = 0;
        HMAC_CTX *ctx = (HMAC_CTX *)hmac_context;

        if(HMAC_Final(ctx, md, &len) != 1) {
            return SG_ERR_UNKNOWN;
        }

        signal_buffer *output_buffer = signal_buffer_create(md, len);
        if(!output_buffer) {
            result = SG_ERR_NOMEM;
            goto complete;
        }

        *output = output_buffer;

    complete:
        return result;
    }

    void __hmac_sha256_cleanup(void *hmac_context, __attribute__((unused)) void *user_data)
    {
        if(hmac_context) {
            HMAC_CTX *ctx = (HMAC_CTX *)hmac_context;
#if OPENSSL_VERSION_NUMBER >= 0x1010000fL
            HMAC_CTX_free(ctx);
#else
            HMAC_CTX_cleanup(ctx);
            free(ctx);
#endif
        }
    }
}
