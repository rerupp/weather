from calendar import monthrange
from collections.abc import MutableMapping
from datetime import date, timedelta, MAXYEAR, MINYEAR
from tkinter import *
from tkinter import messagebox
from tkinter.font import nametofont, Font
from tkinter.ttk import *
from typing import Tuple, Union, NamedTuple, List, Optional, Callable, Iterator, Sequence, Any

from tksheet import Sheet

from weather.domain import DailyWeatherContent, DateRange, WeatherData
from weather.configuration import get_logger
from .gui_utils import month_name, get_tag_builder, Coord


log = get_logger(__name__)


class DailyTemperature(NamedTuple):
    ts: date
    low: float
    high: float


class DailyTemperatureDataPointDict(MutableMapping):

    def __init__(self, *args, **kwargs):
        self.__dict__.update(*args, **kwargs)
        self.__to_key = lambda d: d.strftime("%m%d")

    def __getitem__(self, ts: date) -> Coord:
        return self.__dict__[self.__to_key(ts)]

    def __setitem__(self, ts: date, coord: Coord) -> None:
        self.__dict__[self.__to_key(ts)] = coord

    def __delitem__(self, ts: date) -> None:
        del self.__dict__[self.__to_key(ts)]

    def __len__(self) -> int:
        return len(self.__dict__)

    def __iter__(self) -> Iterator:
        return iter(self.__dict__)

    def __str__(self):
        return str(self.__dict__)


