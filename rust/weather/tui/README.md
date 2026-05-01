# The TUI Library
The TUI library provides a collection of UI widgets for terminal based applications. The 
library is built on top of the `ratatui` and `crossterm` crates.


## Background
The original TUI library was built with the intent it would be used to build a standalone
application. It worked well for the weather TUI however there is a lot of baggage to carry
when building a simple inline screen.

## TUI Components
The library consists of a `ratatui` viewport runner and UI widgets.

### Viewport Runner
The viewport runner manages initialization and restoration of the terminal.
It creates an inline viewport on the screen where the UI is rendered. The
runner calls a user defined function, passing it a `Terminal` instance, to run the UI.

### UI Widgets
All widgets have similar characteristics.
- they have a render method that updates the screen with its contents.
- they have a companion structure of `ratatui` styles that define their appearance on the screen.

#### CommandBar Widget
This widget displays a list of the key bindings supported by the application. It does
not accept key or mouse events.

#### Label Widget
This widget displays text on the screen. Optionally it accepts a selector character
that will underline the first character match in the label text. It does not accept key
or mouse events.

#### Editor Widget
This widget manages an area on the screen to add, update, or delete text. It
optionally contains a *Label* widget to help describe what the text contents are.
It accepts key pressed events but does not accept mouse events.

#### EditorGroup Widget
This widget manages a collection of editors rendered vertically on the screen. Only
one editor in the collection will be considered active allowing it to receive key pressed
events. It accepts key pressed events and left mouse button down events.
