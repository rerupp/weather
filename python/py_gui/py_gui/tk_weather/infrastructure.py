"""This module is holds the common classes."""

from time import time_ns

__all__ = ['Stopwatch', 'WeatherEvent', 'WeatherView']


class WeatherEvent:
    REFRESH_VIEW = '<<WeatherEvent.refresh_view>>'


class WeatherView:

    def refresh(self):
        pass

    def view(self):
        raise NotImplementedError


class Stopwatch:
    def __init__(self):
        self._start_ns = self._stop_ns = time_ns()

    def __str__(self) -> str:
        stop_ns = self._stop_ns if self._start_ns != self._stop_ns else time_ns()
        return f'{(stop_ns - self._start_ns) / (10 ** 6):.3f} ms'

    def stop(self):
        if self._start_ns == self._stop_ns:
            self._stop_ns = time_ns()

    def restart(self) -> None:
        self._start_ns = self._stop_ns = time_ns()
