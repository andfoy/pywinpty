# -*- coding: utf-8 -*-

"""Setup script for pywinpty."""

# Setuptools imports
from setuptools import setup, Extension, find_packages
from Cython.Build import cythonize

# Local imports
from winpty import __version__

REQUIREMENTS = ['cython', 'pypiwin32']

setup(
    name='winpty',
    version=__version__,
    keywords=['winpty'],
    url='https://github.com/spyder-ide/pywinpty',
    license='MIT',
    author='Edgar Andrés Margffoy Tuay',
    author_email='andfoy@gmail.com',
    description='Python bindings for the winpty library',
    ext_modules=cythonize([
        Extension("winpty.cywinpty", sources=["winpty/cywinpty.pyx"],
                  libraries=["winpty"])
    ]),
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
