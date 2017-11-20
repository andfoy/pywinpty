
"""
Cython wrapper around Winpty and Windows functions.
"""

cimport cython
from winpty._winpty cimport winpty, winpty_constants

# Windows API types
cdef extern from "Windows.h":
    ctypedef Py_UNICODE WCHAR
    ctypedef const WCHAR* LPCWSTR
    ctypedef void* HWND
    ctypedef void* HANDLE
    ctypedef unsigned long long UINT64
    ctypedef unsigned long DWORD
    ctypedef DWORD *LPDWORD
    ctypedef LPCWSTR LPCTSTR

    DEF STILL_ACTIVE = 259

    DWORD GetProcessId(HANDLE proc)
    bint GetExitCodeProcess(
      HANDLE  hProcess,
      LPDWORD lpExitCode
    )


cdef class Agent:
    """
    This class wraps a winpty agent and offers communication with
    a winpty-agent process.
    """
    cdef winpty.winpty_t* _c_winpty_t
    cdef HANDLE _agent_process
    cdef HANDLE _process
    cdef DWORD pid
    cdef LPCWSTR conin_pipe_name
    cdef LPCWSTR conout_pipe_name
    cdef DWORD exitstatus
    cdef unsigned char alive

    def __init__(self, int cols, int rows, bint override_pipes=True,
                 int mouse_mode=winpty_constants._WINPTY_MOUSE_MODE_NONE,
                 int timeout=30000, int agent_config=winpty_constants._WINPTY_FLAG_COLOR_ESCAPES):
        """
        Initialize a winpty agent wrapper of size ``(cols, rows)``.
        """
        cdef winpty.winpty_error_ptr_t err
        cdef winpty.winpty_config_t* config = winpty.winpty_config_new(agent_config, &err)

        if config is NULL:
            msg = 'An error has ocurred: {0} - Code: {1}'.format(
                winpty.winpty_error_msg(err),
                winpty.winpty_error_code(err))
            winpty.winpty_error_free(err)
            raise RuntimeError(msg)

        if cols <= 0 or rows <= 0:
            msg = 'PTY cols and rows must be positive and non-zero. Got: ({0}, {1})'.format(
                cols, rows)
            raise RuntimeError(msg)

        winpty.winpty_config_set_initial_size(config, cols, rows)
        winpty.winpty_config_set_mouse_mode(config, mouse_mode)
        winpty.winpty_config_set_agent_timeout(config, timeout)

        cdef winpty.winpty_error_ptr_t err_pointer
        self._c_winpty_t = winpty.winpty_open(config, &err_pointer)
        winpty.winpty_config_free(config)

        if self._c_winpty_t is NULL:
            msg = 'An error has ocurred: {0} - Code: {1}'.format(
                winpty.winpty_error_msg(err_pointer),
                winpty.winpty_error_code(err_pointer))
            winpty.winpty_error_free(err_pointer)
            raise RuntimeError(msg)

        self._agent_process = winpty.winpty_agent_process(self._c_winpty_t)
        self.conin_pipe_name = winpty.winpty_conin_name(self._c_winpty_t)
        self.conout_pipe_name = winpty.winpty_conout_name(self._c_winpty_t)
        self.alive = 0
        self.pid = 0

    property conin_pipe_name:
        def __get__(self):
            return self.conin_pipe_name

    property conout_pipe_name:
        def __get__(self):
            return self.conout_pipe_name

    property pid:
        def __get__(self):
            return self.pid

    property exitstatus:
        def __get__(self):
            if self.pid == 0:
                return None
            if self.alive == 1:
                self.isalive()
            if self.alive == 1:
                return None
            return self.exitstatus

    def spawn(self, LPCWSTR appname, LPCWSTR cmdline=NULL,
              LPCWSTR cwd=NULL, LPCWSTR env=NULL):
        """
        Start a windows process that communicates through a winpty-agent process.
        """
        cdef winpty.winpty_error_ptr_t spawn_conf_err
        cdef winpty.winpty_spawn_config_t* spawn_config = winpty.winpty_spawn_config_new(winpty_constants._WINPTY_SPAWN_FLAG_MASK,
                                                                                         appname, cmdline, cwd, env, &spawn_conf_err)
        if spawn_config is NULL:
            msg = 'An error has ocurred: {0} - Code: {1}'.format(
                winpty.winpty_error_msg(spawn_conf_err),
                winpty.winpty_error_code(spawn_conf_err))
            winpty.winpty_error_free(spawn_conf_err)
            raise RuntimeError(msg)

        cdef winpty.winpty_error_ptr_t spawn_err
        cdef bint succ = winpty.winpty_spawn(self._c_winpty_t, spawn_config, &self._process,
                                             NULL, NULL, &spawn_err)
        winpty.winpty_spawn_config_free(spawn_config)

        if not succ:
            msg = 'An error has ocurred: {0} - Code: {1}'.format(
                winpty.winpty_error_msg(spawn_err),
                winpty.winpty_error_code(spawn_err))
            winpty.winpty_error_free(spawn_err)
            raise RuntimeError(msg)

        self.pid = GetProcessId(self._process)
        self.alive = 1

        return succ

    def set_size(self, int cols, int rows):
        """
        Resize the console size of current agent process.
        """
        if cols <= 0 or rows <= 0:
            msg = 'PTY cols and rows must be positive and non-zero. Got: ({0}, {1})'.format(
                cols, rows)
            raise RuntimeError(msg)

        cdef winpty.winpty_error_ptr_t err_pointer
        cdef bint succ = winpty.winpty_set_size(self._c_winpty_t, cols, rows, &err_pointer)

        if not succ:
            msg = 'An error has ocurred: {0} - Code: {1}'.format(
                winpty.winpty_error_msg(err_pointer),
                winpty.winpty_error_code(err_pointer))
            winpty.winpty_error_free(err_pointer)
            raise RuntimeError(msg)

    def isalive(self):
        """This tests if the pty process is running or not.
        This is non-blocking.
        """
        cdef DWORD lpExitCode
        cdef bint succ = GetExitCodeProcess(self._process, &lpExitCode)
        if not succ:
            raise RuntimeError('Could not check status')

        # Check for STILL_ACTIVE flag
        # https://msdn.microsoft.com/en-us/library/windows/desktop/ms683189(v=vs.85).aspx
        alive = lpExitCode == STILL_ACTIVE
        if not alive:
            self.alive = 0
            self.exitstatus = lpExitCode
        return alive

    def __dealloc__(self):
        if self._c_winpty_t is not NULL:
            winpty.winpty_free(self._c_winpty_t)
