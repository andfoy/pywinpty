
# Install general depedendencies for all Python versions
conda install -q --file conda.txt

# Install distribution and coverage tools
pip install -q twine codecov

if($env:PYTHON_VERSION -match "2.7") {}
    # On Python 2, we need to install this dependency
    conda install -q backports.shutil_which
}
