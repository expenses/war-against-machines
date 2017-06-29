# War Against Machines

[![Build Status](https://travis-ci.org/expenses/war-against-machines.svg?branch=master)](https://travis-ci.org/expenses/war-against-machines)
[![Build status](https://ci.appveyor.com/api/projects/status/80a2soj85wglon1x?svg=true)](https://ci.appveyor.com/project/expenses/war-against-machines)
[![LoC](https://tokei.rs/b1/github/expenses/war-against-machines)](https://github.com/Aaronepower/tokei)

A (very WIP) turn-based strategy game set in the not-so-distant future.

![A screenshot of the game taken 29/06/17](readme/screenshot-29-06-17.png)

Written in [Rust](https://www.rust-lang.org).

## Download

Releases can be found on the [releases page](https://github.com/expenses/war-against-machines/releases).

Note that as these releases are build automatically, I may not have personally tested them.

## Building

Building requires installing the [SDL2 development libraries](https://github.com/AngryLawyer/rust-sdl2#sdl20-development-libraries).

After those are installed, you can build the release with Cargo:

    cargo build --release

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