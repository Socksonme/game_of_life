pub mod life {
    use std::{fmt::{self, Display}, thread, time::Duration};
    // {..., fs::File}
    // use serde::Deserialize;
    // use ron::de::from_reader;

    #[derive(Debug, Clone)]
    pub struct Grid {
        cells: Vec<State>,
        columns: isize,
        rows: isize,
    }

    pub struct Vec2<T> {
        x: T,
        y: T
    }

    impl<T> Vec2<T> {
        pub fn new(x: T, y: T) -> Vec2<T> {
            return Vec2 {
                x,
                y
            };
        }
    }

    impl Grid {
        /// # Panics
        /// Panics if either the rows or columns are less than 1.
        pub fn new(rows: isize, columns: isize) -> Grid {
            if rows <= 0 || columns <= 0 {
                eprintln!("Can't make a grid with less than 1 columns/rows.");
                std::process::exit(1);
            }
            return Grid {
                cells: vec![State::Dead; (rows * columns) as usize],
                columns,
                rows,
            };
        }

        /// Converts a Vec2 to an Option<usize> index or None if the index is out of range.
        pub fn get_index(&self, pos: &Vec2<isize>) -> Option<usize> {
            // Return None if it's not inside the Grid
            if (pos.x > self.rows || pos.x <= 0) || (pos.y > self.columns || pos.y <= 0) {
                return None;
            }

            return Some((pos.x * self.columns + pos.y - self.columns - 1) as usize);
        }

        pub fn get_state(&self, pos: &Vec2<isize>) -> State {
            match self.get_index(pos) {
                Some(index) => return self.cells[index],
                None => {
                    return State::Dead;
                }
            };
        }

        pub fn get_nearby(&self, pos: &Vec2<isize>) -> isize {

            let mut count: isize = 0;

            // xxx
            // xCx
            // xxx - [-1, 0, 1] offset
            let offset: [isize; 3] = [-1, 0, 1];

            for off1 in offset {
                for off2 in offset {
                    // You check the state first, as the second if statement only matters if the state is alive, thus reducing the amount of if calls
                    if self.get_state(&Vec2::new(pos.x + off1, pos.y + off2)) == State::Alive {
                        if !(off1 == 0 && off2 == 0) {
                            count += 1;
                        }
                    }
                }
            }
            return count;
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

    // Setting the underlying type of the enum (So instead of beaing an i32, let's say, we can make it into a u8.)
    #[repr(u8)]
    #[derive(Debug, PartialEq, Copy, Clone)]
    pub enum State {
        Dead,
        Alive,
    }

    impl Display for State {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if *self == State::Alive {
                return write!(f, "X");
            }
            return write!(f, "~");
        }
    }

    pub struct ConwayEngine {
        generation: isize,
        grid: Grid
    }
    
    impl ConwayEngine {
        /// Returns a new ConwayEngine with a grid of size rows * columns.
        pub fn new(rows: isize, columns: isize) -> ConwayEngine {
            return ConwayEngine {
                generation: 0,
                grid: Grid::new(rows, columns),
            };
        }
        /// Updates the grid and increments the generation counter.
        pub fn next_generation(&mut self) {
            let mut next_grid = self.grid.clone();

            self.generation += 1;
            thread::sleep(Duration::from_millis(150));
            for row in 1..=self.grid.rows {
                for col in 1..=self.grid.columns {
                    let pos = Vec2::new(row, col);

                    match self.grid.get_nearby(&pos) {
                        3 => {
                            next_grid.make_cell(&pos);
                        },
                        2 => {},
                        _ => {
                            next_grid.kill_cell(&pos);
                        }
                    }
                }
            }
            self.grid = next_grid;
        }
        /// Sets a cell at `pos` to `State::Alive`, as long as it's inside the grid.
        pub fn set_cell(&mut self, pos: &Vec2<isize>) {
            self.grid.make_cell(pos);
        }

        /// Displays the current grid.
        pub fn display(&self) {
            // Clears the terminal
            println!("{esc}c", esc = 27 as char);
            let mut result: String = String::new();

            result.push_str(&format!("\nGeneration: {}", self.generation));
            
            // .enumerate creates a new iterator which also keeps track of it's current iteration count 
            for (counter, cell) in self.grid.cells.iter().enumerate() {
                if counter as isize % self.grid.columns == 0 {
                    result.push('\n');
                }
                result.push_str(&format!("{}", cell));
            }

            result.push('\n');
            print!("{}", result);
        }

        /// Main game loop.
        /// Displays the current grid and goes to the next generation.
        pub fn run(&mut self) {
            loop {
                self.display();
                self.next_generation();
            }
        }
    }
}