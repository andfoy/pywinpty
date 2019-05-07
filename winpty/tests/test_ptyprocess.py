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
def pty_fixture(cmd=None, **kwargs):
    cmd = cmd or 'cmd'
    return PtyProcess.spawn(cmd, **kwargs)


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

    text = u'Eggs, ham and spam ünicode'
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

    while 1:
        try:
            pty.read()
        except EOFError:
            break

    assert not pty.isalive()
    pty.terminate()


def test_readline():
    env = os.environ.copy()
    env['foo'] = 'bar'
    pty = pty_fixture(env=env)
    pty.write('echo %foo%\r\nexit\r\n')

    while 'bar' not in pty.readline():
        pass

    while 1:
        try:
            pty.readline()
        except EOFError:
            break

    assert not pty.isalive()


def test_no_emit_cursors():
    pty = pty_fixture(emit_cursors=False)
    assert '\x1b[0K' not in pty.readline()
    pty.terminate()

    pty = pty_fixture()
    assert '\x1b[0K' in pty.readline()
    pty.terminate()


def test_close():
    pty = pty_fixture()
    pty.close()
    assert not pty.isalive()


def test_flush():
    pty = pty_fixture()
    pty.flush()
    pty.terminate()


def test_intr():
    pty = pty_fixture(cmd=[sys.executable, 'import time; time.sleep(10)'])
    pty.sendintr()
    assert pty.wait() != 0


def test_send_control():
    pty = pty_fixture(cmd=[sys.executable, 'import time; time.sleep(10)'])
    pty.sendcontrol('d')
    assert pty.wait() != 0


def test_send_eof():
    cat = pty_fixture('cat')
    cat.sendeof()
    assert cat.wait() == 0


def test_isatty():
    pty = pty_fixture()
    assert pty.isatty()
    pty.terminate()
    assert not pty.isatty()


def test_wait():
    pty = pty_fixture(cmd=[sys.executable, '--version'])
    assert pty.wait() == 0


def test_exit_status():
    pty = pty_fixture(cmd=[sys.executable])
    pty.write('import sys;sys.exit(1)\r\n')
    pty.wait()
    assert pty.exitstatus == 1


def test_kill():
    pty = pty_fixture()
    pty.kill(signal.SIGTERM)
    assert not pty.isalive()
    assert pty.exitstatus == signal.SIGTERM


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
