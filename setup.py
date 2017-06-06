from setuptools import setup, Extension
# from distutils.extension import Extension
from Cython.Build import cythonize
# import os.path as osp

setup(
    ext_modules=cythonize([
        Extension("winpty", ["winpty/winpty.pyx"], libraries=["winpty"],
                  language="c++")
    ]),
)
