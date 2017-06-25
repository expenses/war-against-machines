# War Against Machines

[![Build Status](https://travis-ci.org/expenses/war-against-machines.svg?branch=master)](https://travis-ci.org/expenses/war-against-machines)
[![Build status](https://ci.appveyor.com/api/projects/status/80a2soj85wglon1x?svg=true)](https://ci.appveyor.com/project/expenses/war-against-machines)

A (very WIP) turn-based strategy game set in the not-so-distant future.

![A screenshot of the game taken 06/26/17](readme/screenshot-06-26-17.png)

Written in [Rust](https://www.rust-lang.org).

## Download

Releases can be found on the [releases page](https://github.com/expenses/war-against-machines/releases).

Not that as these builds are built by [CIs](https://en.wikipedia.org/wiki/Continuous_integration), I may not have personally tested them.

## Building

Building requires installing the [SDL2 development libraries](https://github.com/AngryLawyer/rust-sdl2#sdl20-development-libraries).

After those are installed, you can build the release with Cargo:

    cargo build --release

## Documentation

To render and open the documentation, run:

    cargo doc --no-deps --open

## Gameplay

### Controls

Menu:
* `up`/`w` to move the selection up
* `down`/`s` to move the selection down
* `enter` to activate the selected item
* `left`/`a` to lower the value of the selected item
* `right`/`d` to raise the value of the selected item
* `escape` to quit

In a battle:
* `up`/`w` to move the camera up
* `down`/`s` to move the camera down
* `left`/`a` to move the camera left
* `right`/`d` to move the camera right
* `o` to zoom out
* `p` to zoom in
* `escape` to quit
* `left mouse button` to select the unit under the cursor
* `right mouse button` to perfom commands such as moving and firing