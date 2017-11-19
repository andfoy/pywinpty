

from winpty import PtyProcess, PTY
from shutil import which
#proc = PtyProcess.spawn('bash', emit_cursors=True)
#proc.write('print("hello")\r\nquit()\r\n')

proc = PTY(80, 10)
import os


path = os.path.abspath('./test.sh').replace(os.sep, '/')
proc.spawn(which('bash'), ' -c %s' % path)
#proc.write('ls\rsleep 2\rclear\rsleep 2\rexit\r')
while 1:
	data = proc.read()
	if data:
		print(data.replace(b'\x1b', b'b').decode('utf-8'), end='')
	else:
		import time
		time.sleep(0.1)