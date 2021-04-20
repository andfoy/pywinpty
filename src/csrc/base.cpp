#include "base.h"

std::wstring base_read(uint64_t length, bool blocking, HANDLE stream) {
	PLARGE_INTEGER size_p;
	if (!blocking) {
		GetFileSizeEx(stream, size_p);
		LONGLONG expected_length = (*size_p).QuadPart;
		length = std::min(static_cast<uint64_t>(expected_length), length);
	}

	std::wstring data;
	data.reserve(length);
	if (length > 0) {
		LPDWORD num_bytes;
		ReadFile(stream, (void*)data.data(), length, num_bytes, NULL);
	}
	return data;
}

std::wstring BaseProcess::read(uint64_t length, bool blocking) {
	return base_read(length, blocking, conout);
}

std::wstring BaseProcess::read_stderr(uint64_t length, bool blocking) {
	return base_read(length, blocking, conerr);
}

std::pair<bool, DWORD> BaseProcess::write(std::wstring str) {
	LPDWORD num_bytes;
	bool success = WriteFile(conin, (void*)str.data(), str.size(), num_bytes, NULL);
	return std::make_pair(success, *num_bytes);
}

bool BaseProcess::is_eof() {
	bool succ = PeekNamedPipe(conout, NULL, false, NULL, NULL, NULL);
	return !succ;
}

int64_t BaseProcess::get_exitstatus() {
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


bool BaseProcess::is_alive() {
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

