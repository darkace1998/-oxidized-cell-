/**
 * oxidized-cell FFI implementation
 */

#include "oc_ffi.h"
#include <cstdlib>

extern "C" {

int oc_init(void) {
    // Initialize C++ runtime
    return 0;
}

void oc_shutdown(void) {
    // Shutdown C++ runtime
}

} // extern "C"
