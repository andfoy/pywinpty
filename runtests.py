# -*- coding: utf-8 -*-

"""Script used to run pytest programatically."""

# Standard library imports
import argparse
import os

# Standard library imports
import pytest


def run_pytest(extra_args=None):
    pytest_args = ['-v', '-x']

    # Allow user to pass a custom test path to pytest to e.g. run just one test
    if extra_args:
        pytest_args += extra_args

    print("Pytest Arguments: " + str(pytest_args))
    errno = pytest.main(pytest_args)

    # sys.exit doesn't work here because some things could be running in the
    # background (e.g. closing the main window) when this point is reached.
    # If that's the case, sys.exit doesn't stop the script as you would expect.
    if errno != 0:
        raise SystemExit(errno)


def main():
    """Parse args then run the pytest suite for pywinpty."""
    test_parser = argparse.ArgumentParser(
        usage='python runtests.py [-h] [pytest_args]',
        description="Helper script to run pywinpty's test suite")
    test_parser.add_argument('--run-slow', action='store_true', default=False,
                             help='Run the slow tests')
    _, pytest_args = test_parser.parse_known_args()
    run_pytest(extra_args=pytest_args)


if __name__ == '__main__':
    main()
