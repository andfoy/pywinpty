#include "conpty_common.h"


#ifdef ENABLE_CONPTY
HRESULT CreatePseudoConsoleAndPipes(int cols, int rows, HPCON* phPC, HANDLE* phPipeIn, HANDLE* phPipeOut) {
    HRESULT hr{ E_UNEXPECTED };
    HANDLE hPipePTYIn{ INVALID_HANDLE_VALUE };
    HANDLE hPipePTYOut{ INVALID_HANDLE_VALUE };

    // Create the pipes to which the ConPTY will connect
    if (CreatePipe(&hPipePTYIn, phPipeOut, NULL, 0) &&
        CreatePipe(phPipeIn, &hPipePTYOut, NULL, 0)) {

        // Define console size
        COORD consoleSize{};
        consoleSize.X = rows;
        consoleSize.Y = cols;

        // Create the Pseudo Console of the required size, attached to the PTY-end of the pipes
        hr = CreatePseudoConsole(consoleSize, hPipePTYIn, hPipePTYOut, 0, phPC);

        // Note: We can close the handles to the PTY-end of the pipes here
        // because the handles are dup'ed into the ConHost and will be released
        // when the ConPTY is destroyed.
        if (INVALID_HANDLE_VALUE != hPipePTYOut) CloseHandle(hPipePTYOut);
        if (INVALID_HANDLE_VALUE != hPipePTYIn) CloseHandle(hPipePTYIn);
    }

    return hr;
}

// Initializes the specified startup info struct with the required properties and
// updates its thread attribute list with the specified ConPTY handle
HRESULT InitializeStartupInfoAttachedToPseudoConsole(STARTUPINFOEXW* pStartupInfo, HPCON hPC)
{
    HRESULT hr{ E_UNEXPECTED };

    if (pStartupInfo)
    {
        size_t attrListSize{};

        pStartupInfo->StartupInfo.cb = sizeof(STARTUPINFOEXW);

        // Get the size of the thread attribute list.
        InitializeProcThreadAttributeList(NULL, 1, 0, &attrListSize);

        // Allocate a thread attribute list of the correct size
        pStartupInfo->lpAttributeList =
            reinterpret_cast<LPPROC_THREAD_ATTRIBUTE_LIST>(malloc(attrListSize));

        // Initialize thread attribute list
        if (pStartupInfo->lpAttributeList
            && InitializeProcThreadAttributeList(pStartupInfo->lpAttributeList, 1, 0, &attrListSize))
        {
            // Set Pseudo Console attribute
            hr = UpdateProcThreadAttribute(
                pStartupInfo->lpAttributeList,
                0,
                PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE,
                hPC,
                sizeof(HPCON),
                NULL,
                NULL)
                ? S_OK
                : HRESULT_FROM_WIN32(GetLastError());
        }
        else
        {
            hr = HRESULT_FROM_WIN32(GetLastError());
        }
    }
    return hr;
}


ConPTY::ConPTY(int cols, int rows, int input_mode, int output_mode) {
    pty_started = false;

	HRESULT hr{ E_UNEXPECTED };
	HANDLE hConsole = { GetStdHandle(STD_OUTPUT_HANDLE) };

    DWORD consoleMode{};
    GetConsoleMode(hConsole, &consoleMode);
    hr = SetConsoleMode(hConsole, consoleMode | input_mode | output_mode)
        ? S_OK
        : GetLastError();

    if (S_OK != hr) {

        char* err;
        if (!FormatMessage(FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM,
                           NULL, hr,
                           MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT), // default language
                           (LPTSTR)&err, 0, NULL)) {
            throw std::runtime_error("An unexpected error has occurred");
        }

        throw std::runtime_error(err);
        LocalFree(err);
    }

    HPCON hPC{ INVALID_HANDLE_VALUE };

    HANDLE hPipeIn{ INVALID_HANDLE_VALUE };
    HANDLE hPipeOut{ INVALID_HANDLE_VALUE };

    hr = CreatePseudoConsoleAndPipes(cols, rows, &hPC, &hPipeIn, &hPipeOut);

    if (S_OK != hr) {

        char* err;
        if (!FormatMessage(FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM,
            NULL, hr,
            MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT), // default language
            (LPTSTR)&err, 0, NULL)) {
            throw std::runtime_error("An unexpected error has occurred");
        }

        throw std::runtime_error(err);
        LocalFree(err);
    }

    pty_handle = hPC;
    conin = hPipeIn;
    conout = hPipeOut;
    pty_created = true;
}

