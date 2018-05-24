# pywinpty
Python bindings for the [winpty](https://github.com/rprichard/winpty) pseudo terminal library. It
allows to create and communicate with Windows processes that print outputs and recieve inputs via
console input and output pipes.

## Project details
[![Project License](https://img.shields.io/pypi/l/pywinpty.svg)](./LICENSE.txt)
[![pypi version](https://img.shields.io/pypi/v/pywinpty.svg)](https://pypi.python.org/pypi/pywinpty)
[![Join the chat at https://gitter.im/spyder-ide/public](https://badges.gitter.im/spyder-ide/spyder.svg)](https://gitter.im/spyder-ide/public)
[![OpenCollective Backers](https://opencollective.com/spyder/backers/badge.svg?color=blue)](#backers)
[![OpenCollective Sponsors](https://opencollective.com/spyder/sponsors/badge.svg?color=blue)](#sponsors)

## Build status
[![Build status](https://ci.appveyor.com/api/projects/status/cowkuaebgeeq45v1?svg=true)](https://ci.appveyor.com/project/spyder-ide/pywinpty)
[![Coverage Status](https://coveralls.io/repos/github/spyder-ide/pywinpty/badge.svg?branch=master)](https://coveralls.io/github/spyder-ide/pywinpty?branch=master)
[![codecov](https://codecov.io/gh/spyder-ide/pywinpty/branch/master/graph/badge.svg)](https://codecov.io/gh/spyder-ide/pywinpty)

----

## Important Announcement: Spyder is unfunded!

Since mid November/2017, [Anaconda, Inc](https://www.anaconda.com/) has
stopped funding Spyder development, after doing it for the past 18
months. Because of that, development will focus from now on maintaining
Spyder 3 at a much slower pace than before.

If you want to contribute to maintain Spyder, please consider donating at

https://opencollective.com/spyder

We appreciate all the help you can provide us and can't thank you enough for
supporting the work of Spyder devs and Spyder development.

If you want to know more about this, please read this
[page](https://github.com/spyder-ide/spyder/wiki/Anaconda-stopped-funding-Spyder).

----


## Dependencies
To compile pywinpty sources, you must have [Cython](https://github.com/cython/cython) and MSYS2/MinGW-w64 installed (Alongside the corresponding Python MSVC Runtime). You must also have Winpty's C header and library files available
on your include path.

## Installation
You can install this library by using conda or pip package managers, as it follows:

Using conda (Recommended):
```bash
conda install pywinpty
```

Using pip:
```bash
pip install pywinpty
```

## Building from sources

To build from sources, we recommend to use conda to install the following packages:

```batch
conda install --file requirements.txt
```

Make sure that you are installing packages from the ``default`` channel. If you don't want to use conda, you will need to have the MSYS2/MinGW-w64-flavoured GCC compiler available on your PATH.

You will need to setup the following environment variables:

### Conda compilation:
```batch
set LIBRARY_INC=<Path to your anaconda installation>\envs\<environment name>\Library\include
set LIBRARY_LIB=<Path to your anaconda installation>\envs\<environment name>\Library\lib
```

### Manual compilation:
```batch
set LIBRARY_INC=<Path to the folder that contains wintpty headers>
set LIBRARY_LIB=<Path to the folder that contains wintpty library files>
```

To test your compilation environment settings, you can build pywinpty Cython sources locally, by
executing:

```bash
python setup.py build_ext -i --compiler=mingw32
```

If everything works correctly, you can install winpty by using ``pip``:

```bash
pip install -U .
```


## Package usage
Pywinpty offers a single python wrapper around winpty library functions. This implies that using a
single object (``winpty.PTY``) it is possible to access to all functionallity, as it follows:

```python
# High level usage using `spawn`
from winpty import PtyProcess

proc = PtyProcess.spawn('python')
proc.write('print("hello, world!")\r\n')
proc.write('exit()\r\n')
while proc.isalive():
	print(proc.readline())

# Low level usage using the raw `PTY` object
from winpty import PTY

# Start a new winpty-agent process of size (cols, rows)
cols, rows = 80, 25
process = PTY(cols, rows)

# Spawn a new console process, e.g., CMD
process.spawn(ur'C:\windows\system32\cmd.exe')

# Read console output (Unicode)
process.read()

# Write input to console (Unicode)
process.write(u'Text')

# Resize console size
new_cols, new_rows = 90, 30
process.set_size(new_cols, new_rows)

# Know if the process is alive
alive = process.isalive()

# Close console pipes
process.close()

# End winpty-agent process
del process
```

## Changelog
Visit our [CHANGELOG](CHANGELOG.md) file to know more about our new features and improvements.

## Contribution guidelines
We follow PEP8 and PEP257 for pure python packages and Cython/VS to compile extensions. Feel free
to send a PR or create an issue if you have any problem/question.

## Backers

Support us with a monthly donation and help us continue our activities.

[![Backers](https://opencollective.com/spyder/backers.svg)](https://opencollective.com/spyder#support)

## Sponsors

Become a sponsor to get your logo on our README on Github.

[![Sponsors](https://opencollective.com/spyder/sponsors.svg)](https://opencollective.com/spyder#support)
