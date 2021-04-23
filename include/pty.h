#pragma once

#include "winpty_common.h"
#include "conpty_common.h"

// Available backends for pywinpty
enum Backend {
    CONPTY,
    WINPTY,
    NONE
};

// Available encodings
enum Encoding {
    UTF8,
    UTF16
};

// Wrapper struct around ConPTY and WinPTY
struct PTY {
    // Partial constructor with automatic backend selection and extended parameters.
    PTY(int cols, int rows, int mouse_mode, int timeout, int agent_config);

    // Main constructor
    PTY(int cols, int rows, Backend backend, int mouse_mode, int timeout, int agent_config);

    // Main destructor
    ~PTY();

    // Spawn a process
    bool spawn(std::wstring appname, std::wstring cmdline = NULL,
               std::wstring cwd = NULL, std::wstring env = NULL);

    // Set the size of a PTY
    void set_size(int cols, int rows);

    // Read n bytes from the stdout stream.
    uint32_t read(char* buf, uint64_t length = 1000, bool blocking = false);

    // Read n bytes from the stderr stream.
    uint32_t read_stderr(char* buf, uint64_t length = 1000, bool blocking = false);

    // Write bytes to the stdin stream.
    std::pair<bool, DWORD> write(const char* str, size_t length);

    // Determine if the process is alive.
    bool is_alive();

    // Determine if the process ended.
    bool is_eof();

    // Get the exit status code of the process.
    int64_t get_exitstatus();

    // Get the PID of the process.
    uint32_t pid();

    ConPTY* conpty;
    WinptyPTY* winpty;
    Backend used_backend;
};
