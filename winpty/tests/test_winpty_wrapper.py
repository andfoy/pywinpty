# -*- coding: utf-8 -*-

import os
import pytest
import os.path as osp
from winpty.winpty_wrapper import PTY

CMD = r'C:\windows\system32\cmd.exe'
LOCATION = osp.realpath(osp.join(os.getcwd(),
                                 osp.dirname(__file__)))


@pytest.fixture(scope='module')
def pty_fixture(cols, rows):
    pty = PTY(cols, rows)
    pty.spawn(CMD)
    return pty


def test_read():
    pty = pty_fixture(80, 25)
    line = pty.read()
    while len(line) < 30:
        line = pty.read()
    line = str(line, 'utf-8')
    assert LOCATION in line
    del pty


def test_write():
    pty = pty_fixture(80, 25)
    line = pty.read()
    while len(line) < 10:
        line = pty.read()
    text = 'Eggs, ham and spam ünicode'
    pty.write(text)
    line = pty.read()
    while len(line) < 10:
        line = pty.read()
    line = str(line, 'utf-8')
    assert text in line
