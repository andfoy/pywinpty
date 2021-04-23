#include "pty.h"

#ifdef DEBUG
// Debug utilities used to print a stack trace
// In order to use it:
// MyStackWalker sw; sw.ShowCallstack();
#include "StackWalker.h"
#endif


PTY::PTY(int cols, int rows, int mouse_mode, int timeout, int agent_config) {
    winpty = NULL;
    conpty = NULL;
    used_backend = Backend::NONE;
    bool initialized = false;

    if (CONPTY_ENABLED) {
        // Check if the host has access to ConPTY API
        auto kernel32 = GetModuleHandleW(L"kernel32.dll");
        auto conpty_addr = GetProcAddress(kernel32, "CreatePseudoConsole");
        if (conpty_addr != NULL) {
            conpty = new ConPTY(cols, rows);
            initialized = true;
            used_backend = Backend::CONPTY;
        }
    }

    if (!initialized && WINPTY_ENABLED) {
        // Fallback to winpty API
        winpty = new WinptyPTY(cols, rows, mouse_mode, timeout, agent_config);
        used_backend = Backend::WINPTY;
    }
    else if (!initialized && !WINPTY_ENABLED && CONPTY_ENABLED) {
        throw std::runtime_error("pywinpty was compiled without WinPTY support and host does not support ConPTY");
    }
    else if (!initialized && !WINPTY_ENABLED) {
        throw std::runtime_error("pywinpty was compiled without ConPTY/WinPTY support");
    }
}

PTY::PTY(int cols, int rows, Backend backend, int mouse_mode, int timeout, int agent_config) {
    winpty = NULL;
    conpty = NULL;
    used_backend = Backend::NONE;
    if (backend == Backend::CONPTY && CONPTY_ENABLED) {
        // Check if the host has access to ConPTY API
        auto kernel32 = GetModuleHandleW(L"kernel32.dll");
        auto conpty_addr = GetProcAddress(kernel32, "CreatePseudoConsole");
        if (conpty_addr != NULL) {
            conpty = new ConPTY(cols, rows);
            used_backend = Backend::CONPTY;
        }
        else {
            throw std::runtime_error("Host does not support ConPTY");
        }
    }
    else if (backend == Backend::CONPTY && !CONPTY_ENABLED) {
        throw std::runtime_error("pywinpty was compiled without ConPTY support");
    }
    else if (backend == Backend::WINPTY && WINPTY_ENABLED) {
        winpty = new WinptyPTY(cols, rows, mouse_mode, timeout, agent_config);
        used_backend = Backend::WINPTY;
    }
    else if (backend == Backend::WINPTY && !WINPTY_ENABLED) {
        throw std::runtime_error("pywinpty was compiled without WinPTY support");
    }
    else if (backend == Backend::NONE) {
        throw std::runtime_error("None is not a valid backend");
    }
}


PTY::~PTY() {
    if (used_backend == Backend::CONPTY) {
        delete conpty;
    }
    else if (used_backend == Backend::WINPTY) {
        delete winpty;
    }
}

bool PTY::spawn(std::wstring appname, std::wstring cmdline,
           std::wstring cwd, std::wstring env) {
    if (used_backend == Backend::CONPTY) {
        bool value = conpty->spawn(appname, cmdline, cwd, env);
        return value;
    }
    else if (used_backend == Backend::WINPTY) {
        return winpty->spawn(appname, cmdline, cwd, env);
    }
    else {
        throw std::runtime_error("PTY was not initialized");
    }
}

void PTY::set_size(int cols, int rows) {
    if (used_backend == Backend::CONPTY) {
        conpty->set_size(cols, rows);
    }
    else if (used_backend == Backend::WINPTY) {
        winpty->set_size(cols, rows);
    }
    else {
        throw std::runtime_error("PTY was not initialized");
    }
}

uint32_t PTY::read(char* buf, uint64_t length, bool blocking) {
    if (used_backend == Backend::CONPTY) {
        return conpty->read(buf, length, blocking);
    }
    else if (used_backend == Backend::WINPTY) {
        return winpty->read(buf, length, blocking);
    }
    else {
        throw std::runtime_error("PTY was not initialized");
    }
}

uint32_t PTY::read_stderr(char* buf, uint64_t length, bool blocking) {
    if (used_backend == Backend::CONPTY) {
        return conpty->read_stderr(buf, length, blocking);
    }
    else if (used_backend == Backend::WINPTY) {
        return winpty->read_stderr(buf, length, blocking);
    }
    else {
        throw std::runtime_error("PTY was not initialized");
    }
}

std::pair<bool, DWORD> PTY::write(const char* str, size_t length) {
    if (used_backend == Backend::CONPTY) {
        return conpty->write(str, length);
    }
    else if (used_backend == Backend::WINPTY) {
        return winpty->write(str, length);
    }
    else {
        throw std::runtime_error("PTY was not initialized");
    }
}

bool PTY::is_alive() {
    if (used_backend == Backend::CONPTY) {
        return conpty->is_alive();
    }
    else if (used_backend == Backend::WINPTY) {
        return winpty->is_alive();
    }
    else {
        throw std::runtime_error("PTY was not initialized");
    }
}

bool PTY::is_eof() {
    if (used_backend == Backend::CONPTY) {
        return conpty->is_eof();
    }
    else if (used_backend == Backend::WINPTY) {
        return winpty->is_eof();
    }
    else {
        throw std::runtime_error("PTY was not initialized");
    }
}

int64_t PTY::get_exitstatus() {
    if (used_backend == Backend::CONPTY) {
        return conpty->get_exitstatus();
    }
    else if (used_backend == Backend::WINPTY) {
        return winpty->get_exitstatus();
    }
    else {
        throw std::runtime_error("PTY was not initialized");
    }
}

uint32_t PTY::pid() {
    if (used_backend == Backend::CONPTY) {
        return conpty->pid;
    }
    else if (used_backend == Backend::WINPTY) {
        return winpty->pid;
    }
    else {
        throw std::runtime_error("PTY was not initialized");
    }
}
