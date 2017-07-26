// Colour conversion functions taken from
// http://entropymine.com/imageworsener/srgbformula/

use std::io::stdin;

fn srgb_to_linear(value: f32) -> f32 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(value: f32) -> f32 {
    if value <= 0.0031308 {
        value * 12.92
    } else {
        1.055 * value.powf(2.4) - 0.055
    }
}

fn main() {
    println!("sRGB to linear? Y/n:");

    let mut input = String::new();

    stdin().read_line(&mut input).unwrap();

    let to_linear = !input.to_lowercase().starts_with('n');
    
    println!("Input colour values, seperated by whitespace:");

    input.clear();
    stdin().read_line(&mut input).unwrap();

    let colours: Vec<f32> = input.split_whitespace().map(|colour| {
        let colour = colour.parse::<f32>().unwrap() / 255.0;

        if to_linear {
            srgb_to_linear(colour)
        } else {
            linear_to_srgb(colour)
        }
    }).collect();

    println!("{:?}", colours);
}