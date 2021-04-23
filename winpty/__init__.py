# -*- coding: utf-8 -*-
"""
Pywinpty
========
This package provides low and high level APIs to create
pseudo terminals in Windows.
"""

# Local imports
from .winpty import PTY, WinptyError
from .ptyprocess import PtyProcess
from .enums import Backend, Encoding, MouseMode, AgentConfig


PTY
PtyProcess
Backend
Encoding
MouseMode
AgentConfig
WinptyError

VERSION_INFO = (0, 6, 0, 'dev0')
__version__ = '.'.join(map(str, VERSION_INFO))

