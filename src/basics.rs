/// a rectangle with a height and width
#[derive(Debug, Clone, Copy)]
pub struct RectArea {
    pub width: u8,
    pub height: u8,
}

impl RectArea {
    pub fn new(width: u8, height: u8) -> Self {
        RectArea { width, height }
    }

    pub fn area(&self) -> usize {
        self.width as usize * self.height as usize
    }
}
