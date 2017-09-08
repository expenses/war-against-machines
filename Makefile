
# Build the tileset
tileset: resources/tileset.png

# Run the colour conversion script
convert_colour: target/release/convert_colour
	$^

# Check the build on both stable and nightly (with clippy)
check:
	cargo +stable check && cargo +nightly clippy

# Test shaders
shaders:
	cargo test compile_shaders

# Optimise the resource images
optimise:
	optipng `find resources/images -name "*.png"`

# Compile Rust source code
target/release/%: make/%.rs
	rustc -L target/release/deps -O $^ -o $@

# Build the tileset
resources/tileset.png: target/release/tileset resources/images
	# Run the tileset script
	$^ $@
	# Optimise the tileset image
	optipng $@
