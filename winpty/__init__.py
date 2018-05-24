# -*- coding: utf-8 -*-
"""
Pywinpty
========
This package provides a Cython wrapper around winpty C++ library.
"""

# yapf: disable

# Local imports
from .ptyprocess import PtyProcess
from .winpty_wrapper import PTY


PTY
PtyProcess
VERSION_INFO = (0, 5, 2)
__version__ = '.'.join(map(str, VERSION_INFO))


# Test that spawing a process is working or raise
# an ImportError.
# Fixes issue 59
for _ in range(10): # Sometimes it doesn't fail the first time
    valid = False
    try:
        proc = PtyProcess.spawn('python')
        proc.write('print("hello, world!")\r\n')
        proc.write('exit()\r\n')
        valid = True
    except Exception:
        break

if not valid:
    raise ImportError('Cannot successfully read from pty, see '
                      'https://github.com/spyder-ide/pywinpty/issues/59')

del proc, valid
