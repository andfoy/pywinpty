#include "conpty_common.h"
#include <string>

#ifdef ENABLE_CONPTY


HRESULT SetUpPseudoConsole(HPCON* hPC, COORD size, HANDLE* inputReadSide, HANDLE* outputWriteSide,
	                       HANDLE* outputReadSide, HANDLE* inputWriteSide) {
	HRESULT hr = S_OK;

	// Create communication channels
	// - Close these after CreateProcess of child application with pseudoconsole object.
	// HANDLE inputReadSide, outputWriteSide;
	// - Hold onto these and use them for communication with the child through the pseudoconsole.
	// HANDLE outputReadSide, inputWriteSide;

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


ConPTY::ConPTY(int cols, int rows, int input_mode, int output_mode) {
    pty_started = false;
	pty_created = false;

    wchar_t szCommand[]{ L"c:\\windows\\system32\\cmd.exe" };
    HRESULT hr{ E_UNEXPECTED };
    //HANDLE hConsole = { GetStdHandle(STD_OUTPUT_HANDLE) };

    // Enable Console VT Processing
    DWORD consoleMode{};
    //GetConsoleMode(hConsole, &consoleMode);
    //hr = SetConsoleMode(hConsole, consoleMode | ENABLE_VIRTUAL_TERMINAL_PROCESSING)
    //    ? S_OK
    //    : GetLastError();
	
	// Create communication channels
	// - Close these after CreateProcess of child application with pseudoconsole object.
	HANDLE inputReadSide{ INVALID_HANDLE_VALUE };
	HANDLE outputWriteSide{ INVALID_HANDLE_VALUE };
	// - Hold onto these and use them for communication with the child through the pseudoconsole.
	HANDLE outputReadSide{ INVALID_HANDLE_VALUE };
	HANDLE inputWriteSide{ INVALID_HANDLE_VALUE };

	// Setup PTY size
	COORD size = {};
	size.X = rows;
	size.Y = cols;

	hr = SetUpPseudoConsole(&pty_handle, size, &inputReadSide, &outputWriteSide,
		&outputReadSide, &inputWriteSide);
	
	if (hr != S_OK) {
        char* err = new char[250];
        if (!FormatMessage(FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM,
            NULL, hr,
            MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT), // default language
            (LPTSTR)&err, 0, NULL)) {
            throw std::runtime_error("An unexpected error has occurred");
        }

        throw std::runtime_error(err);
        LocalFree(err);
	}

	this->inputReadSide = inputReadSide;
	this->outputWriteSide = outputWriteSide;
	this->outputReadSide = outputReadSide;
	this->inputWriteSide = inputWriteSide;
	pty_created = true;

}

ConPTY::~ConPTY() {
	std::cout << "Calling ConPTY destructor" << std::endl;
    /**if (pty_started) {
        // Close process
        CloseHandle(process_info.hThread);
        CloseHandle(process_info.hProcess);

        // Cleanup attribute list
        DeleteProcThreadAttributeList(startup_info.lpAttributeList);
        free(startup_info.lpAttributeList);
    }**/

    if (pty_created) {
        // Close ConPTY - this will terminate client process if running
        ClosePseudoConsole(pty_handle);

        // Clean-up the pipes
        if (INVALID_HANDLE_VALUE != outputReadSide) CloseHandle(outputReadSide);
        if (INVALID_HANDLE_VALUE != inputWriteSide) CloseHandle(inputWriteSide);
    }
}
 
