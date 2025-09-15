# `toolslib` Library

This library contains a collection of utilities used by the various `Rust` projects. It probably 
belongs in a separate repository since it is shared between the `weather` and the 
`rust_playground` repository. Right now it is a copy what is in the current `rust_playground` 
repository.

## Background

The library was initially created when I started to learn `Rust`. I've kept the library around 
because the `rust_playground` has several utility programs that rely on its functionality. When 
the `weather` repository was created I simply made a copy of the library.

## Library Contents

A quick overview of the library utilities follows.

* The `date_time` module contains a collection of date and time utilities.
* The `fmt` module contains a collection of utilities that help format numbers.
* The `log` module contains utilities to bootstrap `log4rs`.
* The `report` module has the infrastructure used by the weather CLI to create text reports.
* The `stopwatch` module is what it sounds like.
* The `text` module is the first generation of the `report` module. It's still around because 
  there are utilities in the `rust_playground` that rely on it.

I would expect in an upcoming release the library will go through some major changes that reflect 
usage in the `weather` project.
