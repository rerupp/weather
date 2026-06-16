@echo off
SET DEV=%HOMEDRIVE%%HOMEPATH%\dev
IF EXIST "%DEV%" CALL %DEV%\setup.cmd
IF EXIST ..\.venv CALL ..\.venv\Scripts\activate.bat
TITLE rust-weather
SET PATH=%PATH%;%~dp0\target\debug
