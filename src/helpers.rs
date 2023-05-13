//! helper functions and structures that will be used throughout the codebase
#![allow(dead_code)]

/// a tuple of  (x: A, y: A) meant to represent a point in space
pub type Point<A> = (A, A);

pub trait Distance: num_traits::Num {
    fn distance(a: Self, b: Self) -> Self;
}

impl<A: num_traits::Num + PartialOrd> Distance for A {
    fn distance(a: Self, b: Self) -> Self {
        if a > b {
            a - b
        } else {
            b - a
        }
    }
}

/// Takes an index, the width and height of a rectangular array, and converts from the i to an x and y location
///
/// return is (x,y)
/// this may be bugged. either this or [point_to_index]
pub fn index_to_point(i: usize, w: usize, h: usize) -> Point<usize> {
    // x is i modulo width
    // y is (i - x) / width
    //

    (i % w, (i - (i % w)) / h)
    // let x = i % h;

    // (x, (i - x) / w)
}

/// takes an x, a y, and a width and converts from x and y to the index of a rectangular array
///
/// this may be bugged. either this or [index_to_point]
pub fn point_to_index(x: usize, y: usize, w: usize) -> usize {
    x * w + y
}

///  returns all of the points surrounding a given point
pub fn points_around<A: num_traits::Num + Copy>(x: A, y: A) -> [Point<A>; 8] {
    // get generic one
    let one = A::one();

    [
        (x - one, y),
        (x - one, y - one),
        (x - one, y + one),
        (x, y + one),
        (x, y - one),
        (x + one, y - one),
        (x + one, y),
        (x + one, y + one),
    ]
}

/// a rectangle with a height and width
#[derive(Debug, Clone, Copy)]
pub struct RectDimension {
    pub width: u8,
    pub height: u8,
}

impl RectDimension {
    pub fn new(width: u8, height: u8) -> Self {
        RectDimension { width, height }
    }

    pub fn area(&self) -> usize {
        self.width as usize * self.height as usize
    }

    pub fn point_to_index(&self, x: usize, y: usize) -> usize {
        point_to_index(x, y, self.width as usize)
    }

    pub fn index_to_point(&self, i: usize) -> Point<usize> {
        index_to_point(i, self.width as usize, self.height as usize)
    }
}
