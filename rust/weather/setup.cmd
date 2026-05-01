@echo off
SET DEV=%HOMEDRIVE%%HOMEPATH%\dev
IF EXIST "%DEV%" CALL %DEV%\setup.cmd
TITLE rust-weather
SET PATH=%PATH%;%~dp0\target\debug
