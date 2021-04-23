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
    if (enc_search != encoding_mapping.end()) {
        enc = enc_search->second;
    }
    return enc;
}

PTYRef create_pty(int cols, int rows, PTYConfig config) {
    std::string enc_str(config.encoding.data(), config.encoding.size());
    Encoding enc = str_to_encoding(enc_str);
    std::shared_ptr<PTY> shared_ptr(new PTY(
        cols, rows, config.mouse_mode, config.timeout, config.agent_config));
    return PTYRef{
        shared_ptr,
        enc
    };
}


PTYRef create_pty(int cols, int rows, int backend, PTYConfig config) {
    std::string enc_str(config.encoding.data(), config.encoding.size());
    Encoding enc = str_to_encoding(enc_str);
    std::shared_ptr<PTY> shared_ptr(new PTY(cols, rows, static_cast<Backend>(backend),
        config.mouse_mode, config.timeout, config.agent_config));
    return PTYRef{
        shared_ptr,
        enc
    };
}


std::wstring vec_to_wstr(rust::Vec<uint8_t> vec_in, Encoding enc) {
    std::wstring wstr;
    if (vec_in.size() > 0) {
        if (vec_in.back() != 0) {
            vec_in.push_back(0);
        }
        uint8_t* up = vec_in.data();
        const char* ccp = reinterpret_cast<const char*>(up);

        if (enc == Encoding::UTF8) {
            std::wstring_convert<std::codecvt_utf8_utf16<wchar_t>> converter;
            wstr = converter.from_bytes(ccp, ccp + vec_in.size());
        }
        else if (enc == Encoding::UTF16) {
            size_t len = mbstowcs(nullptr, &ccp[0], 0);

            wchar_t* wData = new wchar_t[len + 1];
            mbstowcs(&wData[0], &ccp[0], len);
            wstr = std::wstring(wData);
        }
    }

    return wstr;
}

bool spawn(const PTYRef& pty_ref, rust::Vec<uint8_t> appname, rust::Vec<uint8_t> cmdline,
    rust::Vec<uint8_t> cwd, rust::Vec<uint8_t> env) {

    Encoding enc = static_cast<Encoding>(pty_ref.encoding);
    std::wstring app_wstr = vec_to_wstr(appname, enc);
    std::wstring cmdline_wstr = vec_to_wstr(cmdline, enc);
    std::wstring cwd_wstr = vec_to_wstr(cwd, enc);
    std::wstring env_wstr = vec_to_wstr(env, enc);

    auto pty_ptr = pty_ref.pty;
    PTY* pty = pty_ptr.get();

    bool value = pty->spawn(app_wstr, cmdline_wstr, cwd_wstr, env_wstr);
    return value;
}

void set_size(const PTYRef& pty_ref, int cols, int rows) {
    auto pty_ptr = pty_ref.pty;
    PTY* pty = pty_ptr.get();
    pty->set_size(cols, rows);
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
    return vec;
}

rust::Vec<uint8_t> read_stderr(const PTYRef& pty_ref, uint64_t length, bool blocking) {
    auto pty_ptr = pty_ref.pty;
    PTY* pty = pty_ptr.get();

    char* szBuffer = new char[length];
    uint32_t size = pty->read_stderr(szBuffer, length, blocking);

    rust::Vec<uint8_t> vec;
    for (int64_t i = 0; i < size; i++) {
        vec.push_back(szBuffer[i]);
    }
    return vec;
}


uint32_t write(const PTYRef& pty_ref, rust::Vec<uint8_t> in_str) {
    Encoding enc = static_cast<Encoding>(pty_ref.encoding);
    const char* ccp = reinterpret_cast<const char*>(in_str.data());
    size_t length = in_str.size();

    auto pty_ptr = pty_ref.pty;
    PTY* pty = pty_ptr.get();

    bool ret;
    DWORD num_bytes;
    std::tie(ret, num_bytes) = pty->write(ccp, length);
    return num_bytes;
}


bool is_alive(const PTYRef& pty_ref) {
    auto pty_ptr = pty_ref.pty;
    PTY* pty = pty_ptr.get();
    return pty->is_alive();
}


int64_t get_exitstatus(const PTYRef& pty_ref) {
    auto pty_ptr = pty_ref.pty;
    PTY* pty = pty_ptr.get();
    return pty->get_exitstatus();
}


bool is_eof(const PTYRef& pty_ref) {
    auto pty_ptr = pty_ref.pty;
    PTY* pty = pty_ptr.get();
    return pty->is_eof();
}


uint32_t pid(const PTYRef& pty_ref) {
    auto pty_ptr = pty_ref.pty;
    PTY* pty = pty_ptr.get();
    return pty->pid();
}
