from enum import StrEnum, auto

from textual.app import ComposeResult, on
from textual.containers import Center, Container
from textual.screen import ModalScreen
from textual.widgets import Button, Label


class MessageBox(ModalScreen):
    class Type(StrEnum):
        INFO = auto()
        WARNING = auto()
        ERROR = auto()

    CSS_PATH = "message_box.tcss"

    def __init__(self, message: str, type: Type = Type.INFO):
        super().__init__()
        self._message = message
        self._type = type

    def compose(self) -> ComposeResult:
        dialog = Container(id=str(self._type))
        match self._type:
            case self.Type.WARNING:
                dialog.border_title = " Warning "
            case self.Type.ERROR:
                dialog.border_title = " Error "
        with dialog:
            yield Center(Label(self._message))
            yield Center(Button("Ok", compact=True))

    @on(Button.Pressed)
    def _dismiss(self):
        self.dismiss()
