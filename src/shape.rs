use std::str::FromStr;

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

impl FromStr for Shape {
    // I know this is VERY bad
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "glider" => Ok(Self::Glider),
            "circle" => Ok(Self::Circle),
            "sqaure" => Ok(Self::Square),
            _ => Err(format!("Invalid shape {}", s)),
        }
    }
}
