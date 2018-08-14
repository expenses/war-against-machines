#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
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

#[test]
fn grid_tests() {
    use bincode;

    let grid = Grid::new(5, 6, || 55_u8);
    assert_eq!(grid.width(), 5);
    assert_eq!(grid.height(), 6);
    assert_eq!(&grid.inner, &[55_u8; 30]);
    assert!(grid.in_bounds(4, 5));

    let buffer = bincode::serialize(&grid).unwrap();

    assert!(buffer.len() > 0);

    let grid_2 = bincode::deserialize::<Grid<u8>>(&buffer).unwrap();

    assert_eq!(grid_2.width(), 5);
    assert_eq!(grid_2.height(), 6);
    assert!(grid_2.in_bounds(4, 5));
    assert_eq!(&grid_2.inner, &[55_u8; 30]);
}