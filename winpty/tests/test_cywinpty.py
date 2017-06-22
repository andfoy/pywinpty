# -*- coding: utf-8 -*-

import pytest
from winpty.cywinpty import Agent

CMD = r'C:\windows\system32\cmd.exe'


@pytest.fixture(scope='module')
def agent_fixture(cols, rows):
    agent = Agent(cols, rows)
    return agent


def test_agent_spawn():
    agent = agent_fixture(80, 25)
    succ = agent.spawn(CMD)
    assert succ
    del agent


def test_agent_spawn_fail():
    agent = agent_fixture(80, 25)
    try:
        agent.spawn('cmd.exe')
    except RuntimeError:
        pass


def test_agent_resize():
    agent = agent_fixture(80, 25)
    agent.set_size(80, 70)
    del agent
