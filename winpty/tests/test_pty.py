# -*- coding: utf-8 -*-
"""winpty wrapper tests."""

# Standard library imports
import os
import time

# Third party imports
from flaky import flaky
from winpty import PTY
from winpty.enums import Backend
from winpty.ptyprocess import which
import pytest


CMD = bytes(which('cmd').lower(), 'utf-8')


def pty_factory(backend):
    @pytest.fixture(scope='function')
    def pty_fixture():
        pty = PTY(80, 20, backend=backend)
        loc = bytes(os.getcwd(), 'utf8')
        pty.spawn(CMD)
        time.sleep(0.3)
        return pty
    return pty_fixture


conpty_provider = pty_factory(Backend.ConPTY)
winpty_provider = pty_factory(Backend.WinPTY)


@pytest.fixture(scope='function', params=[
    pytest.lazy_fixture('conpty_provider'),
    pytest.lazy_fixture('winpty_provider')])
def pty_fixture(request):
    pty = request.param
    return pty


#@flaky(max_runs=4, min_passes=1)
def test_read(pty_fixture):
    pty = pty_fixture
    loc = os.getcwd()
    line = ''
    readline = ''

    while loc not in readline:
        readline += pty.read().decode('utf-8')
    assert loc in readline


def test_write(pty_fixture):
    pty = pty_fixture
    line = pty.read()

    str_text = 'Eggs, ham and spam ünicode'
    text = bytes(str_text, 'utf-8')
    num_bytes = pty.write(text)

    line = ''
    while str_text not in line:
        line += pty.read().decode('utf-8')

    assert str_text in line


def test_isalive(pty_fixture):
    pty = pty_fixture
    pty.write(b'exit\r\n')

    text = 'exit'
    line = ''
    while text not in line:
        line += pty.read().decode('utf-8')

    while pty.isalive():
        pty.read()
        continue

    assert not pty.isalive()
