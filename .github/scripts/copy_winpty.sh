set -eoux

python_exec=$(which python)
bin_path=$(dirname $python_exec)

# Patch gitignore in order to add binaries
sed -i '/[.]exe/c\' .gitignore
sed -i '/[.]dll/c\' .gitignore

# Copy WinPTY binaries to the main library directory
cp "$bin_path/Library/bin/winpty.dll" winpty
cp "$bin_path/Library/bin/winpty-agent.exe" winpty
