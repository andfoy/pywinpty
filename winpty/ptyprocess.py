# -*- coding: utf-8 -*-

# Standard library imports
from shutil import which
import codecs
import os
import shlex
import subprocess
import time
import uuid


# Local imports
from .winpty_wrapper import PTY


class PtyProcess(object):
    """This class represents a process running in a pseudoterminal.

    The main constructor is the :meth:`spawn` classmethod.
    """

    def __init__(self, proc):
        assert isinstance(proc, PTY)
        self.proc = proc
        self.fd = uuid.uuid4().hex
        self.pid = uuid.uuid4().hex
        self.decoder = codecs.getincrementaldecoder('utf-8')(errors='strict')

    @classmethod
    def spawn(cls, cmd, cwd=None, env=None, dimensions=(24, 80)):
        """Start the given command in a child process in a pseudo terminal.

        This does all the setting up the pty, and returns an instance of
        PtyProcess.

        Dimensions of the psuedoterminal used for the subprocess can be
        specified as a tuple (rows, cols), or the default (24, 80) will be
        used.
        """
        if isinstance(cmd, str):
            cmd = shlex.split(cmd, posix=False)

        if not isinstance(cmd, (list, tuple)):
            raise TypeError("Expected a list or tuple for cmd, got %r" % cmd)

        # Shallow copy of argv so we can modify it
        cmd = cmd[:]
        command = cmd[0]
        env = env or os.environ

        path = env.get('PATH', os.defpath)
        command_with_path = which(command, path=path)
        if command_with_path is None:
            raise FileNotFoundError(
                'The command was not found or was not ' +
                'executable: %s.' % command
            )
        command = command_with_path
        cmd[0] = command
        cmdline = ' ' + subprocess.list2cmdline(cmd[1:])
        cwd = cwd or os.getcwd()

        proc = PTY(dimensions[1], dimensions[0])

        # Create the environemnt string.
        envStrs = []
        for (key, value) in env.items():
            envStrs.append('%s=%s' % (key, value))
        env = '\0'.join(envStrs) + '\0'

        if len(cmd) == 1:
            proc.spawn(command, cwd=cwd, env=env)
        else:
            proc.spawn(command, cwd=cwd, env=env, cmdline=cmdline)

        inst = cls(proc)
        inst._winsize = dimensions
        return inst

    def close(self):
        """Close all communication process streams."""
        if self.proc:
            self.proc.close()

    def flush(self):
        """This does nothing. It is here to support the interface for a
        File-like object. """
        pass

    def isatty(self):
        """This returns True if the file descriptor is open and connected to a
        tty(-like) device, else False."""
        return self.isalive()

    def read(self, size=1024):
        """Read and return at most ``size`` bytes from the pty.
        """
        if not self.isalive():
            raise EOFError('Pty is closed')

        data = self.proc.read(size)
        return self.decoder.decode(data, final=False)

    def readline(self):
        """Read one line from the pseudoterminal as bytes.

        Can block if there is nothing to read. Raises :exc:`EOFError` if the
        terminal was closed.
        """
        buf = []
        while 1:
            if not self.isalive():
                return ''.join(buf)
            ch = self.read(1)
            if not ch:
                time.sleep(0.1)
            buf.append(ch)
            if ch == '\n':
                return ''.join(buf)

    def write(self, s):
        """Write the string ``s`` to the pseudoterminal.

        Returns the number of bytes written.
        """
        if not self.isalive():
            raise EOFError('Pty is closed')
        success, nbytes = self.proc.write(s)
        if not success:
            raise IOError('Write failed')
        return nbytes

    def terminate(self):
        """This forces a child process to terminate."""
        del self.proc
        self.proc = None

    def wait(self):
        """This waits until the child exits. This is a blocking call. This will
        not read any data from the child.
        """
        while self.isalive():
            self.readline()
        return 0

    def isalive(self):
        """This tests if the child process is running or not. This is
        non-blocking. If the child was terminated then this will read the
        exitstatus or signalstatus of the child. This returns True if the child
        process appears to be running or False if not.
        """
        return self.proc and self.proc.isalive()

    def kill(self, sig=None):
        """Kill the process.  This is an alias to terminate.
        """
        self.terminate()

    def getwinsize(self):
        """Return the window size of the pseudoterminal as a tuple (rows, cols).
        """
        return self._winsize

    def setwinsize(self, rows, cols):
        """Set the terminal window size of the child tty.
        """
        self._winsize = (rows, cols)
        self.proc.set_size(cols, rows)
