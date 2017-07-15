extern crate image;

// Build the tileset
// This is run through the Makefile

use image::{GenericImage, DynamicImage};
use std::path::Path;
use std::env::args;

const SIZE: u32 = 480;
const FILES: &[&[&str]] = &[
    &["base/1.png", "base/2.png", "ruin/1.png", "pit/top.png", "pit/left.png", "pit/right.png", "pit/bottom.png", "pit/center.png"],
    &["ruin/2.png", "ruin/3.png", "ruin/4.png", "pit/tl.png", "pit/tr.png", "pit/bl.png", "pit/br.png"],
    &["unit/squaddie.png", "unit/machine.png"],
    &["bullet/regular.png", "bullet/plasma.png"],
    &["item/squaddie_corpse.png", "item/machine_corpse.png", "item/scrap.png", "item/weapon.png"],
    &["cursor/default.png", "cursor/unit.png", "cursor/unwalkable.png", "cursor/crosshair.png"],
    &["path/default.png", "path/no_weapon.png", "path/unreachable.png"],
    &["decoration/left_edge.png", "decoration/right_edge.png", "decoration/skeleton.png", "decoration/rubble.png"],
    &["title.png"],
    &["button/end_turn.png", "button/inventory.png", "button/change_fire_mode.png", "button/save_game.png"],
    &["font.png"]
];

fn main() {
    // Get the directory and output file as arguments
    let mut args = args().skip(1);
    let directory = args.next().unwrap();
    let output = args.next().unwrap();

    // Create the image to copy into
    let mut base = DynamicImage::new_rgba8(SIZE, SIZE).to_rgba();
    // Create a path from the directory string
    let path = Path::new(&directory);
    let mut y = 0;

    // Loop through rows
    for row in FILES.iter() {
        let mut height = 0;
        let mut x = 0;

        // Loop through images
        for image in row.iter() {
            // Load the image
            let image = image::open(path.join(image)).unwrap();
            // Copy the image into the base at the right position
            base.copy_from(&image, x, y);
            
            // Change the x value
            x += image.width();
            // If the height of the image is greater than the current height, set the height
            if image.height() > height { height = image.height(); }
        }
        // Change the y value
        y += height;
    }
    
    // Strip out the value of the transparent pixels (this helps to optimize the image)
    for pixel in base.pixels_mut() {
        if pixel[3] == 0 {
            pixel[0] = 0;
            pixel[1] = 0;
            pixel[2] = 0;
        }
    }

    // Save the tileset
    base.save(output).unwrap();
}
