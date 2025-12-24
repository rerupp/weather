import logging as log
import math
import string
from dataclasses import dataclass
from enum import Enum, auto

from py_weather_lib import PyLocation
from textual import on
from textual.app import ComposeResult
from textual.containers import CenterMiddle, Horizontal, Right, VerticalGroup
from textual.css.query import NoMatches
from textual.message import Message
from textual.screen import ModalScreen
from textual.validation import Failure, ValidationResult, Validator
from textual.widgets import Button, Footer, Header, Input, Label


class CoordValidator(Validator):
    class Type(Enum):
        LATITUDE = auto(),
        LONGITUDE = auto()

    def __init__(self, validator_type: Type) -> None:
        super().__init__()
        self._validator_type = validator_type

    def validate(self, value: str) -> ValidationResult:
        what = "Latitude" if self._validator_type is self.Type.LATITUDE else "Longitude"
        # let someone else decide if it's okay to be empty or not
        if len(value) == 0:
            return ValidationResult.success()

        try:
            float_value = float(value)
        except ValueError:
            return ValidationResult.failure([Failure(self, value=value, description=f"{what} is not valid.")])

        if math.isnan(float_value) or math.isinf(float_value):
            return ValidationResult.failure([Failure(self, value=value, description=f"{what} is NaN.")])

        match self._validator_type:
            case self.Type.LATITUDE:
                if float_value < -90 or float_value > 90:
                    return ValidationResult.failure([
                        Failure(self, value=value, description=f"{what} range is -90 thru 90.")
                    ])
            case self.Type.LONGITUDE:
                if float_value < -180 or float_value > 180:
                    return ValidationResult.failure([
                        Failure(self, value=value, description=f"{what} range is -180 thru 180.")
                    ])
        return self.success()


class LocationProperty(Horizontal):
    @dataclass
    class Changed(Message):
        """Follow the textual Input widget model when changed."""
        property: 'LocationProperty'
        validation_result: ValidationResult | None = None

    @dataclass
    class Blurred(Message):
        """Follow the textual Input widget model when loosing focus."""
        property: 'LocationProperty'
        validation_result: ValidationResult | None = None

    LABEL_CLASS = "location-property-descr"
    INPUT_CLASS = "location-property-text"
    REQUIRED_CLASS = "location-property-required"
    READONLY_CLASS = "location-property-readonly"

    def __init__(self, id: str, label: str, value: str | None, placeholder: str, can_focus: bool,
                 validator: Validator | None = None, read_only: bool = False, required: bool = False):
        super().__init__(id=id)
        self._label_id = f"{id}-lbl"
        self._input_id = f"{id}-inp"
        self._label = label
        self._value = '' if value is None else value
        self._placeholder = placeholder
        self._read_only = read_only
        self._can_focus = can_focus
        self._validator = validator
        self._required = required

    @property
    def value(self) -> str:
        return self.query_one(Input).value

    def compose(self) -> ComposeResult:
        with Right():
            classes = self.LABEL_CLASS
            if self._can_focus and self._required and len(self._value) == 0:
                classes += f" {self.REQUIRED_CLASS}"
            yield Label(self._label, id=self._label_id, classes=classes)
        input = Input(id=self._input_id, value=self._value, compact=True, placeholder=self._placeholder,
                      validators=self._validator, select_on_focus=False)
        input.can_focus = self._can_focus
        if self._read_only:
            input.add_class(self.READONLY_CLASS)
        yield input

    @on(Input.Changed)
    def _on_change(self, event: Input.Changed):
        log.debug("LocationProperty _on_change: %s", event)
        # none of the ancestors should need to know about the change
        event.stop()
        # The CoordValidator will allow scientific notation so cleanse the new value
        if isinstance(self._validator, CoordValidator):
            value = ''
            for (index, c) in enumerate(event.value):
                if c in '-+':
                    if index == 0:
                        value += c
                    continue
                if c == '.':
                    if index == event.value.find('.'):
                        value += c
                    continue
                if c in string.digits:
                    value += c
        else:
            value = event.value

        if value == event.value:
            self.post_message(self.Changed(self, event.validation_result))
        else:
            # there was rejected input so use the cleansed result (which fires the validator again)
            log.debug("replacing %s with %s.", event.input.value, value)
            event.input.replace(value, 0, len(event.input.value))

    @on(Input.Blurred)
    def _blurred(self, event: Input.Blurred) -> None:
        # none of the ancestors should need to know about the input loosing focus
        event.stop()
        self.post_message(self.Blurred(self, event.validation_result))

    def set_required(self) -> None:
        if self._required:
            self.query_one(Label).add_class(self.REQUIRED_CLASS)

    def remove_required(self) -> None:
        if self._required:
            self.query_one(Label).remove_class(self.REQUIRED_CLASS)


