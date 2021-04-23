from winpty.winpty import PTY
from winpty.ptyprocess import PtyProcess


# pty = PTY(80, 25)
# pty.spawn(b'C:\\Windows\\System32\\cmd.exe')
# x = pty.read(1000, blocking=True)
# print(repr(x))

pty = PtyProcess.spawn('cmd', env=None, backend=0)
pty.isalive()

