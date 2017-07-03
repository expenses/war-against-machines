// Struct constants

use graphics::Text;
use graphics::types::Color;

pub const WHITE: Color = [1.0, 1.0, 1.0, 1.0];
pub const OFF_WHITE: Color = [0.8, 0.8, 0.8, 1.0];
pub const BLACK: Color = [0.0, 0.0, 0.0, 1.0];

pub const REGULAR: Text = Text {
    color: WHITE,
    font_size: 20,
    round: false
};