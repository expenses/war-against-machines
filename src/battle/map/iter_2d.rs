// A simple 2D iterator
// todo: this can probably be replaced with impl iterator

pub struct Iter2D {
    x: usize,
    y: usize,
    cols: usize,
    rows: usize
}

impl Iter2D {
    pub fn new(cols: usize, rows: usize) -> Iter2D {
        Iter2D {
            cols, rows,
            x: 0,
            y: 0
        }
    }
}

impl Iterator for Iter2D {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.y < self.rows {
            let point = Some((self.x, self.y));

            self.x += 1;

            if self.x == self.cols {
                self.x = 0;
                self.y += 1;
            }

            point
        } else {
            None
        }
    }
}