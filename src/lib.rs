// TODO: Use a graphics library like druid? (Probably needs a whole seperate github repo by now, lol)
pub mod life {

    #[derive(Debug, Clone)]
    pub struct Grid {
        cells: Vec<State>,
        columns: isize,
        rows: isize,
    }

    #[derive(Debug)]
    pub struct Vec2<T> {
        row: T,
        column: T,
    }

    impl<T> Vec2<T> {
        pub fn new(row: T, column: T) -> Vec2<T> {
            Vec2 { row, column }
        }
    }

    // Setting the underlying type of the enum (So instead of beaing an i32, let's say, we can make it into a u8.)
    #[repr(u8)]
    #[derive(Debug, PartialEq, Copy, Clone)]
    pub enum State {
        Dead,
        Alive,
    }

    impl Grid {
        pub fn new(rows: isize, columns: isize) -> Grid {
            Grid {
                cells: vec![State::Dead; (rows * columns) as usize],
                columns,
                rows,
            }
        }

        /// Converts a [`Vec2`] to a `Some(usize)` index, or [`None`] if the index is out of range.
        pub fn get_index(&self, pos: &Vec2<isize>) -> Option<usize> {
            // Return None if it's not inside the Grid
            if (pos.row > self.rows || pos.row <= 0)
                || (pos.column > self.columns || pos.column <= 0)
            {
                return None;
            }

            Some((pos.row * self.columns + pos.column - self.columns - 1) as usize)
        }

        pub fn get_state(&self, pos: &Vec2<isize>) -> State {
            match self.get_index(pos) {
                Some(index) => self.cells[index],
                None => State::Dead,
            }
        }

        pub fn get_nearby(&self, pos: &Vec2<isize>) -> isize {
            let mut count: isize = 0;
            let offset: [(isize, isize); 8] = [
                (-1, -1),
                (-1, 0),
                (-1, 1),
                (0, -1),
                (0, 1),
                (1, -1),
                (1, 0),
                (1, 1),
            ];

            for off in offset {
                if self.get_state(&Vec2::new(pos.row + off.0, pos.column + off.1)) == State::Alive {
                    count += 1;
                }
            }
            count
        }

        fn kill_cell(&mut self, pos: &Vec2<isize>) {
            if let Some(index) = self.get_index(pos) {
                self.cells[index] = State::Dead;
            }
        }

        fn make_cell(&mut self, pos: &Vec2<isize>) {
            if let Some(index) = self.get_index(pos) {
                self.cells[index] = State::Alive;
            }
        }
    }

    #[derive(Debug)]
    pub struct ConwayEngine {
        generation: isize,
        grid: Grid,
    }

    impl ConwayEngine {
        /// Returns a new ConwayEngine with a grid of size `rows * columns`.
        pub fn new(rows: isize, columns: isize) -> ConwayEngine {
            ConwayEngine {
                generation: 0,
                grid: Grid::new(rows, columns),
            }
        }

        /// Updates the grid and increments the generation counter.
        pub fn next_generation(&mut self) {
            let mut next_grid = self.grid.clone();

            self.generation += 1;
            for row in 1..=self.grid.rows {
                for col in 1..=self.grid.columns {
                    let pos = Vec2::new(row, col);

                    match self.grid.get_nearby(&pos) {
                        3 => {
                            next_grid.make_cell(&pos);
                        }
                        2 => {}
                        _ => {
                            next_grid.kill_cell(&pos);
                        }
                    }
                }
            }
            self.grid = next_grid;
        }
        /// Changes the state of a cell at `pos` to [`State::Alive`]/[`State::Dead`], as long as it's inside the grid.
        pub fn set_cell(&mut self, pos: &Vec2<isize>) {
            if let State::Alive = self.grid.get_state(pos) {
                self.grid.kill_cell(pos);
            } else {
                self.grid.make_cell(pos);
            }
        }

        /// Main game loop.
        pub fn run(&mut self) {
            loop {
                self.next_generation();
            }
        }
    }
}