ConPTY::~ConPTY() {
    if (pty_started) {
        // Close process
        CloseHandle(process_info.hThread);
        CloseHandle(process_info.hProcess);

        // Cleanup attribute list
        DeleteProcThreadAttributeList(startup_info.lpAttributeList);
        free(startup_info.lpAttributeList);
    }

    if (pty_created) {
        // Close ConPTY - this will terminate client process if running
        ClosePseudoConsole(pty_handle);

        // Clean-up the pipes
        if (INVALID_HANDLE_VALUE != conout) CloseHandle(conout);
        if (INVALID_HANDLE_VALUE != conin) CloseHandle(conin);
    }
}
 
bool ConPTY::spawn(std::wstring appname, std::wstring cmdline, std::wstring cwd, std::wstring env) {
    STARTUPINFOEXW startupInfo{};
    HRESULT hr{ E_UNEXPECTED };

    hr = InitializeStartupInfoAttachedToPseudoConsole(&startupInfo, pty_handle);
    if (hr != S_OK) {
        char* err;
        if (!FormatMessage(FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM,
            NULL, hr,
            MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT), // default language
            (LPTSTR)&err, 0, NULL)) {
            throw std::runtime_error("An unexpected error has occurred");
        }

        throw std::runtime_error(err);
        LocalFree(err);
    }

    const wchar_t* app_name = appname.c_str();

    PROCESS_INFORMATION piClient{};
    hr = CreateProcessW(
        app_name,                // Program path
        const_cast<wchar_t*>(cmdline.c_str()),                // Command Line
        NULL,                           // Process handle not inheritable
        NULL,                           // Thread handle not inheritable
        FALSE,                          // Inherit handles
        EXTENDED_STARTUPINFO_PRESENT,   // Creation flags
        (void*)(env.c_str()),           // Use parent's environment block
        cwd.c_str(),                    // Use parent's starting directory 
        &startupInfo.StartupInfo,       // Pointer to STARTUPINFO
        &piClient)                      // Pointer to PROCESS_INFORMATION
        ? S_OK
        : GetLastError();

    if (hr != S_OK) {
        char* err;
        if (!FormatMessage(FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM,
            NULL, hr,
            MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT), // default language
            (LPTSTR)&err, 0, NULL)) {
            std::cerr << "An unexpected error has occurred" << std::endl;
        }

        std::cerr << err << std::endl;
        LocalFree(err);
        return false;
    }

    startup_info = startupInfo;
    process_info = piClient;
    process = piClient.hProcess;
    pid = piClient.dwProcessId;
    alive = 1;
    pty_started = true;
    return true;
}

void ConPTY::set_size(int cols, int rows) {
    COORD consoleSize{};
    consoleSize.X = rows;
    consoleSize.Y = cols;
    HRESULT hr = ResizePseudoConsole(pty_handle, consoleSize);

    if (hr != S_OK) {
        char* err;
        if (!FormatMessage(FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM,
            NULL, hr,
            MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT), // default language
            (LPTSTR)&err, 0, NULL)) {
            throw std::runtime_error("An unexpected error has occurred");
        }

        throw std::runtime_error(err);
        LocalFree(err);
    }
}
#else
ConPTY::ConPTY(int cols, int rows, int input_mode, int output_mode) {
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

std::wstring ConPTY::read_stderr(uint64_t length, bool blocking) {
    throw std::runtime_error("ConPTY stderr reading is disabled");
}