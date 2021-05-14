#include "base.h"

uint32_t base_read(char* szBuffer, uint64_t length, bool blocking, HANDLE stream, bool using_pipes) {
    LARGE_INTEGER size_p;
    if (!blocking) {
        if (!using_pipes) {
            HRESULT hr = GetFileSizeEx(stream, &size_p) ? S_OK : GetLastError();
            if (S_OK != hr) {
                throw_runtime_error(hr);
            }
            LONGLONG expected_length = size_p.QuadPart;
            length = std::min(static_cast<uint64_t>(expected_length), length);
        }
        else {
            DWORD available_bytes;
            HRESULT hr = PeekNamedPipe(stream, NULL, 0, NULL, &available_bytes, NULL) ? S_OK : GetLastError();
            if (S_OK != hr) {
                throw_runtime_error(hr);
            }
            length = std::min(static_cast<uint64_t>(available_bytes), length);

        }
    }

    DWORD dwBytesRead{};

    if (length > 0) {
        HRESULT hr = ReadFile(stream, szBuffer, length, &dwBytesRead, NULL) ? S_OK : GetLastError();
        if (S_OK != hr) {
            throw_runtime_error(hr);
        }
    }
    return dwBytesRead;
}

uint32_t BaseProcess::read(char* buf, uint64_t length, bool blocking) {
    return base_read(buf, length, blocking, conout, using_pipes);
}

uint32_t BaseProcess::read_stderr(char* buf, uint64_t length, bool blocking) {
    return base_read(buf, length, blocking, conerr, using_pipes);
}

std::pair<bool, DWORD> BaseProcess::write(const char* str, size_t length) {
    DWORD num_bytes;
    bool success = WriteFile(conin, str, length, &num_bytes, NULL);
    return std::make_pair(success, num_bytes);
}

bool BaseProcess::is_eof() {
    DWORD available_bytes;
    bool succ = PeekNamedPipe(conout, NULL, 0, NULL, &available_bytes, NULL);
    if (succ) {
        if (available_bytes == 0 && !is_alive()) {
            succ = false;
        }
    }
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

