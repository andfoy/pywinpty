set -eoux

python_exec=$(which python)
bin_path=$(dirname $python_exec)

# Copy WinPTY binaries to the main library directory
cp "$bin_path/Library/bin/winpty.dll" winpty
cp "$bin_path/Library/bin/winpty-agent.exe" winpty
