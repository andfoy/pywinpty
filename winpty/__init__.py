# -*- coding: utf-8 -*-

"""
Pywinpty
========
This package provides a Cython wrapper around winpty C++ library.
"""

from .cywinpty import Agent
from .winpty_wrapper import PTY

VERSION_INFO = (1, 0, "dev0")
__version__ = '.'.join(map(str, VERSION_INFO))
