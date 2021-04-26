#pragma once

#include "base.h"
#include <cstdio>
#include <tchar.h>
#include <process.h>

// Handle to a ConPTY instance.
struct ConPTY: BaseProcess {
    // Main constructor
    ConPTY(int cols, int rows);

    // Main destructor
    ~ConPTY();

    // Spawn the process that the PTY will communicate to
    bool spawn(std::wstring appname, std::wstring cmdline = NULL,
               std::wstring cwd = NULL, std::wstring env = NULL);

    void set_size(int cols, int rows);

    uint32_t read_stderr(char* buf, uint64_t length, bool blocking);

    bool pty_created;
    bool pty_started;
    HPCON pty_handle;
    PROCESS_INFORMATION process_info;
    STARTUPINFOEX startup_info;

    HANDLE inputReadSide;
    HANDLE outputWriteSide;

    HANDLE outputReadSide;
    HANDLE inputWriteSide;
};
