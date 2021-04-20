#include "wrapper.h"
#include <codecvt>
#include <locale>

PTYRef create_pty(int cols, int rows) {
	PTY pty(cols, rows);
	std::shared_ptr<PTY> shared_ptr = std::make_shared<PTY>(std::move(pty));
	return PTYRef{
		shared_ptr
	};
}


PTYRef create_pty(int cols, int rows, int backend) {
	PTY pty(cols, rows, static_cast<Backend>(backend));
	std::shared_ptr<PTY> shared_ptr = std::make_shared<PTY>(std::move(pty));
	return PTYRef{
		shared_ptr
	};
}


PTYRef create_pty(int cols, int rows, PTYConfig config) {
	PTY pty(cols, rows, config.input_mode, config.output_mode,
		config.override_pipes, config.mouse_mode, config.timeout, config.agent_config);
	std::shared_ptr<PTY> shared_ptr = std::make_shared<PTY>(std::move(pty));
	return PTYRef{
		shared_ptr
	};
}


PTYRef create_pty(int cols, int rows, int backend, PTYConfig config) {
	PTY pty(cols, rows, static_cast<Backend>(backend), config.input_mode, config.output_mode,
		config.override_pipes, config.mouse_mode, config.timeout, config.agent_config);
	std::shared_ptr<PTY> shared_ptr = std::make_shared<PTY>(std::move(pty));
	return PTYRef{
		shared_ptr
	};
}


std::wstring vec_to_wstr(rust::Vec<uint16_t> vec_in) {
	std::vector<uint16_t> vec;
	for (uint16_t value : vec_in) {
		vec.push_back(value);
	}

	std::u16string s(vec.data(), vec.data() + vec.size());

	std::wstring_convert<std::codecvt_utf16<wchar_t>, wchar_t> conv;
	std::wstring ws = conv.from_bytes(
		reinterpret_cast<const char*> (&s[0]),
		reinterpret_cast<const char*> (&s[0] + s.size()));

	return ws;
}

bool spawn(PTYRef pty_ref, rust::Vec<uint16_t> appname, rust::Vec<uint16_t> cmdline,
	rust::Vec<uint16_t> cwd, rust::Vec<uint16_t> env) {

	std::wstring app_wstr = vec_to_wstr(appname);
	std::wstring cmdline_wstr = vec_to_wstr(cmdline);
	std::wstring cwd_wstr = vec_to_wstr(cwd);
	std::wstring env_wstr = vec_to_wstr(env);
	
	auto pty_ptr = pty_ref.pty;
	PTY pty = *pty_ptr.get();

	//return false;
	return pty.spawn(app_wstr, cmdline_wstr, cwd_wstr, env_wstr);
}

void set_size(PTYRef pty_ref, int cols, int rows) {
	auto pty_ptr = pty_ref.pty;
	PTY pty = *pty_ptr.get();
	pty.set_size(cols, rows);
}

rust::Vec<uint16_t> read(PTYRef pty_ref, uint64_t length, bool blocking) {
	auto pty_ptr = pty_ref.pty;
	PTY pty = *pty_ptr.get();
	std::wstring wstr = pty.read(length, blocking);
	std::u16string u16str(wstr.begin(), wstr.end());
	
	rust::Vec<uint16_t> out_buf;
	const char16_t* str_ptr = u16str.data();
	for (int64_t i = 0; i < u16str.length(); i++) {
		out_buf.push_back(str_ptr[i]);
	}
	return out_buf;
}

rust::Vec<uint16_t> read_stderr(PTYRef pty_ref, uint64_t length, bool blocking) {
	auto pty_ptr = pty_ref.pty;
	PTY pty = *pty_ptr.get();
	std::wstring wstr = pty.read_stderr(length, blocking);
	std::u16string u16str(wstr.begin(), wstr.end());

	rust::Vec<uint16_t> out_buf;
	const char16_t* str_ptr = u16str.data();
	for (int64_t i = 0; i < u16str.length(); i++) {
		out_buf.push_back(str_ptr[i]);
	}
	return out_buf;
}


uint32_t write(PTYRef pty_ref, rust::Vec<uint16_t> in_str) {
	std::wstring wstr = vec_to_wstr(in_str);

	auto pty_ptr = pty_ref.pty;
	PTY pty = *pty_ptr.get();

	bool ret;
	DWORD num_bytes;
	std::tie(ret, num_bytes) = pty.write(wstr);
	return num_bytes;
}


bool is_alive(PTYRef pty_ref) {
	auto pty_ptr = pty_ref.pty;
	PTY pty = *pty_ptr.get();
	return pty.is_alive();
}


int64_t get_exitstatus(PTYRef pty_ref) {
	auto pty_ptr = pty_ref.pty;
	PTY pty = *pty_ptr.get();
	return pty.get_exitstatus();
}
