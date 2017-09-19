images = $(shell find resources/images -name '*.png')
optipng=optipng -quiet

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
optimise: $(images)
	@$(optipng) $^

# Compile Rust source code
target/release/%: make/%.rs
	rustc -L target/release/deps -O $^ -o $@


# Build the tileset
resources/tileset.png: target/release/tileset $(images) resources/images/glyphs.png
	# Run the tileset script
	$< resources/images $@
	# Optimise the tileset image
	$(optipng) $@	

# How many pixels at the top to crop
top=5
# The four corners of the rectangle to draw as a 'missing character' symbol
tl=324, $(top)
tr=327, $(top)
bl=324, 12
br=327, 12

# Render the glyphs in 'glyphs.txt' and draw in a 'missing character symbol'
resources/images/glyphs.png: resources/font/glyphs.txt
	convert -background none -fill white +antialias -font "TinyUnicode" -pointsize 16 \
	-draw "line $(tl) $(tr) line $(tr) $(br) line $(br) $(bl) line $(bl) $(tl)" \
	label:@$^ -crop x8+0+$(top) $@