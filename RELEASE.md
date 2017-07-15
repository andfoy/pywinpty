To release a new version of pywinpty:

## General steps

* git fetch upstream && git merge upstream/master

* Close release issue on github

* Update CHANGELOG.md with loghub

* Update VERSION_INFO in `__init__.py` (set release version, remove 'dev0')

* git add and git commit

* Open MSVC Command Prompt or execute msvarsall.bat script

* set DISTUTILS_USE_SDK=1

* python setup.py sdist

## Per each PY3 version (3.5/3.6) and each architecture (amd64/win32) we should do

* conda install winpty -c spyder-ide

* set LIBRARY_INC=<CONDA_ENV_PATH>\Library\include

* set LIBRARY_LIB=<CONDA_ENV_PATH>\Library\lib

* python setup.py bdist_wheel

## Uploading wheels

* twine upload dist\\*

**Note:** Do not forget to update conda packages

## Create github release

* git tag -a vX.X.X -m 'comment'

* Update VERSION_INFO in `__init__.py` (add 'dev0' and increment minor)

* git add and git commit

* git push upstream master

* git push upstream --tags
