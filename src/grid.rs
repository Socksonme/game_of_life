use crate::engine::State;
use crate::shape::Offset;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum GridType {
    Closed,
    Open,
}

#[derive(Debug)]
pub struct GridVec2<T> {
    pub row: T,
    pub column: T,
}

impl<T> GridVec2<T> {
    pub fn new(row: T, column: T) -> GridVec2<T> {
        GridVec2 { row, column }
    }
}
impl GridVec2<isize> {
    pub fn from_offset(&self, offset: Offset) -> GridVec2<isize> {
        GridVec2 {
            row: self.row + offset.0,
            column: self.column + offset.1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Grid {
    pub cells: Vec<State>,
    pub columns: isize,
    pub rows: isize,
    pub grid_type: GridType,
}

impl Grid {
    pub fn new(rows: isize, columns: isize, grid_type: GridType) -> Grid {
        Grid {
            cells: vec![State::Dead; (rows * columns) as usize],
            columns,
            rows,
            grid_type,
        }
    }

    /// Converts a [`GridVec2`] to a `Some(usize)` index, or [`None`] if the index is out of range.
    pub fn get_index_closed(&self, pos: &GridVec2<isize>) -> Option<usize> {
        // Return None if it's not inside the Grid
        if (pos.row > self.rows || pos.row <= 0) || (pos.column > self.columns || pos.column <= 0) {
            return None;
        }

        Some((pos.row * self.columns + pos.column - self.columns - 1) as usize)
    }

    /// Converts a [`GridVec2`] to a `Some(usize)` index, or wraps around if the index is out of range.
    pub fn get_index_open(&self, pos: &GridVec2<isize>) -> usize {
        let row;
        let col;

        // ex. 10,10 grid, 0 -> 10, -1 -> 9, -2 -> 8 ..., -10 -> 10, -11 -> 9 etc.
        if pos.row <= 0 {
            row = pos.row % -self.rows + self.rows;
        } else if pos.row > self.rows {
            row = pos.row % self.rows;
        } else {
            row = pos.row;
        }
        if pos.column <= 0 {
            col = pos.column % -self.columns + self.columns;
        } else if pos.column > self.columns {
            col = pos.column % self.columns;
        } else {
            col = pos.column;
        }

        (row * self.columns + col - self.columns - 1) as usize
    }

    pub fn get_state_closed(&self, pos: &GridVec2<isize>) -> State {
        match self.get_index_closed(pos) {
            Some(index) => self.cells[index],
            None => State::Dead,
        }
    }

    pub fn get_state_open(&self, pos: &GridVec2<isize>) -> State {
        self.cells[self.get_index_open(pos)]
    }

    pub fn get_nearby(&self, pos: &GridVec2<isize>) -> isize {
        let mut count: isize = 0;
        let offsets: [Offset; 8] = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];

        for off in offsets {
            if let State::Alive = match self.grid_type {
                GridType::Open => self.get_state_open(&pos.from_offset(off)),
                GridType::Closed => self.get_state_closed(&pos.from_offset(off)),
            } {
                count += 1;
            }
        }
        count
    }

    pub fn kill_cell(&mut self, pos: &GridVec2<isize>) {
        match self.grid_type {
            GridType::Closed => {
                if let Some(index) = self.get_index_closed(pos) {
                    self.cells[index] = State::Dead;
                }
            }
            GridType::Open => {
                let index = self.get_index_open(pos);
                self.cells[index] = State::Dead;
            }
        }
    }

    pub fn make_cell(&mut self, pos: &GridVec2<isize>) {
        match self.grid_type {
            GridType::Closed => {
                if let Some(index) = self.get_index_closed(pos) {
                    self.cells[index] = State::Alive;
                }
            }
            GridType::Open => {
                let index = self.get_index_open(pos);
                self.cells[index] = State::Alive;
            }
        }
    }
}
