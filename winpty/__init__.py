# -*- coding: utf-8 -*-

"""
Pywinpty
========
This package provides a Cython wrapper around winpty C++ library.
"""
try:
    from .cywinpty import Agent
    from .winpty_wrapper import PTY
except ImportError:
    pass

VERSION_INFO = (0, 1, 0)
__version__ = '.'.join(map(str, VERSION_INFO))
