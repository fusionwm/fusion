#include "generated/general.h"
#include "generated/compositor.h"
#include "shortcuts.h"
#include <stdlib.h>

inline void init(void) {
    string_t str;
    set_string(&str, "(Init) Hello, world!");
    info(&str);
}

inline void tick(void) {}
inline void stop(void) {}

inline void rearrange_windows(void) {
    list_window ret;
    get_elements(&ret);

    if (ret.len == 0) {
        return;
    }
}
