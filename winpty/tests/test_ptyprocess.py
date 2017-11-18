# -*- coding: utf-8 -*-
"""winpty wrapper tests."""

# yapf: disable

# Standard library imports
import os
import signal
import sys

# Third party imports
from flaky import flaky
from winpty.ptyprocess import PtyProcess
import pytest


# yapf: enable


@pytest.fixture(scope='module')
def pty_fixture(cmd=None, env=None):
    cmd = cmd or 'cmd'
    return PtyProcess.spawn(cmd, env=env)


@flaky(max_runs=4, min_passes=1)
def test_read():
    pty = pty_fixture()
    loc = os.getcwd()
    data = ''
    while loc not in data:
        data += pty.read()
    pty.terminate()


def test_write():
    pty = pty_fixture()

    text = 'Eggs, ham and spam ünicode'
    pty.write(text)

    data = ''
    while text not in data:
        data += pty.read()
    pty.terminate()


def test_isalive():
    pty = pty_fixture()
    pty.write('exit\r\n')

    text = 'exit'
    data = ''
    while text not in data:
        data += pty.read()

    while pty.isalive():
        pty.read()
        continue

    assert not pty.isalive()
    pty.terminate()


def test_readline():
    env = os.environ.copy()
    env['foo'] = 'bar'
    pty = pty_fixture(env=env)
    pty.write('echo %foo%\r\n')

    while 'bar' not in pty.readline():
        pass

    pty.terminate()


def test_close():
    pty = pty_fixture()
    pty.close()
    assert not pty.isalive()
    pty.terminate()


def test_flush():
    pty = pty_fixture()
    pty.flush()
    pty.terminate()


def test_isatty():
    pty = pty_fixture()
    assert pty.isatty()
    pty.terminate()
    assert not pty.isatty()


def test_wait():
    pty = pty_fixture(cmd=[sys.executable, "--version"])
    assert pty.wait() == 0
    pty.kill()


def test_kill():
    pty = pty_fixture()
    pty.kill(signal.SIGTERM)
    assert not pty.isalive()


def test_getwinsize():
    pty = pty_fixture()
    assert pty.getwinsize() == (24, 80)
    pty.terminate()


def test_setwinsize():
    pty = pty_fixture()
    pty.setwinsize(50, 110)
    assert pty.getwinsize() == (50, 110)
    pty.terminate()

    pty = PtyProcess.spawn('cmd', dimensions=(60, 120))
    assert pty.getwinsize() == (60, 120)
    pty.terminate()
