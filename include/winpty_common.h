#define NOMINMAX

#include <Windows.h>
#include <stdint.h>
#include <string>

#undef min
#undef max


//#ifdef ENABLE_WINPTY
extern "C" {
	#include <winpty.h>
	#include <winpty_constants.h>
}

struct WinptyPTY {
	// Main constructor
	WinptyPTY(int cols, int rows, bool override_pipes = true, int mouse_mode = WINPTY_MOUSE_MODE_NONE,
		int timeout = 30000, int agent_config = WINPTY_FLAG_COLOR_ESCAPES);

	// Destructor
	~WinptyPTY();

	bool spawn(std::wstring appname, std::wstring cmdline = NULL,
		       std::wstring cwd = NULL, std::wstring env = NULL);
	
	void set_size(int cols, int rows);

	bool is_alive();

	std::wstring read(uint64_t length = 1000, bool blocking = false);

	std::pair<bool, DWORD> write(std::wstring str);

	bool is_eof();

	int64_t get_exitstatus();

	winpty_t* pty_ref;
	HANDLE agent_process;
	HANDLE process;
	HANDLE conin;
	HANDLE conout;
	HANDLE conerr;
	DWORD pid;
	LPCWSTR conin_pipe_name;
	LPCWSTR conout_pipe_name;
	LPCWSTR conerr_pipe_name;
	DWORD exitstatus;
	uint8_t alive;
};
//#endif

//struct PTY {
//	PTY()
//};