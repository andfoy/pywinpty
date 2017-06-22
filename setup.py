# -*- coding: utf-8 -*-

"""Setup script for pywinpty."""

# Standard Library imports
import os

# Setuptools imports
from setuptools import setup, Extension, find_packages
from Cython.Build import cythonize

# Local imports
from winpty import __version__

try:
    include_dirs = [os.environ['LIBRARY_INC']]
except KeyError:
    include_dirs = []
try:
    library_dirs = [os.environ['LIBRARY_LIB']]
except KeyError:
    library_dirs = []


REQUIREMENTS = ['cython']

try:
    # Only if building on conda-build
    import win32file
except ImportError:
    # Add to pip requirements
    REQUIREMENTS += ['pypiwin32']

ext_options = {}
cythonize_options = {}
if os.environ.get('CYTHON_COVERAGE'):
    cythonize_options['compiler_directives'] = {'linetrace': True}
    cythonize_options['annotate'] = True
    ext_options['define_macros'] = [('CYTHON_TRACE', '1'),
                                    ('CYTHON_TRACE_NOGIL', '1')]

setup(
    name='pywinpty',
    version=__version__,
    keywords=['winpty'],
    url='https://github.com/spyder-ide/pywinpty',
    license='MIT',
    author='Edgar Andrés Margffoy Tuay',
    author_email='andfoy@gmail.com',
    description='Python bindings for the winpty library',
    ext_modules=cythonize([
        Extension("winpty.cywinpty", sources=["winpty/cywinpty.pyx"],
                  libraries=["winpty"], include_dirs=include_dirs,
                  library_dirs=library_dirs, **ext_options)
    ], **cythonize_options),
    packages=find_packages(exclude=['contrib', 'docs', 'tests*']),
    include_package_data=True,
    install_requires=REQUIREMENTS,
    classifiers=[
        'Development Status :: 4 - Beta',
        'Intended Audience :: Developers',
        'License :: OSI Approved :: MIT License',
        'Operating System :: Microsoft :: Windows',
        'Programming Language :: Python :: 3.5',
        'Programming Language :: Python :: 3.6'
    ]
)
