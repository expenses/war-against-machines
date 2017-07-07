use graphics::Context;

// Display the dimensions of something
pub trait Dimensions {
    fn width(&self) -> f64;
    fn height(&self) -> f64;
}

impl Dimensions for Context {
    fn width(&self) -> f64 {
        self.get_view_size()[0]
    }

    fn height(&self) -> f64 {
        self.get_view_size()[1]
    }
}