from time import perf_counter as time_perf_counter


class StopWatch:
    _start: float = 0.0
    _end: float = _start
    _elapsed: float = 0.0

    def __init__(self, label: str = "", start: bool = False,  in_ms: bool = True):
        self._label = label
        self._in_ms = in_ms
        if start:
            self.start()

    def start(self):
        self._start = self._end = time_perf_counter()
        self._elapsed = 0.0

    def stop(self):
        if not self._elapsed:
            self._end = time_perf_counter()
            diff = self._end - self._start
            self._elapsed = round(diff * 1000) if self._in_ms else diff

    def elapsed(self):
        return self._elapsed

    def __str__(self):
        return "{}{}{}".format(self._label + " " if self._label else "", self._elapsed, "ms" if self._in_ms else "s")
