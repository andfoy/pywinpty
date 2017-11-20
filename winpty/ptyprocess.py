# -*- coding: utf-8 -*-

# Standard library imports
from shutil import which
import codecs
import os
import shlex
import signal
import socket
import subprocess
import threading
import time


# Local imports
from .winpty_wrapper import PTY


class PtyProcess(object):
    """This class represents a process running in a pseudoterminal.

    The main constructor is the :meth:`spawn` classmethod.
    """

    def __init__(self, pty):
        assert isinstance(pty, PTY)
        self.pty = pty
        self.pid = pty.pid
        self.fd = pty.conout_pipe
        self.decoder = codecs.getincrementaldecoder('utf-8')(errors='strict')
        # Used by terminate() to give kernel time to update process status.
        # Time in seconds.
        self.delayafterterminate = 0.1

        # Set up our file reader sockets.
        self._server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        address = _get_address()
        self._server.bind(address)
        self._server.listen(1)

        # Read from the pty in a thread.
        self._thread = threading.Thread(target=self._read_in_thread,
            args=(address,))
        self._thread.setDaemon(True)
        self._thread.start()

        self.fileobj, _ = self._server.accept()
        self.fd = self.fileobj.fileno()

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

    @property
    def exitstatus(self):
        """The exit status of the process.
        """
        return self.pty.exitstatus

    def fileno(self):
        """This returns the file descriptor of the pty for the child.
        """
        return self.fd

    def close(self):
        """Close all communication process streams.
        """
        if self.pty:
            self.pty.close()
            self.fileobj.close()
            self._server.close()

    def __del__(self):
        """This makes sure that no system resources are left open. Python only
        garbage collects Python objects. OS file descriptors are not Python
        objects, so they must be handled explicitly. If the child file
        descriptor was opened outside of this class (passed to the constructor)
        then this does not close it.
        """
        # It is possible for __del__ methods to execute during the
        # teardown of the Python VM itself. Thus self.close() may
        # trigger an exception because os.close may be None.
        try:
            del self.pty
            self.pty = None
            self.close()
        except Exception:
            pass

    def flush(self):
        """This does nothing. It is here to support the interface for a
        File-like object. """
        pass

    def isatty(self):
        """This returns True if the file descriptor is open and connected to a
        tty(-like) device, else False."""
        return self.isalive()

    def read(self, size=1024):
        """Read and return at most ``size`` characters from the pty.

        Can block if there is nothing to read. Raises :exc:`EOFError` if the
        terminal was closed.
        """
        data = self.fileobj.recv(size)
        if not data:
            raise EOFError('Pty is closed')
        return self.decoder.decode(data, final=False)

    def readline(self):
        """Read one line from the pseudoterminal as bytes.

        Can block if there is nothing to read. Raises :exc:`EOFError` if the
        terminal was closed.
        """
        buf = []
        while 1:
            try:
                ch = self.read(1)
            except EOFError:
                return ''.join(buf)
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
        success, nbytes = self.pty.write(s)
        if not success:
            raise IOError('Write failed')
        return nbytes

    def terminate(self, force=False):
        """This forces a child process to terminate."""
        if not self.isalive():
            return True
        self.kill(signal.SIGINT)
        time.sleep(self.delayafterterminate)
        if not self.isalive():
            return True
        if force:
            self.kill(signal.SIGKILL)
            time.sleep(self.delayafterterminate)
            if not self.isalive():
                return True
            else:
                return False

    def wait(self):
        """This waits until the child exits. This is a blocking call. This will
        not read any data from the child.
        """
        while self.isalive():
            time.sleep(0.1)
        return self.exitstatus

    def isalive(self):
        """This tests if the child process is running or not. This is
        non-blocking. If the child was terminated then this will read the
        exitstatus or signalstatus of the child. This returns True if the child
        process appears to be running or False if not.
        """
        return self.pty and self.pty.isalive()

    def kill(self, sig=None):
        """Kill the process with the given signal.
        """
        os.kill(self.pid, sig)

    def getwinsize(self):
        """Return the window size of the pseudoterminal as a tuple (rows, cols).
        """
        return self._winsize

    def setwinsize(self, rows, cols):
        """Set the terminal window size of the child tty.
        """
        self._winsize = (rows, cols)
        self.pty.set_size(cols, rows)

    def _read_in_thread(self, address):
        """Read data from the pty in a thread.
        """
        client = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        client.connect(address)

        while 1:
            data = self.pty.read(4096)

            if not data and not self.isalive():
                while not data and not self.pty.iseof():
                    time.sleep(0.1)
                    data += self.pty.read(4096)

                if not data:
                    try:
                        client.send(b'')
                    except socket.error:
                        pass
                    break
            try:
                client.send(data)
            except socket.error:
                break

        client.close()


def _get_address(default_port=20128):
    """Find and return a non used port"""
    while True:
        try:
            sock = socket.socket(socket.AF_INET,
                                 socket.SOCK_STREAM,
                                 socket.IPPROTO_TCP)
            sock.bind(("127.0.0.1", default_port))
        except socket.error as _msg:  # analysis:ignore
            default_port += 1
        else:
            break
        finally:
            sock.close()
            sock = None
    return ("127.0.0.1", default_port)
