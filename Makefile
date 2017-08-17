# Don't worry about the target being up to date or whatever
.PHONY: tileset

# Build the tileset file and then run it
tileset:
	# Build the tileset binary, linking to the release deps
	rustc -L target/release/deps -O resources/tileset.rs -o target/release/tileset
	# Run it
	target/release/tileset resources/images resources/tileset.png
	# Optimise the tileset with optipng
	optipng resources/tileset.png

# Run the colour conversion script
convert_colour:
	rustc resources/convert_colour.rs -o target/release/convert_colour
	target/release/convert_colour

# Test shaders
shaders:
	cargo test compile_shaders

# Optimise the resource images
optimise:
	optipng `find resources/images`

# Check the build on both stable and nightly (with clippy)
check:
	rustup run stable cargo check && cargo clippy