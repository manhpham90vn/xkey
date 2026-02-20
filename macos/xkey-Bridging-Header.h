#ifndef XKEY_BRIDGING_HEADER_H
#define XKEY_BRIDGING_HEADER_H

#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

// CoreState is an opaque pointer to the Rust state
typedef void CoreState;

// Callbacks structure matching the Rust side
typedef struct {
    void *context;
    void (*update_preedit)(void *context, const uint8_t *text_utf8, size_t text_len, size_t caret, bool visible);
    void (*hide_preedit)(void *context);
    void (*commit)(void *context, const uint8_t *text_utf8, size_t text_len);
} XKeyCallbacks;

#ifdef __cplusplus
extern "C" {
#endif

/// Creates a new CoreState instance.
CoreState* xkey_core_create(void);

/// Destroys a CoreState instance.
void xkey_core_destroy(CoreState *core);

/// Clears the internal buffer of the CoreState.
void xkey_core_clear_buffer(CoreState *core);

/// Handles a key event and invokes the provided callbacks to perform actions.
/// Returns true if the key was consumed, false if it should be passed through.
bool xkey_core_handle_key(CoreState *core, uint32_t keyval, uint32_t state, XKeyCallbacks callbacks);

/// Commits any pending text in the buffer (used when deactivating).
/// Returns true if text was committed.
bool xkey_core_flush_buffer(CoreState *core, XKeyCallbacks callbacks);

#ifdef __cplusplus
}
#endif

#endif // XKEY_BRIDGING_HEADER_H