bool ConPTY::spawn(std::wstring appname, std::wstring cmdline, std::wstring cwd, std::wstring env) {
	HRESULT hr{ E_UNEXPECTED };
	STARTUPINFOEX siEx;
	hr = PrepareStartupInformation(pty_handle, &siEx);

	if (hr != S_OK) {
		char* err;
		if (!FormatMessage(FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM,
			NULL, hr,
			MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT), // default language
			(LPTSTR)&err, 0, NULL)) {
			throw std::runtime_error("An unexpected error has occurred");
		}

		std::cerr << err << std::endl;
		LocalFree(err);
		return false;
	}

	PCWSTR childApplication = appname.c_str(); // L"C:\\windows\\system32\\cmd.exe";

	// Create mutable text string for CreateProcessW command line string.
	const size_t charsRequired = wcslen(childApplication) + 1; // +1 null terminator
	PWSTR cmdLineMutable = (PWSTR)HeapAlloc(GetProcessHeap(), 0, sizeof(wchar_t) * charsRequired);

	if (!cmdLineMutable)
	{
		hr = E_OUTOFMEMORY;
	}

	if (hr != S_OK) {
		char* err;
		if (!FormatMessage(FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM,
			NULL, hr,
			MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT), // default language
			(LPTSTR)&err, 0, NULL)) {
			throw std::runtime_error("An unexpected error has occurred");
		}

		std::cerr << err << std::endl;
		LocalFree(err);
		return false;
	}

	wcscpy_s(cmdLineMutable, charsRequired, childApplication);

	PROCESS_INFORMATION pi;
	ZeroMemory(&pi, sizeof(pi));

	/**
	siEx.StartupInfo.hStdInput = inputReadSide;
	siEx.StartupInfo.hStdError = outputWriteSide;
	siEx.StartupInfo.hStdOutput = outputWriteSide;
	siEx.StartupInfo.dwFlags |= STARTF_USESTDHANDLES;**/

	wchar_t szCommand[]{ L"c:\\windows\\system32\\cmd.exe" };
	wchar_t* env_test = L"\0PATH=C:\\Users\\andfoy-windows\\AppData\\Local\\Continuum\\miniconda3\\envs\\pywinpty;C:\\Users\\andfoy-windows\\AppData\\Local\\Continuum\\miniconda3\\envs\\pywinpty\\Library\\mingw-w64\\bin;C:\\Users\\andfoy-windows\\AppData\\Local\\Continuum\\miniconda3\\envs\\pywinpty\\Library\\usr\\bin;C:\\Users\\andfoy-windows\\AppData\\Local\\Continuum\\miniconda3\\envs\\pywinpty\\Library\\bin;C:\\Users\\andfoy-windows\\AppData\\Local\\Continuum\\miniconda3\\envs\\pywinpty\\Scripts;C:\\Users\\andfoy-windows\\AppData\\Local\\Continuum\\miniconda3\\envs\\pywinpty\\bin;C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\Community\\VC\\Tools\\MSVC\\14.25.28610\\bin\\HostX64\\x64;C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\Community\\Common7\\IDE\\VC\\VCPackages;C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\Community\\Common7\\IDE\\CommonExtensions\\Microsoft\\TestWindow;C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\Community\\Common7\\IDE\\CommonExtensions\\Microsoft\\TeamFoundation\\Team Explorer;C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\Community\\MSBuild\\Current\\bin\\Roslyn;C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\Community\\Team Tools\\Performance Tools\\x64;C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\Community\\Team Tools\\Performance Tools;C:\\Program Files (x86)\\Microsoft Visual Studio\\Shared\\Common\\VSPerfCollectionTools\\vs2019\\x64;C:\\Program Files (x86)\\Microsoft Visual Studio\\Shared\\Common\\VSPerfCollectionTools\\vs2019;C:\\Program Files (x86)\\Windows Kits\\10\\bin\\10.0.18362.0\\x64;C:\\Program Files (x86)\\Windows Kits\\10\\bin\\x64;C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\Community\\MSBuild\\Current\\Bin;C:\\Windows\\Microsoft.NET\\Framework64\\v4.0.30319;C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\Community\\Common7\\IDE;C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\Community\\Common7\\Tools;C:\\Users\\andfoy-windows\\AppData\\Local\\Continuum\\miniconda3\\condabin;C:\\Windows\\system32;C:\\Windows;C:\\Windows\\System32\\Wbem;C:\\Windows\\System32\\WindowsPowerShell\\v1.0;C:\\Windows\\System32\\OpenSSH;C:\\Program Files\\Git\\cmd;C:\\ProgramData\\chocolatey\\bin;C:\\Program Files (x86)\\Windows Kits\\10\\Windows Performance Toolkit;C:\\Users\\andfoy-windows\\.cargo\\bin;C:\\Users\\andfoy-windows\\AppData\\Local\\Continuum\\miniconda3;C:\\Users\\andfoy-windows\\AppData\\Local\\Continuum\\miniconda3\\Library\\mingw-w64\\bin;C:\\Users\\andfoy-windows\\AppData\\Local\\Continuum\\miniconda3\\Library\\usr\\bin;C:\\Users\\andfoy-windows\\AppData\\Local\\Continuum\\miniconda3\\Library\\bin;C:\\Users\\andfoy-windows\\AppData\\Local\\Continuum\\miniconda3\\Scripts;C:\\Users\\andfoy-windows\\AppData\\Local\\Microsoft\\WindowsApps\0";

	// Call CreateProcess
	if (!CreateProcessW(NULL,
		cmdLineMutable,
		NULL,
		NULL,
		FALSE,
		EXTENDED_STARTUPINFO_PRESENT | CREATE_UNICODE_ENVIRONMENT,
		NULL, //(void*)env_test,
		NULL,
		&siEx.StartupInfo,
		&pi))
	{
		HeapFree(GetProcessHeap(), 0, cmdLineMutable);
		hr = GetLastError();
		
		char* err;
		LPVOID lpMsgBuf;
		if (!FormatMessage(
			FORMAT_MESSAGE_ALLOCATE_BUFFER |
			FORMAT_MESSAGE_FROM_SYSTEM |
			FORMAT_MESSAGE_IGNORE_INSERTS,
			NULL,
			hr,
			MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT),
			(LPTSTR)&lpMsgBuf,
			0, NULL)) {
			throw std::runtime_error("An unexpected error has occurred");
		}

		std::cerr << (LPTSTR) lpMsgBuf << std::endl;
		//throw std::runtime_error(err);
		LocalFree(err);
		return false;
	}

	CloseHandle(inputReadSide);
	CloseHandle(outputWriteSide);

	conout = outputReadSide;
	conin = inputWriteSide;
	pid = pi.dwProcessId;
	process = pi.hProcess;
	pty_started = true;
	//startup_info = siEx;
	process_info = pi;
	//std::cout << "Process is alive: " << is_alive() << std::endl;
	/**const DWORD BUFF_SIZE{ 512 };
	char szBuffer[BUFF_SIZE]{};

	DWORD dwBytesRead{};
	bool result = read(szBuffer, BUFF_SIZE, true); // ReadFile(conout, szBuffer, BUFF_SIZE, &dwBytesRead, NULL);**/

    return true;
}

void ConPTY::set_size(int cols, int rows) {
    COORD consoleSize{};
    consoleSize.X = rows;
    consoleSize.Y = cols;
    HRESULT hr = ResizePseudoConsole(pty_handle, consoleSize);

    if (hr != S_OK) {
        char* err = new char[250];
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

uint32_t ConPTY::read_stderr(char* buf, uint64_t length, bool blocking) {
    throw std::runtime_error("ConPTY stderr reading is disabled");
}
