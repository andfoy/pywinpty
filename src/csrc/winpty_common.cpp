#include "winpty_common.h"

#ifdef ENABLE_WINPTY
void compose_error_message(winpty_error_ptr_t err, char* tmp) {
    std::wstring err_msg = winpty_error_msg(err);
    std::wstring err_code = std::to_wstring(winpty_error_code(err));

    std::wstring prefix = L"An error has occurred: ";
    std::wstring error = prefix + err_msg + L" - Code: " + err_code;

    sprintf(tmp, "%ls", error.c_str());
}

WinptyPTY::WinptyPTY(int cols, int rows, int mouse_mode,
                     int timeout, int agent_config) {
    alive = 0;
    pid = 0;

    winpty_error_ptr_t err;
    winpty_config_t* config = winpty_config_new(agent_config, &err);
    if (config == nullptr) {
        char tmp[256];
        compose_error_message(err, tmp);
        throw std::runtime_error(tmp);
    }

    if (cols <= 0 || rows <= 0) {
        std::string prefix = "PTY cols and rows must be positive and non-zero. Got: ";
        std::string size = "(" + std::to_string(cols) + "," + std::to_string(rows) + ")";
        std::string error = prefix + size;
        throw std::runtime_error(error.c_str());
    }

    winpty_config_set_initial_size(config, cols, rows);
    winpty_config_set_mouse_mode(config, mouse_mode);
    winpty_config_set_agent_timeout(config, timeout);

    winpty_error_ptr_t err_pointer;
    pty_ref = winpty_open(config, &err_pointer);
    winpty_config_free(config);

    if (pty_ref == nullptr) {
        char tmp[256];
        compose_error_message(err_pointer, tmp);
        throw std::runtime_error(tmp);
    }

    agent_process = winpty_agent_process(pty_ref);
    conin_pipe_name = winpty_conin_name(pty_ref);
    conout_pipe_name = winpty_conout_name(pty_ref);
    conerr_pipe_name = winpty_conerr_name(pty_ref);

    conin = CreateFileW(
        conin_pipe_name, GENERIC_WRITE, 0, NULL, OPEN_EXISTING, 0,
        NULL
    );

    conout = CreateFileW(
        conout_pipe_name, GENERIC_READ, 0, NULL, OPEN_EXISTING, 0,
        NULL
    );

    conerr = CreateFileW(
        conerr_pipe_name, GENERIC_READ, 0, NULL, OPEN_EXISTING, 0,
        NULL
    );
}

WinptyPTY::~WinptyPTY() {
    if (pty_ref != nullptr) {
        CloseHandle(conout);
        CloseHandle(conerr);
        CloseHandle(conin);
        winpty_free(pty_ref);
    }
}

bool WinptyPTY::spawn(std::wstring appname, std::wstring cmdline,
                      std::wstring cwd, std::wstring env) {

    LPCWSTR command_line = L"";
    if (cmdline.length() > 0) {
        command_line = cmdline.c_str();
    }

    LPCWSTR environment = NULL;
    if (env.length() > 0) {
        environment = env.c_str();
    }

    LPCWSTR working_dir = NULL;
    if (cwd.length() > 0) {
        working_dir = cwd.c_str();
    }

    winpty_error_ptr_t spawn_conf_err;
    winpty_spawn_config_t* spawn_config = winpty_spawn_config_new(WINPTY_SPAWN_FLAG_MASK,
        appname.c_str(), command_line, working_dir, environment, &spawn_conf_err);

    if (spawn_config == nullptr) {
        char tmp[256];
        compose_error_message(spawn_conf_err, tmp);
        throw std::runtime_error(tmp);
    }

    winpty_error_ptr_t spawn_err;
    bool succ = winpty_spawn(pty_ref, spawn_config, &process,
        NULL, NULL, &spawn_err);

    winpty_spawn_config_free(spawn_config);

    if (!succ) {
        char tmp[256];
        compose_error_message(spawn_err, tmp);
        throw std::runtime_error(tmp);
    }

    pid = GetProcessId(process);
    alive = 1;
    return succ;
}

void WinptyPTY::set_size(int cols, int rows) {
    if (cols <= 0 || rows <= 0) {
        std::string prefix = "PTY cols and rows must be positive and non-zero. Got: ";
        std::string size = "(" + std::to_string(cols) + "," + std::to_string(rows) + ")";
        std::string error = prefix + size;
        throw std::runtime_error(error.c_str());
    }

    winpty_error_ptr_t err_pointer;
    bool succ = winpty_set_size(pty_ref, cols, rows, &err_pointer);

    if (!succ) {
        char tmp[256];
        compose_error_message(err_pointer, tmp);
        throw std::runtime_error(tmp);
    }
}
#else
WinptyPTY::WinptyPTY(int cols, int rows, int mouse_mode,
    int timeout, int agent_config) {
    throw std::runtime_error("pywinpty was compiled without winpty support");
}

WinptyPTY::~WinptyPTY() {
    throw std::runtime_error("pywinpty was compiled without winpty support");
}

bool WinptyPTY::spawn(std::wstring appname, std::wstring cmdline,
    std::wstring cwd, std::wstring env) {
    throw std::runtime_error("pywinpty was compiled without winpty support");
}

void WinptyPTY::set_size(int cols, int rows) {
    throw std::runtime_error("pywinpty was compiled without winpty support");
}
#endif
