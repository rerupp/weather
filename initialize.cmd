py -3.13-64 -m venv .venv
call .venv\Scripts\activate.bat
@echo on
python.exe -m pip install --upgrade pip
pip install maturin
pip install setuptools
pip install setuptools-scm
pip install importlib-resources