class DailyTemperatureGraph(Frame):

    def __init__(self,
                 master,
                 date_range: DateRange,
                 temperature_range: Tuple[int, int] = (20, 110),
                 title="TBD",
                 offset: Coord = Coord(20, 35)):
        super(DailyTemperatureGraph, self).__init__(master)
        self.grid(sticky=NSEW)
        self.grid_rowconfigure(0, weight=1)
        self.grid_columnconfigure(0, weight=1)

        self._canvas = Canvas(self,
                              bg="white",
                              borderwidth=1,
                              relief=GROOVE)
        self._canvas.grid(row=0, column=0, sticky=NSEW, padx=1, pady=1)
        self._canvas.grid_rowconfigure(0, weight=1)
        self._canvas.grid_columnconfigure(0, weight=1)

        #######################################################################
        #
        # axis
        #
        #######################################################################

        # font support

        def make_font_builder(base_font_: Font) -> Callable[[int], Font]:
            def builder(size_: int):
                font_ = base_font_.copy()
                font_.config(size=size_)
                return font_

            return builder

        make_font = make_font_builder(nametofont("TkDefaultFont"))

        axis_label_font = make_font(12)
        axis_tick_font = make_font(8)
        axis_line_width = 2
        axis_color = "gray"

        self._low_temp, self._high_temp = temperature_range
        self._pixels_per_degree = 4 if self._high_temp - self._low_temp > 40 else 6
        v_label_width = 40
        v_axis_len = (self._high_temp - self._low_temp) * self._pixels_per_degree

        # horizontal axis metrics
        h_label_height: int = axis_label_font.metrics("linespace")
        self._starting_date, self._ending_date = date_range
        days = (self._ending_date - self._starting_date).days

        pixels_per_day = 3 if days > 120 else 6 if days > 90 else 8
        h_axis_length: int = days * pixels_per_day

        # cartesian axis metrics
        axis_origin = Coord(offset.x + v_label_width, offset.y + v_axis_len)

        # utility functions

        mk_tags = get_tag_builder("axis")

        def v_line(origin: Coord, length_,
                   width_=axis_line_width,
                   color=axis_color,
                   tags: Union[str, Tuple] = None,
                   dash: Tuple[int, int] = None):
            return self._add_line(origin, origin.with_y_offset(length_), width_, color, mk_tags(tags), dash=dash)

        def h_line(origin: Coord, length_,
                   width=axis_line_width,
                   color=axis_color,
                   tags: Union[str, Tuple] = None,
                   dash: Tuple[int, int] = None):
            return self._add_line(origin, origin.with_x_offset(length_), width, color, mk_tags(tags), dash=dash)

        def text(origin: Coord, label, anchor=CENTER, justify=CENTER, font=axis_label_font, tags=None):
            return self._add_text(origin, label,
                                  anchor=anchor,
                                  justify=justify,
                                  font=font,
                                  tags=mk_tags(tags))

        #######################################################################
        #
        # Title
        #
        #######################################################################

        title_font = make_font(14)
        title_point = axis_origin.with_offset(int(h_axis_length / 2), -v_axis_len - 5)
        text(title_point, label=title, anchor=S, font=title_font)

        #######################################################################
        #
        # vertical axis
        #
        #######################################################################

        v_line(axis_origin, -v_axis_len, tags="v_axis")

        def h_tick(degree_count_, length_, label_=None, width_=1, color_="light gray", dash_=None, tags=None):
            origin_ = axis_origin.with_y_offset(-v_axis_len + degree_count_ * self._pixels_per_degree)
            h_line(origin_, length_, tags=tags)
            if label_:
                text(origin_.with_offset(-5, -2), label=label_, anchor=E, justify=RIGHT, font=axis_tick_font, tags=tags)
            h_line(origin_, h_axis_length, width=width_, color=color_, dash=dash_, tags=tags)

        temperatures = reversed(list(range(self._low_temp, self._high_temp)))
        for degree_count, degree in enumerate(temperatures):
            degree += 1
            if 0 == (degree % 10):
                h_tick(degree_count, 5, label_=str(degree), tags=("v_axis_label", "v_axis_tick"))
            elif 0 == (degree_count % 5):
                h_tick(degree_count, 3, dash_=(4, 4), tags=("v_axis_label", "v_axis_minor_tick"))

        vertical_label_text = "\n".join(list("Temperatures")) + "\n\n(F" + u"\N{DEGREE SIGN}" + ")"
        text(axis_origin.with_offset(-v_label_width, -int(v_axis_len / 2)),
             label=vertical_label_text,
             tags="v_axis_label")

        #######################################################################
        #
        # horizontal axis
        #
        #######################################################################

        h_line(axis_origin, h_axis_length, tags="h_axis")
        self._data_points = DailyTemperatureDataPointDict()

        def v_tick(origin_, length_, label_, font=axis_tick_font, tags=None):
            v_line(origin_.with_y_offset(-length_), length_, tags=tags)
            text(origin_.with_y_offset(2), label=label_, anchor=N, font=font, tags=tags)

        minor_tick_font = make_font(axis_tick_font.cget("size") - 2)
        tick_origin = axis_origin
        for axis_date in [self._starting_date + timedelta(days=d) for d in range(0, days + 1)]:
            if 1 == axis_date.day:
                v_tick(tick_origin, 5, month_name(axis_date.month), tags=("h_axis_label", "h_axis_tick"))
            elif 15 == axis_date.day:
                v_tick(tick_origin, 3, "15th", font=minor_tick_font, tags=("h_axis_label", "h_axis_minor_tick"))
            elif 2 == axis_date.month and 29 == axis_date.day:
                # skip leap year dates
                continue
            self._data_points[axis_date] = tick_origin
            tick_origin = tick_origin.with_x_offset(pixels_per_day)

        #######################################################################
        #
        # plot line metrics
        #
        #######################################################################

        self._plot_count = 0
        self._current_history_highlight = ""
        self._default_line_width = 1
        self._highlight_line_width = 3

        #######################################################################
        #
        # legend area
        #
        #######################################################################

        def date_next_month(_date: date) -> date:
            return date(_date.year, _date.month, monthrange(_date.year, _date.month)[1]) + timedelta(days=1)

        # figure out how many months are being plotted
        axis_date = self._starting_date
        axis_months = 0
        while True:
            axis_months += 1
            axis_date = date_next_month(axis_date)
            if self._ending_date < axis_date:
                break

        axis_date = self._starting_date
        months = int(axis_months / 2)
        while 0 < months:
            axis_date = date_next_month(axis_date)
            months -= 1
        if axis_months % 2:
            axis_date = axis_date + timedelta(days=14)
        axis_center_coord = self._data_points[axis_date]

        self._legend_origin = axis_center_coord.with_y_offset(h_label_height)
        self._legend_label_font = make_font(7)
        self._legend_label_height = axis_label_font.metrics("linespace")

        #######################################################################
        #
        # scroll bars
        #
        #######################################################################

        horizontal_scrollbar = Scrollbar(self, orient=HORIZONTAL)
        horizontal_scrollbar.grid(row=1, column=0, sticky=EW)
        horizontal_scrollbar.config(command=self._canvas.xview)

        vertical_scrollbar = Scrollbar(self, orient=VERTICAL)
        vertical_scrollbar.grid(row=0, column=1, sticky=NS)
        vertical_scrollbar.config(command=self._canvas.yview)

        bbox = self._canvas.bbox(ALL)
        self._canvas.config(xscrollcommand=horizontal_scrollbar.set,
                            yscrollcommand=vertical_scrollbar.set,
                            scrollregion=(0, 0, bbox[2], bbox[3]))
        self._canvas.yview_moveto(1.0)

        self._canvas.bind_all("<MouseWheel>", lambda e: self._canvas.yview_scroll(-1 * (e.delta // 120), "units"))

    @property
    def plot_count(self):
        return self._plot_count

    def _add_line(self, lhs: Coord, rhs: Coord,
                  width: int = 1,
                  color: str = "black",
                  tags: Tuple = None,
                  dash: Tuple = None):
        ident = self._canvas.create_line(lhs.x, lhs.y, rhs.x, rhs.y,
                                         width=width,
                                         fill=color,
                                         tags=tags,
                                         dash=dash)
        log.debug("line: {}, ident: {}, coords={}".format(tags, ident, (lhs, rhs)))
        return ident

    def _add_rectangle(self, upper_left: Coord, lower_right: Coord,
                       width: float = 1.0,
                       outline: str = "black",
                       fill: str = "",
                       tags: Tuple = None):
        ident = self._canvas.create_rectangle(upper_left.x, upper_left.y, lower_right.x, lower_right.y,
                                              width=width,
                                              outline=outline,
                                              tags=tags,
                                              fill=fill)
        log.debug("rectangle: {}, ident: {}, bounds={}".format(tags, ident, (upper_left, lower_right)))
        return ident

    def _add_text(self, origin: Coord, text: str, anchor=CENTER, justify=CENTER, font: Font = None, tags=None):
        ident = self._canvas.create_text(origin.x, origin.y, text=text,
                                         anchor=anchor,
                                         justify=justify,
                                         font=font if font else nametofont("TkDefaultFont"),
                                         tags=tags)
        log.debug("text: {}, ident: {}, coord={}, anchor: {}, justify: {}".format(tags,
                                                                                  ident,
                                                                                  origin,
                                                                                  anchor,
                                                                                  justify))
        return ident

    def plot(self, daily_samples: Sequence[DailyTemperature], color: str, label: str):

        # sanitize the plot samples

        starting = date(MAXYEAR, 12, 31)
        ending = date(MINYEAR, 1, 3)
        low_t_bounds, high_t_bounds = float(self._low_temp), float(self._high_temp)
        plot_data: List[Tuple[Coord, Optional[float], Optional[float]]] = []
        for data in daily_samples:
            label_origin = self._data_points.get(data.ts)
            if label_origin:
                starting = min(starting, data.ts)
                ending = max(ending, data.ts)
                if data.low:
                    low_t = low_t_bounds if data.low < low_t_bounds else min(data.low, high_t_bounds)
                    low_t -= low_t_bounds
                else:
                    low_t = None
                if data.high:
                    high_t = high_t_bounds if data.high > high_t_bounds else max(data.high, low_t_bounds)
                    high_t -= low_t_bounds
                else:
                    high_t = None
                plot_data.append((label_origin, low_t, high_t))
            else:
                log.warning("ignoring: {}".format(data))

        # draw the samples

        self._plot_count += 1
        data_tags = get_tag_builder("plot_data_" + str(self._plot_count))
        point_tag = "plot_data_point_" + str(self._plot_count)
        line_tag = "plot_data_line_" + str(self._plot_count)

        def draw_sample(p1, p2):
            self._canvas.create_oval(p1.x - 1, p1.y - 1, p1.x + 1, p1.y + 1,
                                     fill=color,
                                     tags=data_tags(point_tag))
            if p2:
                self._add_line(p2, p1,
                               color=color,
                               tags=data_tags(line_tag))
            return p1

        previous_low_coord = None
        previous_high_coord = None
        for label_origin, low_t, high_t in plot_data:
            if not low_t:
                previous_low_coord = None
            else:
                coord = label_origin.with_y_offset(-round(low_t * self._pixels_per_degree))
                previous_low_coord = draw_sample(coord, previous_low_coord)
            if not high_t:
                previous_high_coord = None
            else:
                coord = label_origin.with_y_offset(-round(high_t * self._pixels_per_degree))
                previous_high_coord = draw_sample(coord, previous_high_coord)

        legend_tags = get_tag_builder("legend")
        movable_legend_tag = "legend_labels"
        legend_tag_template = "legend_label_{}"

        label_width_overall = 80
        label_box_width = 60
        label_x_separation = 10
        label_y_separation = 5

        def add_label(_origin: Coord):
            _rectangle_offset = int(label_box_width / 2)
            _legend_tag = legend_tag_template.format(self._plot_count)
            self._add_rectangle(_origin.with_x_offset(-_rectangle_offset),
                                _origin.with_offset(_rectangle_offset, self._legend_label_height),
                                outline=color,
                                fill=color,
                                tags=legend_tags(movable_legend_tag, _legend_tag))
            self._canvas.tag_bind(_legend_tag, "<1>", lambda e: self.highlight_history(line_tag))
            _origin = _origin.with_y_offset(label_y_separation + self._legend_label_height)
            self._add_text(_origin, label,
                           anchor=N,
                           font=self._legend_label_font,
                           tags=legend_tags(movable_legend_tag, _legend_tag))

        if 1 == self._plot_count:

            # add_label(self._legend_metrics.origin)
            add_label(self._legend_origin)

            # since this is the first label adjust the scrolling region to include the legend
            bbox = self._canvas.bbox(ALL)
            self._canvas.config(scrollregion=(0, 0, bbox[2], bbox[3] + 15))

            return

        # calculate the x bounds of the plot labels that are on the canvas
        x_bounds = int((self._plot_count - 1) * label_width_overall / 2)
        lhs_x = self._legend_origin.x - x_bounds
        rhs_x = self._legend_origin.x + x_bounds

        # add the new plot label
        label_offset = rhs_x + label_x_separation + int(label_width_overall / 2)
        add_label(self._legend_origin.with_x(label_offset))

        # adjust the rhs bounds to include the label just added
        rhs_x += label_x_separation + label_width_overall

        # now move the labels to center
        current_center_x = lhs_x + int((rhs_x - lhs_x) / 2)
        delta_x = self._legend_origin.x - current_center_x
        for item in self._canvas.find_withtag(movable_legend_tag):
            self._canvas.move(item, delta_x, 0)

    def highlight_history(self, plot_line_tag: str):
        if 1 < self._plot_count:
            if self._current_history_highlight:
                self._canvas.itemconfig(self._current_history_highlight, width=1)
            if plot_line_tag == self._current_history_highlight:
                self._current_history_highlight = ""
            else:
                self._canvas.itemconfig(plot_line_tag, width=2)
                self._current_history_highlight = plot_line_tag


class ProgressWidget:

    def __init__(self, master: Frame, maximum: int, description="in-progress", length=100):
        if len(master.grid_slaves()):
            raise RuntimeError("Progress frame is not empty.")
        Label(master, text=description).grid(row=0, column=0, sticky=(W, S), padx=5)
        self._progress_bar = Progressbar(master=master,
                                         length=length,
                                         orient="horizontal",
                                         mode="determinate",
                                         maximum=maximum)
        self._progress_bar.grid(row=0, column=1, sticky=E)

    def set_maximum(self, maximum: int):
        self._progress_bar.configure(maximum=maximum)

    def step(self):
        self._progress_bar.step()
        self._progress_bar.master.update()

    def end(self):
        for slave in self._progress_bar.master.grid_slaves():
            slave.grid_forget()
            slave.destroy()


class DailyWeatherWidget(Frame):
    _headings: Tuple[Tuple[DailyWeatherContent, str]] = (
        (DailyWeatherContent.TIME, "Date"),
        (DailyWeatherContent.TEMPERATURE_HIGH, "Daytime\nHigh"),
        (DailyWeatherContent.TEMPERATURE_HIGH_TIME, "Daytime\nHigh TOD"),
        (DailyWeatherContent.TEMPERATURE_LOW, "Overnight\nLow"),
        (DailyWeatherContent.TEMPERATURE_LOW_TIME, "Overnight\nLow TOD"),
        (DailyWeatherContent.TEMPERATURE_MAX, "Daily\nHigh"),
        (DailyWeatherContent.TEMPERATURE_MAX_TIME, "Daily\nHigh TOD"),
        (DailyWeatherContent.TEMPERATURE_MIN, "Daily\nLow"),
        (DailyWeatherContent.TEMPERATURE_MIN_TIME, "Daily\nLow TOD"),
        (DailyWeatherContent.WIND_SPEED, "Wind Speed"),
        (DailyWeatherContent.WIND_GUST, "Wind Gust"),
        (DailyWeatherContent.WIND_GUST_TIME, "Wind Gust\nTOD"),
        (DailyWeatherContent.WIND_BEARING, "Wind Bearing"),
        (DailyWeatherContent.CLOUD_COVER, "Cloud Cover")
    )

    def __init__(self, master, content_selection: List[DailyWeatherContent], **kw):
        super().__init__(master=master, **kw)
        self.columnconfigure(0, weight=1)
        self.rowconfigure(0, weight=1)

        content_order: List[DailyWeatherContent] = []
        headings: List[str] = []
        for heading, heading_value in self._headings:
            if heading in content_selection:
                content_order.append(heading)
                headings.append(heading_value)
        self.content_order = tuple(content_order)

        default_font = nametofont("TkDefaultFont")
        font_config = nametofont("TkDefaultFont").config()
        font = (font_config["family"], font_config["size"], "normal")
        header_font = (font_config["family"], font_config["size"], "bold")

        header_height = (default_font.metrics("linespace") + 3) * 2

        self._sheet = Sheet(self,
                            show_top_left=False,
                            show_row_index=False,
                            align="center",
                            font=font,
                            header_font=header_font,
                            popup_menu_font=font,
                            headers=headings,
                            header_height=header_height,
                            set_all_heights_and_widths=True)
        self._sheet.set_all_column_widths()
        self._sheet.grid(row=0, column=0, sticky=(N, S, E, W))

    def load(self, content: List[List[Any]]):
        for row in content:
            self._sheet.insert_row(values=row)
        self._sheet.set_all_cell_sizes_to_text()


class LocationsWidget(Frame):
    class Column(NamedTuple):
        id: str
        text: str
        anchor: str
        min_chars: int
        default_chars: int
        stretch: str

    columns: Tuple[Column] = (
        Column("#0", "Name", CENTER, 10, 30, YES),
        Column("alias", "Alias", CENTER, 10, 15, NO),
        Column("long", "Longitude", CENTER, 10, 15, NO),
        Column("lat", "Latitude", CENTER, 10, 15, NO),
        Column("tz", "Timezone", CENTER, 10, 15, NO),
        Column("hist", "Weather History", W, 10, 20, YES),
    )

    def __init__(self, master, **kwargs):
        super().__init__(master, **kwargs)
        self.columnconfigure(0, weight=1)
        self.rowconfigure(0, weight=1)

        scrollbar = Scrollbar(self)
        scrollbar.grid(row=0, column=1, sticky=(N, S, E))

        ids = [cid.id for cid in self.columns[1:]]
        self._tree = tree = Treeview(self, columns=ids, yscrollcommand=scrollbar.set)
        tree.grid(row=0, column=0, sticky=NSEW)

        scrollbar.config(command=tree.yview)

        self._font = nametofont(Style().lookup(tree.winfo_class(), "font"))

        for column in self.columns:
            tree.heading(column.id, text=column.text, anchor=column.anchor)
            min_width = self.measure('0' * column.min_chars)
            default_width = self.measure('0' * column.default_chars)
            tree.column(column.id, minwidth=min_width, width=default_width, stretch=column.stretch)

    def clear(self):
        tree = self._tree
        for item in tree.get_children():
            log.debug("clearing {}".format(item))
            tree.delete(item)

    def set_column_chars(self, cid: str, char_count: int):
        width = self.measure("0" * (char_count + 2))
        self._tree.column(cid, width=width, minwidth=width)

    def add_location(self, location) -> str:
        log.debug("Adding %s", location.name)
        values = (location.alias, location.longitude, location.latitude, location.tz)
        return self._tree.insert("", "end", iid=location.name, text=location.name, values=values)

    def add_history(self, parent: str, date_range: DateRange) -> str:
        if 1 < date_range.total_days():
            history = "{} to {}".format(date_range.low, date_range.high)
        else:
            history = str(date_range.low)
        return self._tree.insert(parent, "end", values=("", "", "", "", history))

    def measure(self, text: str) -> int:
        return self._font.measure(text)

    def on_event(self, event=None, function=None):
        self._tree.bind(event, function)

    def get_selection(self):
        return self._tree.selection()


class NotebookWidget(Frame):

    def __init__(self, master, **kwargs):
        super().__init__(master, **kwargs)
        self.columnconfigure(0, weight=1)
        self.rowconfigure(0, weight=1)

        self._notebook = Notebook(master=self, **kwargs)
        self._notebook.grid(sticky=NSEW)

        def popup():
            selected_tab = self._notebook.select()
            if 0 < self._notebook.index(selected_tab):
                try:
                    xy = Coord(self.winfo_pointerx() - self.winfo_vrootx(), self.winfo_pointery() - self.winfo_vrooty())
                    popup_menu.tk_popup(*xy)
                finally:
                    popup_menu.grab_release()
        popup_menu = Menu(self._notebook, tearoff=0)
        popup_menu.add_command(label="Delete", command=self.remove_tab)
        self._notebook.bind('<Button-3>', lambda e: popup())
        self._notebook.bind('<Delete>', lambda e: self.remove_tab())

    def add_tab(self, widget, tab_name: str):
        self._notebook.add(widget, text=tab_name)
        tabs = self._notebook.index('end')
        self._notebook.select(tabs - 1)
        self.master.update_idletasks()

    def bind_notebook_tab_changed(self, tab_changed_function: Callable):
        self._notebook.bind("<<NotebookTabChanged>>", tab_changed_function)

    def remove_tab(self):
        # ignore the locations tab
        selected_tab = self._notebook.select()
        if 0 < self._notebook.index(selected_tab):
            prompt = "Remove {}?".format(self._notebook.tab(selected_tab, 'text'))
            if messagebox.askyesno(title="Remove Tab", message=prompt) == YES:
                self._notebook.forget(selected_tab)


class StatusWidget(Frame):

    def __init__(self, master, **kw):
        super().__init__(master=master, **kw)

        self.columnconfigure(0, weight=1)
        self.columnconfigure(1, weight=1)

        self._weather_data_dir = StringVar()
        lhs = Frame(self)
        lhs.grid(row=0, column=0, sticky=W)
        lhs.columnconfigure(0, weight=1)
        lhs.columnconfigure(1, weight=1)
        Label(master=lhs, text="Weather Folder: ").grid(row=0, column=0, sticky=W)
        Label(master=lhs, textvariable=self._weather_data_dir).grid(row=0, column=1, sticky=W)

        self._progress_frame = Frame(self)
        self._progress_frame.grid(row=0, column=1, sticky=E)
        self._progress_frame.columnconfigure(0, weight=1)
        self._progress_frame.columnconfigure(1, weight=1)

        Sizegrip(master=self).grid(row=0, column=2, sticky=SE)

    def set_weather_data_dir(self, weather_data: WeatherData):
        self._weather_data_dir.set(weather_data.data_path() if weather_data else "None")

    def create_progress_widget(self, description: str, maximum: int) -> ProgressWidget:
        widget = ProgressWidget(self._progress_frame, maximum=maximum, description=description)
        self.update_idletasks()
        return widget
