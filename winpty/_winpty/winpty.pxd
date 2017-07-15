
"""
Import winpty.h struct and function definitions.
"""

cimport cython

# Import some windows.h type shorthands
cdef extern from "Windows.h":
    ctypedef Py_UNICODE WCHAR
    ctypedef const WCHAR* LPCWSTR
    ctypedef void* HWND
    ctypedef void* HANDLE
    ctypedef unsigned long long UINT64
    ctypedef unsigned long DWORD


# Import winpty.h C++ definitions
cdef extern from 'winpty.h':
    # Error constants and structs
    ctypedef struct winpty_error_t
    ctypedef winpty_error_t* winpty_error_ptr_t
    ctypedef DWORD winpty_result_t

    # Error related functions
    winpty_result_t winpty_error_code(winpty_error_ptr_t err)
    LPCWSTR winpty_error_msg(winpty_error_ptr_t err)
    void winpty_error_free(winpty_error_ptr_t err)

    # Winpty agent configuration settings
    # ctypedef struct winpty_config_s winpty_config_t
    ctypedef struct winpty_config_t

    # Winpty agent configuration functions
    winpty_config_t* winpty_config_new(UINT64 agentFlags, winpty_error_ptr_t *err)
    void winpty_config_free(winpty_config_t *cfg)
    void winpty_config_set_initial_size(winpty_config_t *cfg, int cols, int rows)
    void winpty_config_set_mouse_mode(winpty_config_t *cfg, int mouseMode)
    void winpty_config_set_agent_timeout(winpty_config_t *cfg, DWORD timeoutMs)

    # Agent start related structs
    # ctypedef struct winpty_s winpty_t
    ctypedef struct winpty_t

    # Start a new agent
    winpty_t * winpty_open(const winpty_config_t *cfg, winpty_error_ptr_t *err)
    HANDLE winpty_agent_process(winpty_t *wp)

    # I/O Pipes
    LPCWSTR winpty_conin_name(winpty_t *wp)
    LPCWSTR winpty_conout_name(winpty_t *wp)
    LPCWSTR winpty_conerr_name(winpty_t *wp)


    # Process creation configuration struct
    ctypedef struct winpty_spawn_config_t

    # Process creation functions
    winpty_spawn_config_t * winpty_spawn_config_new(UINT64 spawnFlags,
                                                    LPCWSTR appname,
                                                    LPCWSTR cmdline,
                                                    LPCWSTR cwd,
                                                    LPCWSTR env,
                                                    winpty_error_ptr_t *err)

    void winpty_spawn_config_free(winpty_spawn_config_t *cfg)

    # Spawn new processes
    bint winpty_spawn(winpty_t *wp,
                      const winpty_spawn_config_t *cfg,
                      HANDLE *process_handle,
                      HANDLE *thread_handle,
                      DWORD *create_process_error,
                      winpty_error_ptr_t *err)

    # Winpty agent RPC calls
    bint winpty_set_size(winpty_t *wp, int cols, int rows,
                         winpty_error_ptr_t *err)

    # Free winpty_t pointer
    void winpty_free(winpty_t *wp)
