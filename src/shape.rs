pub type Offset = (isize, isize);
pub const SQUARE: [Offset; 8] = [
    (0, 0),
    (0, 1),
    (0, 2),
    (1, 0),
    (1, 2),
    (2, 0),
    (2, 1),
    (2, 2),
];
pub const CIRCLE: [Offset; 4] = [(0, 0), (1, -1), (1, 1), (2, 0)];
pub const GLIDER: [Offset; 5] = [(0, 0), (0, 1), (1, 0), (1, 2), (2, 0)];

#[repr(u8)]
#[derive(Debug)]
pub enum Shape {
    Square,
    Circle,
    Glider,
}
