#include "system.h"

#ifdef _WIN32
#include "ports/system_windows.c"
#else
#include "ports/system_posix.c"
#endif
