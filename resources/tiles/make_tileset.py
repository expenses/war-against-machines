from skimage import io
import numpy as np

TILE = 48
OUTPUT = np.zeros((TILE * 10, TILE * 10, 4), np.uint8)
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
    ["button/end_turn.png", "button/inventory.png", "button/change_fire_mode.png"]
]

def insert(image, x, y):
    OUTPUT[
        y * TILE: y * TILE + image.shape[0],
        x * TILE: x * TILE + image.shape[1],
    ] = image


for y, row in enumerate(FILES):
    for x, image in enumerate(row):
        insert(io.imread(image), x, y)

io.imsave("tileset.png", OUTPUT)