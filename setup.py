from setuptools import setup, Extension
# from distutils.extension import Extension
from Cython.Build import cythonize
# import os.path as osp

setup(
    name='winpty',
    version='1.0',
    ext_modules=cythonize([
        Extension("winpty.cy_winpty", sources=["winpty/winpty.pyx"],
                  libraries=["winpty"])
    ]),
)
