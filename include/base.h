#pragma once
#include "common.h"


// Base struct that contains methods to read/write to the standard streams of a process.
struct BaseProcess {
	// Read n bytes from the stdout stream.
	uint32_t read(char* buf, uint64_t length = 1000, bool blocking = false);

	// Read n bytes from the stderr stream.
	uint32_t read_stderr(char* buf, uint64_t length = 1000, bool blocking = false);

	// Write bytes to the stdin stream.
	std::pair<bool, DWORD> write(std::wstring str);

	// Determine if the process is alive.
	bool is_alive();

	// Determine if the process ended.
	bool is_eof();

	// Get the exit status code of the process.
	int64_t get_exitstatus();

	// Handle to the process.
	HANDLE process;

	// Handle to the stdin stream.
	HANDLE conin;

	// Handle to the stdout stream.
	HANDLE conout;

	// Handle to the stderr stream.
	HANDLE conerr;

	// PID of the process.
	DWORD pid;

	// Exit status code of the process.
	DWORD exitstatus;

	// Attribute that indicates if the process is alive.
	uint8_t alive;
};
