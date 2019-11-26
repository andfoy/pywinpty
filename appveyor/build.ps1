
if($env:PYTHON_VERSION -match "2.7") {
    if ($env:ARCH -match "amd64") {
        # We need to pass the -DMS_WIN64 flag in order to prevent a
        # compilation error on Python 2.7 (x64)
        python setup.py build_ext -i --compiler=mingw32 -DMS_WIN64
    }
    else {
        python setup.py build_ext -i --compiler=mingw32
    }
}
else { python setup.py build_ext -i --compiler=mingw32 }
