#include "conpty_common.h"
#include <string>

/**
Native ConPTY calls.

See: https://docs.microsoft.com/en-us/windows/console/creating-a-pseudoconsole-session
**/

#ifdef ENABLE_CONPTY

HRESULT SetUpPseudoConsole(HPCON* hPC, COORD size, HANDLE* inputReadSide, HANDLE* outputWriteSide,
                           HANDLE* outputReadSide, HANDLE* inputWriteSide) {
    HRESULT hr = S_OK;

    if (!CreatePipe(inputReadSide, inputWriteSide, NULL, 0)) {
        return HRESULT_FROM_WIN32(GetLastError());
    }

    if (!CreatePipe(outputReadSide, outputWriteSide, NULL, 0)) {
        return HRESULT_FROM_WIN32(GetLastError());
    }

    hr = CreatePseudoConsole(size, *inputReadSide, *outputWriteSide, 0, hPC);
    return hr;
}


// Initializes the specified startup info struct with the required properties and
// updates its thread attribute list with the specified ConPTY handle
HRESULT PrepareStartupInformation(HPCON hpc, STARTUPINFOEX* psi)
{
    // Prepare Startup Information structure
    STARTUPINFOEX si;
    ZeroMemory(&si, sizeof(si));
    si.StartupInfo.cb = sizeof(STARTUPINFOEX);

    // Discover the size required for the list
    size_t bytesRequired;
    InitializeProcThreadAttributeList(NULL, 1, 0, &bytesRequired);

    // Allocate memory to represent the list
    si.lpAttributeList = (PPROC_THREAD_ATTRIBUTE_LIST)HeapAlloc(GetProcessHeap(), 0, bytesRequired);
    if (!si.lpAttributeList)
    {
        return E_OUTOFMEMORY;
    }

    // Initialize the list memory location
    if (!InitializeProcThreadAttributeList(si.lpAttributeList, 1, 0, &bytesRequired))
    {
        HeapFree(GetProcessHeap(), 0, si.lpAttributeList);
        return HRESULT_FROM_WIN32(GetLastError());
    }

    // Set the pseudoconsole information into the list
    if (!UpdateProcThreadAttribute(si.lpAttributeList,
        0,
        PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE,
        hpc,
        sizeof(hpc),
        NULL,
        NULL))
    {
        HeapFree(GetProcessHeap(), 0, si.lpAttributeList);
        return HRESULT_FROM_WIN32(GetLastError());
    }

    *psi = si;

    return S_OK;
}


ConPTY::ConPTY(int cols, int rows) {
    pty_started = false;
    pty_created = false;
    using_pipes = true;
    pid = 0;

    if (cols <= 0 || rows <= 0) {
        std::string prefix = "PTY cols and rows must be positive and non-zero. Got: ";
        std::string size = "(" + std::to_string(cols) + "," + std::to_string(rows) + ")";
        std::string error = prefix + size;
        throw std::runtime_error(error.c_str());
    }

    HRESULT hr{ E_UNEXPECTED };

    // Recreate the standard stream inputs in case the parent process
    // has redirected them
    HANDLE hConsole = CreateFile(
        L"CONOUT$",
        GENERIC_READ | GENERIC_WRITE,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        NULL, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, NULL);

    HANDLE hIn = CreateFile(
        L"CONIN$",
        GENERIC_READ | GENERIC_WRITE,
        FILE_SHARE_READ, 0, OPEN_EXISTING, 0, 0);

    if (hConsole != INVALID_HANDLE_VALUE) {
        // Enable Console VT Processing
        DWORD consoleMode{};
        hr = GetConsoleMode(hConsole, &consoleMode)
            ? S_OK
            : GetLastError();

        if (hr != S_OK) {
            throw_runtime_error(hr);
        }

        // Enable stream to accept VT100 input sequences
        hr = SetConsoleMode(hConsole, consoleMode | ENABLE_VIRTUAL_TERMINAL_PROCESSING)
            ? S_OK
            : GetLastError();

        if (hr != S_OK) {
            throw_runtime_error(hr);
        }

        // Set new streams
        SetStdHandle(STD_OUTPUT_HANDLE, hConsole);
        SetStdHandle(STD_INPUT_HANDLE, hIn);
    }

    // Create communication channels
    // - Close these after CreateProcess of child application with pseudoconsole object.
    HANDLE inputReadSide{ INVALID_HANDLE_VALUE };
    HANDLE outputWriteSide{ INVALID_HANDLE_VALUE };
    // - Hold onto these and use them for communication with the child through the pseudoconsole.
    HANDLE outputReadSide{ INVALID_HANDLE_VALUE };
    HANDLE inputWriteSide{ INVALID_HANDLE_VALUE };

    // Setup PTY size
    COORD size = {};
    size.X = cols;
    size.Y = rows;

    hr = SetUpPseudoConsole(&pty_handle, size, &inputReadSide, &outputWriteSide,
        &outputReadSide, &inputWriteSide);

    if (hr != S_OK) {
        throw_runtime_error(hr);
    }

    this->inputReadSide = inputReadSide;
    this->outputWriteSide = outputWriteSide;
    this->outputReadSide = outputReadSide;
    this->inputWriteSide = inputWriteSide;
    pty_created = true;

}

