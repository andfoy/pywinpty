# -*- coding: utf-8 -*-
"""Setup script for pywinpty."""

# yapf: disable

# Standard library imports
import ast
import os

# Third party imports
from setuptools import Extension, find_packages, setup


# yapf: enable

HERE = os.path.abspath(os.path.dirname(__file__))


def get_version(module='winpty'):
    """Get version from text file and avoids importing the module."""
    with open(os.path.join(HERE, module, '__init__.py'), 'r') as f:
        data = f.read()
    lines = data.split('\n')
    for line in lines:
        if line.startswith('VERSION_INFO'):
            version_tuple = ast.literal_eval(line.split('=')[-1].strip())
            version = '.'.join(map(str, version_tuple))
            break
    return version


try:
    include_dirs = [os.environ['LIBRARY_INC']]
except KeyError:
    include_dirs = []
try:
    library_dirs = [os.environ['LIBRARY_LIB']]
except KeyError:
    library_dirs = []


ext_options = {}
cythonize_options = {}
if os.environ.get('CYTHON_COVERAGE'):
    cythonize_options['compiler_directives'] = {'linetrace': True}
    cythonize_options['annotate'] = True
    ext_options['define_macros'] = [
        ('CYTHON_TRACE', '1'), ('CYTHON_TRACE_NOGIL', '1')
    ]

try:
    from Cython.Build import cythonize
    ext_modules = cythonize([
        Extension(
            "winpty.cywinpty",
            sources=["winpty/cywinpty.pyx"],
            libraries=["winpty"],
            include_dirs=include_dirs,
            library_dirs=library_dirs,
            **ext_options
        )],
        **cythonize_options
    )
except ImportError:
    ext_modules = []


setup(
    name='pywinpty',
    version=get_version(),
    keywords=['winpty', 'pty', 'pseudoterminal', 'pseudotty'],
    url='https://github.com/spyder-ide/pywinpty',
    license='MIT',
    author='Edgar Andrés Margffoy-Tuay',
    author_email='andfoy@gmail.com',
    description='Python bindings for the winpty library',
    ext_modules=ext_modules,
    packages=find_packages(exclude=['contrib', 'docs', 'tests*']),
    setup_requires=['Cython'],
    package_data=dict(winpty=['*.pyd', '*.dll', '*.exe']),
    install_requires=['backports.shutil_which;python_version<"3.0"'],
    classifiers=[
        'Development Status :: 4 - Beta', 'Intended Audience :: Developers',
        'License :: OSI Approved :: MIT License',
        'Operating System :: Microsoft :: Windows',
        'Programming Language :: Python :: 2.7',
        'Programming Language :: Python :: 3.5',
        'Programming Language :: Python :: 3.6'
    ]
)
