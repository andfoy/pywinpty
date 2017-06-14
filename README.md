# pywinpty
Python bindings for the [winpty](https://github.com/rprichard/winpty) pseudo terminal library. It allows to create and communicate with Windows processes that print outputs and recieve inputs via console input and output pipes.

## Project info
[![Project License](https://img.shields.io/pypi/l/winpty.svg)](./LICENSE.txt)
[![pypi version](https://img.shields.io/pypi/v/winpty.svg)](https://pypi.python.org/pypi/winpty)

## Build status
[![AppVeyor status](https://ci.appveyor.com/api/projects/status/tvjcqa4kf53br8s0/branch/master?svg=true)](https://ci.appveyor.com/project/spyder-ide/pywinpty/branch/master)

## Installation
To install this library, you must have the winpty libraries available on your system path, to install them, you can either compile from winpty source or you can install them using conda:

```bash
conda install winpty -c conda-forge
```

Then, you can install this library by using conda or pip package managers, as it follows:

Using conda:
```bash
conda install pywinpty -c conda-forge
```

Using pip:
```
pip install winpty
```

Due to Visual Studio version incompatibilities, we currently only support Python 3.5 and 3.6 versions.

## Dependencies
To compile pywinpty sources, you must have [Cython](https://github.com/cython/cython) and Visual Studio (2013 or later) installed. You must also have Winpty's C header files available on your include path.

## Changelog
Visit our [CHANGELOG](CHANGELOG.md) file to know more about our new features and improvements.

## Contribution guidelines
We follow PEP8 and PEP257 for pure python packages and Cython/VS to compile extensions. Feel free to send a PR or create an issue if you have any problem/question.
