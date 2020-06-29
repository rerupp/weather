from time import perf_counter as time_perf_counter


class StopWatch:
    """A simple timer."""
    def __init__(self, label: str = "", in_ms: bool = False):
        """
        Create and start a simple timer. Calling start multiple times
        will restart the internal timer. It does not need to be stopped
        in order to get the elapsed time.
        """
        self._label = label
        self._in_ms = in_ms
        self._start = self._stop = 0.0
        self.start()

    def start(self):
        self._start = self._stop = time_perf_counter()

    def stop(self):
        self._stop = time_perf_counter()

    def elapsed(self) -> float:
        stop = self._stop if self._stop != self._start else time_perf_counter()
        elapsed = stop - self._start
        return round(elapsed * 1000 if self._in_ms else elapsed, 3)

    def __str__(self):
        return f'{self._label + " " if self._label else ""}{self.elapsed():,.3f}{"ms" if self._in_ms else "s"}'
