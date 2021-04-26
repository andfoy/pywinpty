#pragma once
#include "base.h"


#ifdef ENABLE_WINPTY
extern "C" {
    #include <winpty.h>
    #include <winpty_constants.h>
}

struct WinptyPTY: BaseProcess {
    // Main constructor
    WinptyPTY(int cols, int rows, int mouse_mode = WINPTY_MOUSE_MODE_NONE,
        int timeout = 30000, int agent_config = WINPTY_FLAG_COLOR_ESCAPES);

    // Destructor
    ~WinptyPTY();

    bool spawn(std::wstring appname, std::wstring cmdline = NULL,
               std::wstring cwd = NULL, std::wstring env = NULL);

    void set_size(int cols, int rows);

    winpty_t* pty_ref;
    HANDLE agent_process;
    LPCWSTR conin_pipe_name;
    LPCWSTR conout_pipe_name;
    LPCWSTR conerr_pipe_name;
};
#else
struct WinptyPTY: BaseProcess {
    // Main constructor
    WinptyPTY(int cols, int rows, int mouse_mode,
              int timeout, int agent_config);

    // Destructor
    ~WinptyPTY();

    bool spawn(std::wstring appname, std::wstring cmdline = NULL,
               std::wstring cwd = NULL, std::wstring env = NULL);

    void set_size(int cols, int rows);

    HANDLE agent_process;
    LPCWSTR conin_pipe_name;
    LPCWSTR conout_pipe_name;
    LPCWSTR conerr_pipe_name;
};
#endif