ConPTY::~ConPTY() {
    if (pty_started) {
        // Close process
        CloseHandle(process_info.hThread);
        CloseHandle(process_info.hProcess);

        // Cleanup attribute list
        DeleteProcThreadAttributeList(startup_info.lpAttributeList);
    }

    if (pty_created) {
        // Close ConPTY - this will terminate client process if running
        ClosePseudoConsole(pty_handle);

        // Clean-up the pipes
        if (INVALID_HANDLE_VALUE != outputReadSide) CloseHandle(outputReadSide);
        if (INVALID_HANDLE_VALUE != inputWriteSide) CloseHandle(inputWriteSide);
    }
}

bool ConPTY::spawn(std::wstring appname, std::wstring cmdline, std::wstring cwd, std::wstring env) {
    if (pty_started && is_alive()) {
        throw std::runtime_error("A process was already spawned and is running currently");
    }

    HRESULT hr{ E_UNEXPECTED };
    STARTUPINFOEX siEx;
    hr = PrepareStartupInformation(pty_handle, &siEx);

    if (hr != S_OK) {
        throw_runtime_error(hr);
    }

    PCWSTR childApplication = L"";
    if(cmdline.length() > 0) {
        childApplication = cmdline.c_str();
    }

    LPVOID environment = NULL;
    if (env.length() > 0) {
        environment = (void*)env.c_str();
    }

    LPCWSTR working_dir = NULL;
    if (cwd.length() > 0) {
        working_dir = cwd.c_str();
    }

    // Create mutable text string for CreateProcessW command line string.
    const size_t charsRequired = wcslen(childApplication) + 1; // +1 null terminator
    PWSTR cmdLineMutable = (PWSTR)HeapAlloc(GetProcessHeap(), 0, sizeof(wchar_t) * charsRequired);

    if (!cmdLineMutable) {
        hr = E_OUTOFMEMORY;
    }

    if (hr != S_OK) {
        throw_runtime_error(hr);
    }

    wcscpy_s(cmdLineMutable, charsRequired, childApplication);

    PROCESS_INFORMATION pi;
    ZeroMemory(&pi, sizeof(pi));

    // Call CreateProcess
    hr = CreateProcessW(
        appname.c_str(),    // Application name
        cmdLineMutable,     // Command line arguments
        NULL,               // Process attributes (unused)
        NULL,               // Thread attributes (unused)
        FALSE,              // Inherit pipes (false)
        EXTENDED_STARTUPINFO_PRESENT | CREATE_UNICODE_ENVIRONMENT,  // Process creation flags
        environment,              // Environment variables
        working_dir,              // Current working directory
        &siEx.StartupInfo,        // Startup info
        &pi                       // Process information
    ) ? S_OK : GetLastError();

    if (hr != S_OK) {
        HeapFree(GetProcessHeap(), 0, cmdLineMutable);
        throw_runtime_error(hr);
    }

    CloseHandle(inputReadSide);
    CloseHandle(outputWriteSide);

    conout = outputReadSide;
    conin = inputWriteSide;
    pid = pi.dwProcessId;
    process = pi.hProcess;
    pty_started = true;
    process_info = pi;

    return true;
}

void ConPTY::set_size(int cols, int rows) {
    if (cols <= 0 || rows <= 0) {
        std::string prefix = "PTY cols and rows must be positive and non-zero. Got: ";
        std::string size = "(" + std::to_string(cols) + "," + std::to_string(rows) + ")";
        std::string error = prefix + size;
        throw std::runtime_error(error.c_str());
    }

    COORD consoleSize{};
    consoleSize.X = cols;
    consoleSize.Y = rows;
    HRESULT hr = ResizePseudoConsole(pty_handle, consoleSize);

    if (hr != S_OK) {
        throw_runtime_error(hr);
    }
}
#else
ConPTY::ConPTY(int cols, int rows) {
    throw std::runtime_error("pywinpty was compiled without ConPTY support");
}

ConPTY::~ConPTY() {

}

bool ConPTY::spawn(std::wstring appname, std::wstring cmdline, std::wstring cwd, std::wstring env) {
    throw std::runtime_error("pywinpty was compiled without ConPTY support");
}

void ConPTY::set_size(int cols, int rows) {
    throw std::runtime_error("pywinpty was compiled without ConPTY support");
}
#endif  // ENABLE_CONPTY

uint32_t ConPTY::read_stderr(char* buf, uint64_t length, bool blocking) {
    throw std::runtime_error("ConPTY stderr reading is disabled");
}
