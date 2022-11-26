#include <valgrind/callgrind.h>

#define SXT_TOGGLE_COLLECT CALLGRIND_TOGGLE_COLLECT

void toggle_collect_c() {
    SXT_TOGGLE_COLLECT;
}
