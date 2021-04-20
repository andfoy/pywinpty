#pragma once

#include "base.h"
#include <cstdio>
#include <tchar.h>
#include <process.h>

// Handle to a ConPTY instance.
struct ConPTY: BaseProcess {
	// Main constructor
	ConPTY(int cols, int rows, int input_mode = ENABLE_VIRTUAL_TERMINAL_INPUT,
		   int output_mode = ENABLE_VIRTUAL_TERMINAL_PROCESSING);

	// Main destructor
	~ConPTY();

	// Spawn the process that the PTY will communicate to
	bool spawn(std::wstring appname, std::wstring cmdline = NULL,
			   std::wstring cwd = NULL, std::wstring env = NULL);

	void set_size(int cols, int rows);

	std::wstring read_stderr(uint64_t length, bool blocking);

	bool pty_created;
	bool pty_started;
	HPCON pty_handle;
	PROCESS_INFORMATION process_info;
	STARTUPINFOEXW startup_info;
};
