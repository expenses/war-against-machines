#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Grid<T> {
    width: usize,
    inner: Vec<T>
}

impl<T> Grid<T> {
    pub fn new<C: FnMut() -> T>(width: usize, height: usize, mut constuctor: C) -> Self {
        let len = width * height;

        let mut inner = Vec::with_capacity(len);

        for _ in 0 .. len {
            inner.push(constuctor());
        }

        Self {
            width, inner
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.inner.len() / self.width()
    }

    fn index(&self, x: usize, y: usize) -> usize {
        assert!(x < self.width() && y < self.height(), "Item at ({}, {}) is out of bounds", x, y);
        x * self.height() + y
    }

    pub fn at(&self, x: usize, y: usize) -> &T {
        let index = self.index(x, y);
        &self.inner[index]
    }

    pub fn at_mut(&mut self, x: usize, y: usize) -> &mut T {
        let index = self.index(x, y);
        &mut self.inner[index]
    }

    pub fn in_bounds(&self, x: usize, y: usize) -> bool {
        x < self.width() && y < self.height()
    }
}