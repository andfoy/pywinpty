#pragma once
#define NOMINMAX

#include <Windows.h>
#undef min
#undef max

#include <stdint.h>
#include <string>
#include <stdexcept>
#include <algorithm>
#include <iostream>


#ifdef ENABLE_WINPTY
static constexpr bool WINPTY_ENABLED = true;
#else
static constexpr bool WINPTY_ENABLED = false;
#endif // ENABLE_WINPTY

#ifdef ENABLE_CONPTY
static constexpr bool CONPTY_ENABLED = true;
#else
static constexpr bool CONPTY_ENABLED = false;
#endif // ENABLE_CONPTY

