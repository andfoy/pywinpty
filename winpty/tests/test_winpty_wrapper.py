# -*- coding: utf-8 -*-
"""winpty wrapper tests."""

# yapf: disable

# Standard library imports
import os

# Third party imports
from flaky import flaky
from winpty.winpty_wrapper import PTY, PY2
from winpty.ptyprocess import which
import pytest


# yapf: enable

CMD = which('cmd')
if PY2:
    CMD = unicode(CMD)  # noqa


@pytest.fixture(scope='module')
def pty_fixture(cols, rows):
    pty = PTY(cols, rows)
    pty.spawn(CMD)
    return pty


@flaky(max_runs=4, min_passes=1)
def test_read():
    pty = pty_fixture(80, 25)
    loc = os.getcwd()
    line = ''
    while loc not in line:
        line += pty.read().decode('utf-8')
    assert loc in line
    pty.close()
    del pty


def test_write():
    pty = pty_fixture(80, 25)
    line = pty.read()
    while len(line) < 10:
        line = pty.read()

    text = u'Eggs, ham and spam ünicode'
    pty.write(text)

    line = u''
    while text not in line:
        line += pty.read().decode('utf-8')

    assert text in line

    pty.close()
    del pty


def test_isalive():
    pty = pty_fixture(80, 25)
    pty.write(u'exit\r\n')

    text = u'exit'
    line = u''
    while text not in line:
        line += pty.read().decode('utf-8')

    while pty.isalive():
        pty.read()
        continue

    assert not pty.isalive()
    pty.close()
    del pty
