"""
Pack all the seperate images into one big tileset
Requires numpy (pip install numpy) and skimage (pip install scikit-image)
"""

import os
from skimage import io
import numpy as np


TILE_SIZE = 48
DIRECTORY = "tiles"
FILES = [
    ["base/1.png", "base/2.png", "base/fog.png", "pit/top.png", "pit/left.png", "pit/right.png", "pit/bottom.png", "pit/center.png"],
    ["ruin/1.png", "ruin/2.png", "ruin/3.png", "pit/tl.png", "pit/tr.png", "pit/bl.png", "pit/br.png"],
    ["unit/squaddie.png", "unit/machine.png"],
    ["bullet/regular.png", "bullet/plasma.png"],
    ["item/squaddie_corpse.png", "item/machine_corpse.png", "item/skeleton.png", "item/scrap.png", "item/weapon.png"],
    ["cursor/default.png", "cursor/unit.png", "cursor/unwalkable.png", "cursor/crosshair.png"],
    ["path/default.png", "path/no_weapon.png", "path/unreachable.png"],
    ["edge/left.png", "edge/right.png"],
    ["title.png"],
    ["button/end_turn.png", "button/inventory.png", "button/change_fire_mode.png"],
    ["font.png"]
]

def insert(image, x, height):
    """ Insert an image into the output image """
    output[
        height: height + image.shape[0],
        x * TILE_SIZE: x * TILE_SIZE + image.shape[1],
    ] = image

output = np.zeros((TILE_SIZE * 10, TILE_SIZE * 10, 4), np.uint8)
height = 0

for y, row in enumerate(FILES):
    row = [io.imread(os.path.join(DIRECTORY, image)) for image in row]

    for x, image in enumerate(row):
        insert(image, x, height)
    height += row[0].shape[0]

io.imsave("tileset.png", output)
