# -*- coding: utf-8 -*-

import pytest
from winpty.cywinpty import Agent

CMD = r'C:\windows\system32\cmd.exe'


@pytest.fixture(scope='module')
def agent_fixture(cols, rows):
    agent = Agent(cols, rows)
    return agent


def agent_spawn_test():
    agent = agent_fixture(80, 25)
    succ = agent.spawn(CMD)
    assert succ
    del agent


def agent_resize_test():
    agent = agent_fixture(80, 25)
    agent.set_size(80, 70)
    del agent
