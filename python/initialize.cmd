@echo on
py -3.13-64 -m venv .venv
call .venv\Scripts\activate.bat
pip install maturin
pip install setuptools
pip install setuptools-scm
pip install importlib-resources
pushd ..\rust\weather\py_lib
maturin develop
popd
pip install --editable py_gui
pip install textual[syntax]
pip install textual-dev
pip install --editable py_tui
