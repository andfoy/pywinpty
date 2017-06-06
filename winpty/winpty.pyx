
cimport cython
# from libcpp.vector cimport vector
# from libcpp.string cimport string
from libc.stdlib cimport malloc, free, calloc
# from libcpp.cast import reinterpret_cast
from winpty._winpty cimport winpty, winpty_constants

# cdef enum AgentConstants:
#    WINPTY_FLAG_CONERR = winpty_constants._WINPTY_FLAG_CONERR
#    WINPTY_FLAG_PLAIN_OUTPUT = winpty_constants._WINPTY_FLAG_PLAIN_OUTPUT
#    WINPTY_FLAG_COLOR_ESCAPES = winpty_constants._WINPTY_FLAG_COLOR_ESCAPES
#    WINPTY_FLAG_ALLOW_CURPROC_DESKTOP_CREATION = winpty_constants._WINPTY_FLAG_ALLOW_CURPROC_DESKTOP_CREATION

cdef extern from "Windows.h":
    ctypedef Py_UNICODE WCHAR
    ctypedef const WCHAR* LPCWSTR
    ctypedef void* HWND
    ctypedef void* HANDLE
    ctypedef unsigned long long UINT64
    ctypedef unsigned long DWORD
    ctypedef DWORD *LPDWORD
    ctypedef struct SECURITY_ATTRIBUTES
    ctypedef LPCWSTR LPCTSTR
    ctypedef SECURITY_ATTRIBUTES* LPSECURITY_ATTRIBUTES
    ctypedef struct OVERLAPPED:
        pass
    ctypedef OVERLAPPED* LPOVERLAPPED
    ctypedef void *LPVOID
    ctypedef const void* LPCVOID

    ctypedef struct COMMTIMEOUTS:
        DWORD ReadIntervalTimeout
        DWORD ReadTotalTimeoutMultiplier
        DWORD ReadTotalTimeoutConstant
        DWORD WriteTotalTimeoutMultiplier
        DWORD WriteTotalTimeoutConstant

    # ctypedef struct COMMTIMEOUTS
    ctypedef COMMTIMEOUTS* LPCOMMTIMEOUTS
    # ctypedef __stdcall CALLBACK
    ctypedef void (*LPOVERLAPPED_COMPLETION_ROUTINE) (DWORD, DWORD, LPVOID)


    HANDLE CreateFileW(LPCTSTR lpFileName, DWORD dwDesiredAccess, DWORD dwShareMode,
                       LPSECURITY_ATTRIBUTES lpSecurityAttributes, DWORD dwCreationDisposition,
                       DWORD dwFlagsAndAttributes, HANDLE hTemplateFile)

    bint ReadFile(HANDLE hFile, LPVOID lpBuffer, DWORD nNumberOfBytesToRead,
                  LPDWORD lpNumberOfBytesRead, LPOVERLAPPED lpOverlapped)

    bint ReadFileEx(HANDLE hFile, LPVOID lpBuffer, DWORD nNumberOfBytesToRead,
                    LPOVERLAPPED lpNumberOfBytesRead, LPOVERLAPPED_COMPLETION_ROUTINE lpOverlapped)

    bint WriteFile(HANDLE hfile, LPCVOID lpBuffer, DWORD nNumberOfBytesToWrite,
                   LPDWORD lpNumberOfBytesWritten, LPOVERLAPPED lpOverlapped)

    bint CloseHandle(HANDLE hObject)
    bint SetCommTimeouts(HANDLE hFile, LPCOMMTIMEOUTS lpCommTimeouts)

    DWORD GetLastError()
    DWORD SleepEx(DWORD dwMilliseconds, bint bAlertable)

    cdef int GENERIC_WRITE
    cdef int GENERIC_READ
    cdef int OPEN_EXISTING
    cdef int FILE_FLAG_OVERLAPPED
    cdef int WAIT_IO_COMPLETION


ctypedef unsigned char UCHAR

ctypedef struct OVLP:
    OVERLAPPED readOvlp
    UCHAR buf[1024]

cdef void callback(DWORD err, DWORD bytes, LPVOID ovlp):
    cdef OVLP* temp = <OVLP*> ovlp

