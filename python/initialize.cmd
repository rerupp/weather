@echo on
call %~dp0\PyO3.cmd
pip install --editable py_gui
pip install textual[syntax]
pip install textual-dev
pip install --editable py_tui
