images = $(shell find resources/images -name '*.png')
optipng=optipng -quiet

tileset = make/tileset/target/release/tileset

# Build the tileset
tileset: resources/tileset.png

# Run the colour conversion script
convert_colour: target/release/convert_colour
	$^

# Check the build on both stable and nightly (with clippy)
check:
	cargo +stable check && cargo +nightly clippy && cargo +nightly test

# Test shaders
shaders:
	cargo test compile_shaders

# Compile Rust source code
$(tileset): make/tileset/src/main.rs
	#rustc -L target/release/deps -O $^ -o $@
	cd ./make/tileset && cargo build --release


# Build the tileset
resources/tileset.png: $(tileset) $(images)
	# Run the tileset script
	$< resources/images $@
	# Optimise the tileset image
	$(optipng) $@	