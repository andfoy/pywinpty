#define NOMINMAX

#include <Windows.h>
#undef min
#undef max

#include <stdint.h>
#include <string>
#include <stdexcept>
#include <algorithm>

#ifdef ENABLE_WINPTY
static constexpr bool WINPTY_ENABLED = true;
#else
static constexpr bool WINPTY_ENABLED = false;
#endif
