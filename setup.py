from setuptools import setup, Extension, find_packages
# from distutils.extension import Extension
from Cython.Build import cythonize
# import os.path as osp

setup(
    name='winpty',
    version='1.0',
    ext_modules=cythonize([
        Extension("winpty.cywinpty", sources=["winpty/cywinpty.pyx"],
                  libraries=["winpty"])
    ]),
    packages=find_packages(exclude=['contrib', 'docs', 'tests*']),
    include_package_data=True
)
