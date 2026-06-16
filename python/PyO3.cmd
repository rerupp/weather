@echo on
pushd %~dp0..\rust\py_weather_lib
maturin develop
popd
