# -*- coding: utf-8 -*-
"""winpty wrapper tests."""

# yapf: disable

# Standard library imports
import os

# Third party imports
from flaky import flaky
from winpty.winpty_wrapper import PTY
import pytest


# yapf: enable

CMD = r'C:\windows\system32\cmd.exe'


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
        line += str(pty.read(), 'utf-8')
    assert loc in line
    pty.close()
    del pty


def test_write():
    pty = pty_fixture(80, 25)
    line = pty.read()
    while len(line) < 10:
        line = pty.read()

    text = 'Eggs, ham and spam ünicode'
    pty.write(text)

    line = ''
    while text not in line:
        line += str(pty.read(), 'utf-8')

    assert text in line

    pty.close()
    del pty


def test_isalive():
    pty = pty_fixture(80, 25)
    pty.write('exit\r\n')

    text = 'exit'
    line = ''
    while text not in line:
        line += str(pty.read(), 'utf-8')

    while pty.isalive():
        pty.read()
        continue

    assert not pty.isalive()
    pty.close()
    del pty
