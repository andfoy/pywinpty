#include "winpty_common.h"
#include <stdexcept>
#include <algorithm>


void compose_error_message(winpty_error_ptr_t err, char* tmp) {
	std::wstring err_msg = winpty_error_msg(err);
	std::wstring err_code = std::to_wstring(winpty_error_code(err));
	// convertLPWToString(&err_msg, winpty_error_msg(err));
	// std::wstringstream mBufferStream;
	std::wstring prefix = L"An error has occurred: ";
	std::wstring error = prefix + err_msg + L" - Code: " + err_code;

	//char tmp[256];
	sprintf(tmp, "%ls", error.c_str());
	//return tmp;
}

WinptyPTY::WinptyPTY(int cols, int rows, bool override_pipes, int mouse_mode,
	                 int timeout, int agent_config) {
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

	alive = 0;
	pid = 0;
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
	winpty_error_ptr_t spawn_conf_err;
	winpty_spawn_config_t* spawn_config = winpty_spawn_config_new(WINPTY_SPAWN_FLAG_MASK,
		appname.c_str(), cmdline.c_str(), cwd.c_str(), env.c_str(), &spawn_conf_err);
	
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

bool WinptyPTY::is_alive() {
	DWORD lpExitCode;
	bool succ = GetExitCodeProcess(process, &lpExitCode);
	if (!succ) {
		throw std::runtime_error("Could not check status");
	}

	// Check for STILL_ACTIVE flag
	// https://msdn.microsoft.com/en-us/library/windows/desktop/ms683189(v=vs.85).aspx
	alive = lpExitCode == STILL_ACTIVE;
	if (!alive) {
		alive = 0;
		exitstatus = lpExitCode;
	}
	return alive;
}

std::wstring WinptyPTY::read(uint64_t length, bool blocking) {
	PLARGE_INTEGER size_p;
	if (!blocking) {
		GetFileSizeEx(conout, size_p);
		LONGLONG expected_length = (*size_p).QuadPart;
		length = std::min(static_cast<uint64_t>(expected_length), length);
	}

	std::wstring data;
	data.reserve(length);
	if (length > 0) {
		LPDWORD num_bytes;
		ReadFile(conout, (void*)data.data(), length, num_bytes, NULL);
	}
	return data;
}

std::pair<bool, DWORD> WinptyPTY::write(std::wstring str) {
	LPDWORD num_bytes;
	bool success = WriteFile(conin, (void*)str.data(), str.size(), num_bytes, NULL);
	return std::make_pair(success, *num_bytes);
}


bool WinptyPTY::is_eof() {
	bool succ = PeekNamedPipe(conout, NULL, false, NULL, NULL, NULL);
	return !succ;
}


int64_t WinptyPTY::get_exitstatus() {
	if (pid == 0) {
		return -1;
	}
	if (alive == 1) {
		is_alive();
	}
	if (alive == 1) {
		return -1;
	}
	return exitstatus;
}
