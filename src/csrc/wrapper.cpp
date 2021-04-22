#include "wrapper.h"
#include <codecvt>
#include <locale>
#include <memory>
#include <unordered_map>


static std::unordered_map<std::string, Encoding> encoding_mapping = {
	{"utf8", Encoding::UTF8},
	{"utf-8", Encoding::UTF8},
	{"utf16", Encoding::UTF16},
	{"utf-16", Encoding::UTF16}
};

Encoding str_to_encoding(std::string enc_str) {
	Encoding enc = Encoding::UTF8;
	auto enc_search = encoding_mapping.find(enc_str);
	if (enc_search != encoding_mapping.end())
	{
		enc = enc_search->second;
	}
	return enc;
}

PTYRef create_pty(int cols, int rows, PTYConfig config) {
	//PTY* pty = ;
	std::string enc_str(config.encoding.data(), config.encoding.size());
	Encoding enc = str_to_encoding(enc_str);
	std::shared_ptr<PTY> shared_ptr(new PTY(cols, rows, config.input_mode, config.output_mode,
		config.override_pipes, config.mouse_mode, config.timeout, config.agent_config));
	std::cout << "Reference created?" << std::endl;
	return PTYRef{
		shared_ptr,
		enc
	};
}


PTYRef create_pty(int cols, int rows, int backend, PTYConfig config) {
	std::string enc_str(config.encoding.data(), config.encoding.size());
	Encoding enc = str_to_encoding(enc_str);
	std::shared_ptr<PTY> shared_ptr(new PTY(cols, rows, static_cast<Backend>(backend), config.input_mode, config.output_mode,
		config.override_pipes, config.mouse_mode, config.timeout, config.agent_config));
	return PTYRef{
		shared_ptr,
		enc
	};
}


std::wstring vec_to_wstr(rust::Vec<uint8_t> vec_in, Encoding enc) {
	//std::vector<uint8_t> vec;
	std::wstring wstr;
	if (vec_in.size() > 0) {
		vec_in.push_back(0);
		const char* ccp = reinterpret_cast<const char*>(vec_in.data());

		std::cout << "CCP: " << ccp << std::endl;
		if (enc == Encoding::UTF8) {
			std::wstring_convert<std::codecvt_utf8_utf16<wchar_t>> converter;
			wstr = converter.from_bytes(ccp);
		}
		else if (enc == Encoding::UTF16) {
			size_t len = mbstowcs(nullptr, &ccp[0], 0);

			wchar_t* wData = new wchar_t[len + 1];
			mbstowcs(&wData[0], &ccp[0], len);
			wstr = std::wstring(wData);
		}
	}

	/**
	std::u16string s(vec.data(), vec.data() + vec.size());

	std::wstring_convert<std::codecvt_utf16<wchar_t>, wchar_t> conv;
	std::wstring ws = conv.from_bytes(
		reinterpret_cast<const char*> (&s[0]),
		reinterpret_cast<const char*> (&s[0] + s.size()));**/

	return wstr;
}

rust::Vec<uint8_t> str_to_vec(std::wstring str, Encoding enc) {
	rust::Vec<uint8_t> vec;
	if (enc == Encoding::UTF8) {
		std::wstring_convert<std::codecvt_utf8_utf16<wchar_t>> converter;
		std::string enc_str = converter.to_bytes(str);
		const char* str_p = enc_str.c_str();
		for (int64_t i = 0; i < enc_str.length(); i++) {
			vec.push_back(str_p[i]);
		}
	}
	else {
		wchar_t* str_p = const_cast<wchar_t*>(str.c_str());
		unsigned char* str_uc = reinterpret_cast<unsigned char*>(str_p);

		size_t len = mbstowcs(&str_p[0], nullptr, 0);
		for (int64_t i = 0; i < len + 1; i++) {
			vec.push_back(str_uc[i]);
		}
	}
	return vec;
}

bool spawn(const PTYRef& pty_ref, rust::Vec<uint8_t> appname, rust::Vec<uint8_t> cmdline,
	rust::Vec<uint8_t> cwd, rust::Vec<uint8_t> env) {

	Encoding enc = static_cast<Encoding>(pty_ref.encoding);
	std::cout << "appname: " << appname.data() << std::endl;
	std::wstring app_wstr = vec_to_wstr(appname, enc);
	std::wstring cmdline_wstr = vec_to_wstr(cmdline, enc);
	std::wstring cwd_wstr = vec_to_wstr(cwd, enc);
	std::wstring env_wstr = vec_to_wstr(env, enc);
	
	auto pty_ptr = pty_ref.pty;
	PTY* pty = pty_ptr.get();

	bool value = pty->spawn(app_wstr, cmdline_wstr, cwd_wstr, env_wstr);
	//return false;
	std::cout << "Call: " << value << std::endl;
	return value;
}

void set_size(const PTYRef& pty_ref, int cols, int rows) {
	auto pty_ptr = pty_ref.pty;
	PTY pty = *pty_ptr.get();
	pty.set_size(cols, rows);
}

rust::Vec<uint8_t> read(const PTYRef& pty_ref, uint64_t length, bool blocking) {
	auto pty_ptr = pty_ref.pty;
	PTY* pty = pty_ptr.get();

	const DWORD BUFF_SIZE{ 512 };
	char szBuffer[BUFF_SIZE]{};
	uint32_t size = pty->read(szBuffer, BUFF_SIZE, blocking);
	
	rust::Vec<uint8_t> vec;
	for (int64_t i = 0; i < size; i++) {
		vec.push_back(szBuffer[i]);
	}

	//Encoding enc = static_cast<Encoding>(pty_ref.encoding);
	//rust::Vec<uint8_t> vec = str_to_vec(wstr, enc);
	/**std::u16string u16str(wstr.begin(), wstr.end());
	
	rust::Vec<uint16_t> out_buf;
	const char16_t* str_ptr = u16str.data();
	for (int64_t i = 0; i < u16str.length(); i++) {
		out_buf.push_back(str_ptr[i]);
	}**/
	return vec;
}

rust::Vec<uint8_t> read_stderr(const PTYRef& pty_ref, uint64_t length, bool blocking) {
	auto pty_ptr = pty_ref.pty;
	PTY* pty = pty_ptr.get();

	const DWORD BUFF_SIZE{ 512 };
	char szBuffer[BUFF_SIZE]{};
	uint32_t size = pty->read_stderr(szBuffer, BUFF_SIZE, blocking);

	rust::Vec<uint8_t> vec;
	for (int64_t i = 0; i < size; i++) {
		vec.push_back(szBuffer[i]);
	}
	return vec;
}


uint32_t write(const PTYRef& pty_ref, rust::Vec<uint8_t> in_str) {
	Encoding enc = static_cast<Encoding>(pty_ref.encoding);
	std::wstring wstr = vec_to_wstr(in_str, enc);

	auto pty_ptr = pty_ref.pty;
	PTY* pty = pty_ptr.get();

	bool ret;
	DWORD num_bytes;
	std::tie(ret, num_bytes) = pty->write(wstr);
	return num_bytes;
}


bool is_alive(PTYRef pty_ref) {
	auto pty_ptr = pty_ref.pty;
	PTY* pty = pty_ptr.get();
	return pty->is_alive();
}


int64_t get_exitstatus(PTYRef pty_ref) {
	auto pty_ptr = pty_ref.pty;
	PTY* pty = pty_ptr.get();
	return pty->get_exitstatus();
}