class LocationProperties(VerticalGroup):
    DEFAULT_CSS = """
    LocationProperties {
        align: center middle;
        width: auto;
        height: auto;
        CenterMiddle {
            width: auto;
            height: auto;
            Horizontal {
                width: auto;
                height: 1;
            }
            Right {
                width: 10;
                height: auto;
            }
            Input {
                margin-left: 1;
                width: auto;
                height: 1;
            }
        }
    }
    """

    CITY_ID = "#location-properties-city"
    STATE_ID = "#location-properties-state"
    SHORT_STATE_ID = "#location-properties-short-state"
    ALIAS_ID = "#location-properties-alias"
    LATITUDE_ID = "#location-properties-latitude"
    LONGITUDE_ID = "#location-properties-longitude"
    TZ_ID = "#location-properties-tz"

    def __init__(self, location: PyLocation | None = None, changeable=False, alias_changeable=False,
                 id: str | None = None, classes: str | None = None):
        super().__init__(id=id, classes=classes)
        self.title = "Location Properties"
        self._location = location
        self._changeable = changeable
        self._alias_changeable = alias_changeable

    def compose(self) -> ComposeResult:
        log.debug("LocationProperties compose")
        changeable = self._changeable
        with CenterMiddle():
            location = self._location
            yield LocationProperty(id=self.CITY_ID[1:], label="City:", value=location.city if location else None,
                                   placeholder="the city name", can_focus=changeable, required=changeable)
            yield LocationProperty(id=self.STATE_ID[1:], label="State:", value=location.state if location else None,
                                   placeholder="state name", can_focus=changeable, required=changeable)
            # todo: upper case only?
            yield LocationProperty(id=self.SHORT_STATE_ID[1:], label="State ID:",
                                   value=location.state_id if location else None, placeholder="ID",
                                   can_focus=changeable, required=changeable)
            alias_changeable = changeable and self._alias_changeable
            yield LocationProperty(id=self.ALIAS_ID[1:], label="Alias:", value=location.alias if location else None,
                                   placeholder="alias name", can_focus=alias_changeable,
                                   read_only=not self._alias_changeable, required=alias_changeable)
            yield LocationProperty(id=self.LATITUDE_ID[1:], label="Latitude:",
                                   value=location.latitude if location else None,
                                   placeholder="latitude", can_focus=changeable, required=changeable,
                                   validator=CoordValidator(CoordValidator.Type.LATITUDE))
            yield LocationProperty(id=self.LONGITUDE_ID[1:], label="Longitude:",
                                   value=location.longitude if location else None, placeholder="longitude",
                                   can_focus=changeable, required=changeable,
                                   validator=CoordValidator(CoordValidator.Type.LONGITUDE))
            # todo: check the timezone
            yield LocationProperty(id=self.TZ_ID[1:], label="Timezone:", value=location.tz if location else None,
                                   placeholder="city timezone", can_focus=changeable, required=changeable)

    @property
    def location(self) -> PyLocation:
        return PyLocation(
            city=self.query_one(self.CITY_ID, LocationProperty).value,
            state=self.query_one(self.STATE_ID, LocationProperty).value,
            state_id=self.query_one(self.SHORT_STATE_ID, LocationProperty).value,
            alias=self.query_one(self.ALIAS_ID, LocationProperty).value,
            latitude=self.query_one(self.LATITUDE_ID, LocationProperty).value,
            longitude=self.query_one(self.LONGITUDE_ID, LocationProperty).value,
            tz=self.query_one(self.TZ_ID, LocationProperty).value
        )


class LocationEditor(ModalScreen[PyLocation | None]):
    CSS_PATH = "location_editor.tcss"
    BINDINGS = [
        ("escape", "dismiss()", "Cancel")
    ]
    LOCATION_EDITOR_ID = "#location-editor"
    OK_ID = '#ok'
    CANCEL_ID = "#cancel"

    def __init__(self, location: PyLocation | None = None, title="Change Location", editor_title="Location Properties"):
        super().__init__()
        self.title = title
        self._editor_title = editor_title
        self._location = location

    def compose(self) -> ComposeResult:
        yield Header()
        yield Footer()
        editor = CenterMiddle(id=self.LOCATION_EDITOR_ID[1:])
        editor.border_title = self._editor_title
        with editor:
            yield LocationProperties(location=self._location, changeable=True)
            with Horizontal(id="buttons"):
                yield Button("Ok", id=self.OK_ID[1:], compact=True, disabled=True)
                yield Button("Cancel", id=self.CANCEL_ID[1:], compact=True)

    @on(LocationProperty.Changed)
    def _property_changed(self, event: LocationProperty.Changed) -> None:
        event.stop()
        self._set_editor_state(event.property, event.validation_result)

    @on(LocationProperty.Blurred)
    def _blurred(self, event: LocationProperty.Blurred) -> None:
        event.stop()
        self._set_editor_state(event.property, event.validation_result)

    def _set_editor_state(self, location_property: LocationProperty, validation_result: ValidationResult) -> None:
        # set the dialog result
        if len(location_property.value) == 0:
            location_property.set_required()
            error_description = "A value is required."
            log.debug("_set_editor_state: %s is empty", location_property)
        elif not validation_result:
            location_property.remove_required()
            error_description = ''
            log.debug("_set_editor_state: %s not validated", location_property)
        elif validation_result.is_valid:
            location_property.remove_required()
            error_description = ''
            log.debug("_set_editor_state: %s is valid", location_property)
        else:
            location_property.set_required()
            error_description = "\n".join(validation_result.failure_descriptions)
            log.debug("_set_editor_state: %s invalid (%s)", location_property, error_description)
        self.query_one(self.LOCATION_EDITOR_ID).border_subtitle = error_description

        # check to see if the ok button can be active
        try:
            self.query_one(f".{LocationProperty.REQUIRED_CLASS}")
            self.query_one(self.OK_ID).disabled = True
        except NoMatches:
            self.query_one(self.OK_ID).disabled = False

    @on(Button.Pressed)
    def _pressed(self, event: Button.Pressed) -> None:
        if event.button.id == self.CANCEL_ID[1:]:
            self.dismiss(None)
        else:
            properties = self.query_one(f"{self.LOCATION_EDITOR_ID} LocationProperties", LocationProperties)
            self.dismiss(properties.location)
