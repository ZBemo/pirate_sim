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
pub fn index_to_point(i: usize, w: usize, h: usize) -> Point<usize> {
    ((i - (i % h)) / w, i % h)
}

/// takes an x, a y, and a width and converts from x and y to the index of a rectangular array
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
