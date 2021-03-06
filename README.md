# War Against Machines

[![Build Status](https://travis-ci.org/expenses/war-against-machines.svg?branch=master)](https://travis-ci.org/expenses/war-against-machines)
[![Build status](https://ci.appveyor.com/api/projects/status/80a2soj85wglon1x?svg=true)](https://ci.appveyor.com/project/expenses/war-against-machines)
[![LoC](https://tokei.rs/b1/github/expenses/war-against-machines)](https://github.com/Aaronepower/tokei)

A (very WIP) turn-based strategy game set in the not-so-distant future.

![A screenshot of the game taken 25/07/17](readme/screenshot-26-12-17.png)

Written in [Rust](https://www.rust-lang.org).

## Building

The game can be built with:

    cargo build --release

If you get an error like:

    thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: "`"pkg-config" "--libs" "--cflags" "alsa"` did not exit successfully: exit code: 1

This means that you will need to install the ALSA development libraries:

    sudo apt-get install libasound2-dev

## Gameplay

### Controls

On a menu:
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
* `e` to use items on the inventory screen
* `escape` to quit
* `lmb` for performing actions (selecting/moving/firing)