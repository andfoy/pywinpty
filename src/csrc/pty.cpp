#include "pty.h"

PTY::PTY(int cols, int rows) {
	winpty = NULL;
	conpty = NULL;
	used_backend = Backend::NONE;
	bool initialized = false;

	if (CONPTY_ENABLED) {
		// Check if the host has access to ConPTY API
		auto kernel32 = GetModuleHandleW(L"kernel32.dll");
		auto conpty_addr = GetProcAddress(kernel32, "CreatePseudoConsole");
		if (conpty_addr != NULL) {
			auto conpty_ref = ConPTY(cols, rows);
			conpty = &conpty_ref;
			initialized = true;
			used_backend = Backend::CONPTY;
		}
	}

	if (!initialized && WINPTY_ENABLED) {
		// Fallback to winpty API
		auto winpty_ref = WinptyPTY(cols, rows);
		winpty = &winpty_ref;
		used_backend = Backend::WINPTY;
	}
	else if (!initialized && !WINPTY_ENABLED && CONPTY_ENABLED) {
		throw std::runtime_error("pywinpty was compiled without WinPTY support and host does not support ConPTY");
	}
	else if (!initialized && !WINPTY_ENABLED) {
		throw std::runtime_error("pywinpty was compiled without ConPTY/WinPTY support");
	}
}

PTY::PTY(int cols, int rows, Backend backend) {
	winpty = NULL;
	conpty = NULL;
	used_backend = Backend::NONE;
	if (backend == Backend::CONPTY && CONPTY_ENABLED) {
		// Check if the host has access to ConPTY API
		auto kernel32 = GetModuleHandleW(L"kernel32.dll");
		auto conpty_addr = GetProcAddress(kernel32, "CreatePseudoConsole");
		if (conpty_addr != NULL) {
			auto conpty_ref = ConPTY(cols, rows);
			conpty = &conpty_ref;
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
		auto winpty_ref = WinptyPTY(cols, rows);
		winpty = &winpty_ref;
		used_backend = Backend::WINPTY;
	}
	else if (backend == Backend::WINPTY && !WINPTY_ENABLED) {
		throw std::runtime_error("pywinpty was compiled without WinPTY support");
	}
	else if (backend == Backend::NONE) {
		throw std::runtime_error("None is not a valid backend");
	}
}

PTY::PTY(int cols, int rows, int input_mode, int output_mode, bool override_pipes, int mouse_mode,
	     int timeout, int agent_config) {
	winpty = NULL;
	conpty = NULL;
	used_backend = Backend::NONE;
	bool initialized = false;

	if (CONPTY_ENABLED) {
		// Check if the host has access to ConPTY API
		auto kernel32 = GetModuleHandleW(L"kernel32.dll");
		auto conpty_addr = GetProcAddress(kernel32, "CreatePseudoConsole");
		if (conpty_addr != NULL) {
			auto conpty_ref = ConPTY(cols, rows, input_mode, output_mode);
			conpty = &conpty_ref;
			initialized = true;
			used_backend = Backend::CONPTY;
		}
	}

	if (!initialized && WINPTY_ENABLED) {
		// Fallback to winpty API
		auto winpty_ref = WinptyPTY(cols, rows, override_pipes, mouse_mode, timeout, agent_config);
		winpty = &winpty_ref;
		used_backend = Backend::WINPTY;
	}
	else if (!initialized && !WINPTY_ENABLED && CONPTY_ENABLED) {
		throw std::runtime_error("pywinpty was compiled without WinPTY support and host does not support ConPTY");
	}
	else if (!initialized && !WINPTY_ENABLED) {
		throw std::runtime_error("pywinpty was compiled without ConPTY/WinPTY support");
	}
}

PTY::PTY(int cols, int rows, Backend backend, int input_mode, int output_mode, bool override_pipes, int mouse_mode,
	     int timeout, int agent_config) {
	winpty = NULL;
	conpty = NULL;
	used_backend = Backend::NONE;
	if (backend == Backend::CONPTY && CONPTY_ENABLED) {
		// Check if the host has access to ConPTY API
		auto kernel32 = GetModuleHandleW(L"kernel32.dll");
		auto conpty_addr = GetProcAddress(kernel32, "CreatePseudoConsole");
		if (conpty_addr != NULL) {
			auto conpty_ref = ConPTY(cols, rows, input_mode, output_mode);
			conpty = &conpty_ref;
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
		auto winpty_ref = WinptyPTY(cols, rows, override_pipes, mouse_mode, timeout, agent_config);
		winpty = &winpty_ref;
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
		return conpty->spawn(appname, cmdline, cwd, env);
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

std::wstring PTY::read(uint64_t length, bool blocking) {
	if (used_backend == Backend::CONPTY) {
		return conpty->read(length, blocking);
	}
	else if (used_backend == Backend::WINPTY) {
		return winpty->read(length, blocking);
	}
	else {
		throw std::runtime_error("PTY was not initialized");
	}
}

std::wstring PTY::read_stderr(uint64_t length, bool blocking) {
	if (used_backend == Backend::CONPTY) {
		return conpty->read_stderr(length, blocking);
	}
	else if (used_backend == Backend::WINPTY) {
		return winpty->read_stderr(length, blocking);
	}
	else {
		throw std::runtime_error("PTY was not initialized");
	}
}

std::pair<bool, DWORD> PTY::write(std::wstring str) {
	if (used_backend == Backend::CONPTY) {
		return conpty->write(str);
	}
	else if (used_backend == Backend::WINPTY) {
		return winpty->write(str);
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
