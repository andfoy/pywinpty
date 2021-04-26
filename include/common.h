#pragma once
#define NOMINMAX
#define UNICODE

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


static void throw_runtime_error(HRESULT hr, bool throw_exception = true) {
    LPSTR messageBuffer = nullptr;

    //Ask Win32 to give us the string version of that message ID.
    //The parameters we pass in, tell Win32 to create the buffer that holds the message for us (because we don't yet know how long the message string will be).
    size_t size = FormatMessageA(FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
        NULL, hr, MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT), (LPSTR)&messageBuffer, 0, NULL);

    //Copy the error message into a std::string.
    std::string message(messageBuffer, size);

    //Free the Win32's string's buffer.
    LocalFree(messageBuffer);

    if (throw_exception) {
        throw std::runtime_error(message.c_str());
    }

    std::cerr << message << std::endl;
}
