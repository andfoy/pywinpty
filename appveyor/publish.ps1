
if ($env:APPVEYOR_REPO_TAG -match "true") {
    $TWINE_USERNAME = $env:TWINE_USERNAME
    $TWINE_PASSWORD = $env:TWINE_PASSWORD

    $LIB_BIN_PATH = "Library\bin"
    $CONDA_ENV_PATH = split-path -Path ((Get-Command python).Path)
    $CONDA_ENV_BIN = join-path $CONDA_ENV_PATH $LIB_BIN_PATH
    $WINPTY_EXE = join-path $CONDA_ENV_BIN "winpty-agent.exe"
    # $WINPTY_DEBUG = join-path $CONDA_ENV_BIN "winpty-debugserver.exe"
    $WINPTY_DLL = join-path $CONDA_ENV_BIN "winpty.dll"

    $MINGW_PATH = split-path -Path ((Get-Command gcc).Path)
    $LIB_GCC_32 = join-path $MINGW_PATH "libgcc_s_dw2-1.dll"
    $LIB_GCC_64 = join-path $MINGW_PATH "libgcc_s_seh-1.dll"
    $LIBPTHREAD = join-path $MINGW_PATH "libwinpthread-1.dll"

    $FOLDER = "winpty\"

    # Copy winpty binaries to winpty folder
    copy-item $WINPTY_EXE $FOLDER
    # copy-item $WINPTY_DEBUG $FOLDER
    copy-item $WINPTY_DLL $FOLDER
    copy-item $LIBPTHREAD $FOLDER

    if ($env:ARCH -match "amd64") {
        copy-item $LIB_GCC_64 $FOLDER
        if($env:PYTHON_VERSION -match "3.7") {
            python setup.py sdist
        }
    }

    if ($env:ARCH -match "x86") {
        copy-item $LIB_GCC_32 $FOLDER
    }

    python setup.py bdist_wheel
    twine upload dist\*

}
else { echo "Not deploying because not a tagged commit." }
