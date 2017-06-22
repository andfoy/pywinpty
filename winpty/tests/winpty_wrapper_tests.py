# -*- coding: utf-8 -*-

import pytest
from winpty.winpty_wrapper import PTY

CMD = r'C:\windows\system32\cmd.exe'


@pytest.fixture(scope='module')
def pty_fixture(cols, rows):
    pty = PTY(cols, rows)
    pty.spawn(CMD)
    return pty


def read_test():
    pty = pty_fixture(80, 25)
    line = pty.read()
    assert 'Microsoft' in line
    del pty


def write_test():
    pty = pty_fixture(80, 25)
    pty.read()
    text = 'Eggs, ham and spam ünicode'
    pty.write(text)
    line = pty.read()
    assert text in line
