from winpty.winpty import PTY

pty = PTY(80, 25)
pty.spawn('C:\\Windows\\System32\\cmd.exe')
pty.read(1000, blocking=True)
