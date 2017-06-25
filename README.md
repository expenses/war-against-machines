# War Against Machines

[![Build Status](https://travis-ci.com/expenses/war-against-machines.svg?token=xXFRB8sW6quEs4edwh57&branch=master)](https://travis-ci.com/expenses/war-against-machines)

## Building

Building requires installing the SDL2 development libraries.

### Debian

    sudo apt-get install libsdl2-dev libsdl2-image-dev libsdl2-ttf-dev

### OS X

    brew install sdl2 sdl2-image sdl2-ttf

Then build the release build with cargo:

    cargo build --release

## Documentation

To render and open the documentation, run:

    cargo doc --no-deps --open