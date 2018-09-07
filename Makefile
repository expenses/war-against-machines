images = $(shell find resources/images -name '*.png')
optipng=optipng -quiet

tileset = make/tileset/target/release/tileset

# Build the tileset
tileset: resources/tileset.png

# Check the build on both stable and nightly (with clippy)
# Useful for git pre-commit hooks mostly
check:
	cargo +stable check && cargo +nightly clippy && cargo +nightly test

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