cdef class Agent:
    cdef winpty.winpty_t* _c_winpty_t
    cdef HANDLE _agent_process
    cdef HANDLE _conin_pipe
    cdef HANDLE _conout_pipe
    cdef HANDLE _conerr_pipe

    def __cinit__(self, int cols, int rows,
                  int mouse_mode=winpty_constants._WINPTY_MOUSE_MODE_AUTO,
                  int timeout=3000, int agent_config=winpty_constants._WINPTY_FLAG_MASK):
        cdef winpty.winpty_error_ptr_t* err_pointer = NULL
        cdef winpty.winpty_config_t* config = winpty.winpty_config_new(agent_config, err_pointer)

        if config is NULL:
            raise MemoryError(winpty.winpty_error_msg(err_pointer[0]))

        if err_pointer is not NULL:
            msg = 'An error has ocurred: {0} - Code: {1}'.format(
                winpty.winpty_error_msg(err_pointer[0]),
                winpty.winpty_error_code(err_pointer[0]))
            winpty.winpty_error_free(err_pointer[0])
            raise RuntimeError(msg)

        winpty.winpty_config_set_initial_size(config, cols, rows)
        winpty.winpty_config_set_mouse_mode(config, mouse_mode)
        winpty.winpty_config_set_agent_timeout(config, timeout)

        err_pointer = NULL
        self._c_winpty_t = winpty.winpty_open(config, err_pointer)
        winpty.winpty_config_free(config)

        if err_pointer is not NULL:
            msg = 'An error has ocurred: {0} - Code: {1}'.format(
                winpty.winpty_error_msg(err_pointer[0]),
                winpty.winpty_error_code(err_pointer[0]))
            winpty.winpty_error_free(err_pointer[0])
            raise RuntimeError(msg)

        self._agent_process = winpty.winpty_agent_process(self._c_winpty_t)
        conin_pipe_name = winpty.winpty_conin_name(self._c_winpty_t)
        conout_pipe_name = winpty.winpty_conout_name(self._c_winpty_t)
        conerr_pipe_name = winpty.winpty_conerr_name(self._c_winpty_t)

        self._conin_pipe = CreateFileW(conin_pipe_name, GENERIC_WRITE,
                                       0, NULL, OPEN_EXISTING, 0, NULL)
        self._conout_pipe = CreateFileW(conout_pipe_name, GENERIC_READ,
                                       0, NULL, OPEN_EXISTING, FILE_FLAG_OVERLAPPED, NULL)


    def spawn(self, LPCWSTR appname, LPCWSTR cmdline=NULL,
              LPCWSTR cwd=NULL, LPCWSTR env=NULL):
        cdef winpty.winpty_spawn_config_t* spawn_config = NULL
        cdef winpty.winpty_error_ptr_t* spawn_conf_err = NULL
        spawn_config = winpty.winpty_spawn_config_new(winpty_constants._WINPTY_SPAWN_FLAG_MASK,
                                                      appname, cmdline, cwd, env, spawn_conf_err)
        if spawn_conf_err is not NULL:
            msg = 'An error has ocurred: {0} - Code: {1}'.format(
                winpty.winpty_error_msg(spawn_conf_err[0]),
                winpty.winpty_error_code(spawn_conf_err[0]))
            winpty.winpty_error_free(spawn_conf_err[0])
            raise RuntimeError(msg)

        cdef winpty.winpty_error_ptr_t* spawn_err = NULL
        cdef bint succ = winpty.winpty_spawn(self._c_winpty_t, spawn_config, NULL,
                                             NULL, NULL, spawn_err)

        winpty.winpty_spawn_config_free(spawn_config)

        return succ
        # if not succ:
        #     msg = 'An error has ocurred: {0} - Code: {1}'.format(
        #         winpty.winpty_error_msg(spawn_err[0]),
        #         winpty.winpty_error_code(spawn_err[0]))
        #     winpty.winpty_error_free(spawn_err[0])
        #     raise RuntimeError(msg)

    def set_size(self, int cols, int rows):
        cdef winpty.winpty_error_ptr_t* err_pointer = NULL
        cdef bint succ = winpty.winpty_set_size(self._c_winpty_t, cols, rows, err_pointer)

        if not succ:
            msg = 'An error has ocurred: {0} - Code: {1}'.format(
                winpty.winpty_error_msg(err_pointer[0]),
                winpty.winpty_error_code(err_pointer[0]))
            winpty.winpty_error_free(err_pointer[0])
            raise RuntimeError(msg)

    def read_blocking(self, DWORD length=1000):
        cdef unsigned char buf[1024]
        cdef bint ret = False

        ret = ReadFile(self._conout_pipe, buf, sizeof(buf),
                       &length, NULL)
        return buf

    def read(self, DWORD length=1000, DWORD timeout=1000):
        cdef OVLP ovlp_read
        cdef bint ret = ReadFileEx(self._conout_pipe, ovlp_read.buf, sizeof(ovlp_read.buf),
                                   <LPOVERLAPPED>(&ovlp_read), callback)
        cdef DWORD status = SleepEx(timeout, True)
        cdef UCHAR* lines = ''
        if status == WAIT_IO_COMPLETION:
            lines = ovlp_read.buf
        return lines


    def __dealloc__(self):
        if self._c_winpty_t is not NULL:
            winpty.winpty_free(self._c_winpty_t)



