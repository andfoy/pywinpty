# -*- coding: utf-8 -*-
"""Wrap process I/O pipe communication using pywin32."""

# yapf: disable

# Third party imports
import ctypes
from ctypes import windll
from winpty.cywinpty import Agent

# yapf: enable

OPEN_EXISTING = 3
GENERIC_WRITE = 0x40000000
GENERIC_READ = 0x80000000

DWORD = ctypes.c_ulong
LARGE_INTEGER = ctypes.c_longlong
PLARGE_INTEGER = ctypes.POINTER(LARGE_INTEGER)


class PTY(Agent):
    """
    This class provides a pywin32 communication wrapper around winpty process
    communication pipes.

    Inherits all Cython winpty agent functionality and properties.
    """

    def __init__(self, cols, rows):
        """Initialize a new Pseudo Terminal of size ``(cols, rows)``."""
        Agent.__init__(self, cols, rows, True)
        self.conin_pipe = windll.kernel32.CreateFileW(
            self.conin_pipe_name, GENERIC_WRITE, 0, None,
            OPEN_EXISTING, 0, None)
        self.conout_pipe = windll.kernel32.CreateFileW(
            self.conout_pipe_name, GENERIC_READ, 0, None,
            OPEN_EXISTING, 0, None
        )

    def read(self, length=1000):
        """
        Read ``length`` characters from current process output stream.

        Note: This method is not fully non-blocking, however it
        behaves like one.
        """
        size_p = PLARGE_INTEGER(LARGE_INTEGER(0))
        windll.kernel32.GetFileSizeEx(self.conout_pipe, size_p)
        size = size_p[0]
        length = min(size, length)
        data = ctypes.create_string_buffer(b'\000' * length)
        if length > 0:
            windll.kernel32.ReadFile(self.conout_pipe, data, length,
                                     None, None)
        return data.value

    def write(self, data):
        """Write data to current process input stream."""
        data = bytes(data, 'utf-8')
        data_p = ctypes.create_string_buffer(data)
        num_bytes = PLARGE_INTEGER(LARGE_INTEGER(0))
        bytes_to_write = ctypes.sizeof(data_p)
        err = windll.kernel32.WriteFile(self.conin_pipe, data_p,
                                        bytes_to_write, num_bytes, None)
        return err, num_bytes[0]

    def close(self):
        """Close all communication process streams."""
        windll.kernel32.CloseHandle(self.conout_pipe)
        windll.kernel32.CloseHandle(self.conin_pipe)

    def isalive(self):
        """Check if current process streams are still open."""
        alive = True
        err, _ = self.write('')
        alive = err == 0
        return alive
