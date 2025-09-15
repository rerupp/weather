# termui lib

The `termui` library provides terminal based UI components that can be used to build TUI
applications. The library relies on the `ratatui` and `crossterm` crates.

All library components sit on top of `ratatui` widgets. This will probably change in the future 
however right now it was the fastest way to get the weather data TUI up and running.

## Library Features
The library includes several macros that have been helpful to debug issues while developing the 
TUI.

The features can be activated using the cargo `--features` option. The following command line 
activates both features.

```cargo build --features log_key_event,log_render```

### The `log_key_event` macro.
This macro typically sits at the top of a `key_pressed()` method. When the method is called a *key 
pressed* `DEBUG` message is written to the log file. When the method exits a `DEBUG` message 
with the elapsed execution time is written to the log file.

### The `log_render` macro.
This macro typically sits at the top of a `render()` method. When the method is called a *render* 
`DEBUG` message is written to the log file. When the method exits a `DEBUG` message with the 
elapsed execution time is written to the log file.

## Module Overview

The components are divided into the following modules.

### The `console` Module

This module provides the runtime environment for the terminal UI. The `Application` trait
is the API users implement to run the application. The `Console` structure is the 
application runner. It manages the setup and teardown of the `crossterm` backend. It also 
dispatches key and render events. 

### The `controls` Module

This module contains basic controls such as *label*, *button*, *checkbox*, *report* viewer, and 
*text* editors. There are currently 2 *text* editors. One is specialized for date entry and the 
other for *text* editing. The *text* editor supports limiting input to a set of characters such as 
numbers or characters,

### The `dialogs` Module

This module provides TUI based dialogs. The following dialogs are currently available.

- A modal button dialog consisting of a button bar and a dialog widow. The dialog manages 
  dispatching key events and render events to the button bar and dialog window.
- A menu dialog consisting of a menu and a dialog widow. The dialog manages 
  dispatching key events and render events to the menu and dialog window.
- A modal message dialog used to display a message. Messages can be defined as error, warning, 
  or informational.
- A progress dialog showing an indicator that some action is happening.
- A tabbed dialog used to display multiple windows within a tab like environment.

### The `menus` Module

This module provides TUI based menus. The following menus are currently available.

- A dropdown menu used to provide a collection of cascading menus.
- A popup menu used for context menus.
- A menubar that manages a collection of menu items.

### The `styles` Module

This module provides the styling used to render `ratatui` widgets. The intent is at some point 
styling will be externalized and customizable. That's not the state today, it is all hardcoded 
but the infrastructure is mostly there.

- The `bootstrap` module provides the hardcoded styling defaults.
- The `persistence` module provides the persistent data objects (PDO) that can be used to 
  serialize and deserialize the style catalogs.
- The `store` module provides the runtime accessible style catalogs. The runtime is currently a 
  singleton built on top of `OnceLock`.
