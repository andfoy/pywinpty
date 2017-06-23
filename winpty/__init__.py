# -*- coding: utf-8 -*-
"""
Pywinpty
========
This package provides a Cython wrapper around winpty C++ library.
"""

try:
    from .winpty_wrapper import PTY
    PTY
except ImportError:
    pass

VERSION_INFO = (0, 1, 1, 'dev0')
__version__ = '.'.join(map(str, VERSION_INFO))
