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