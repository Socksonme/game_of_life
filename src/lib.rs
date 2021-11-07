// TODO: Use a graphics library like druid?
// TODO: Add a method to change the state of the game at the start of it, using rows and columns as indexes?
// Could also change the initial display too include numbers for the rows and columns like how they are on a chessboard, then remove them later  
pub mod life {
    use std::{fmt::{self, Display}, thread, time::Duration, io, error::Error};
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
        row: T,
        column: T
    }

    impl<T> Vec2<T> {
        pub fn new(row: T, column: T) -> Vec2<T> {
            return Vec2 {
                row,
                column
            };
        }
    }

    pub enum GridCommand {
        Exit,
        Set(Vec2<isize>),
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
            return Grid {
                cells: vec![State::Dead; (rows * columns) as usize],
                columns,
                rows,
            };
        }

        
        /// Converts a [`Vec2`] to a `Some(usize)` index, or [`None`] if the index is out of range.
        pub fn get_index(&self, pos: &Vec2<isize>) -> Option<usize> {
            // Return None if it's not inside the Grid
            if (pos.row > self.rows || pos.row <= 0) || (pos.column > self.columns || pos.column <= 0) {
                return None;
            }

            return Some((pos.row * self.columns + pos.column - self.columns - 1) as usize);
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
            let offset: [(isize, isize); 8] = [(-1, -1), (-1, 0), (-1, 1),
            (0, -1), (0, 1), 
            (1, -1), (1, 0), (1, 1)];
            
            for off in offset {
                if self.get_state(&Vec2::new(pos.row + off.0, pos.column + off.1)) == State::Alive {
                    count += 1;
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
    
    
    impl Display for State {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if *self == State::Alive {
                // U+2588 - Large unicode box
                return write!(f, "\u{2588}");
            }
            return write!(f, " ");
        }
    }
    
    pub struct ConwayEngine {
        generation: isize,
        grid: Grid
    }
    
    impl ConwayEngine {
        /// Returns a new ConwayEngine with a grid of size `rows * columns`.
        pub fn new(rows: isize, columns: isize) -> ConwayEngine {
            return ConwayEngine {
                generation: 0,
                grid: Grid::new(rows, columns),
            };
        }

        pub fn from_user_input() -> Result<ConwayEngine, Box<dyn Error>> {
            loop {
                let mut input = String::new();
        
                println!("Give the number of rows and columns (in the format of \"row, column\") that you want to be in the grid.");
        
                io::stdin().read_line(&mut input)?;
                let size: Vec<&str> = input.split(',').map(|s| s.trim()).collect();
                
                match size.len() {
                    len if len > 1 => {
                        let row: isize = match size[0].parse() {
                            Err(_) => {
                                continue;
                            }
                            Ok(r) => r
                        };
                        let col: isize = match size[1].parse() {
                            Err(_) => {
                                continue;
                            }
                            Ok(c) => c
                        };
                        if row < 1 || col < 1 {
                            println!("Grid cannot have less than one row/column.");
                            continue;
                        } 
                        return Ok(ConwayEngine::new(row, col));
                    }
                    _ => {
                        continue; 
                    }
                }
            }
        }


        pub fn get_next() -> Result<GridCommand, Box<dyn Error>> {
            let mut input = String::new();
    
            println!("Give a single co-ordinate with the format \"row, column\" to set/remove a cell or type anything else to stop changing the board.");
    
            io::stdin().read_line(&mut input)?;
            let coords: Vec<&str> = input.split(',').map(|s| s.trim()).collect();
            
            match coords.len() {
                len if len > 1 => {
                    let row: isize = match coords[0].parse() {
                        Err(_) => {
                            return Ok(GridCommand::Exit);
                        }
                        Ok(r) => r
                    };
                    let col: isize = match coords[1].parse() {
                        Err(_) => {
                            return Ok(GridCommand::Exit);
                        }
                        Ok(c) => c
                    };

                    return Ok(GridCommand::Set(Vec2::new(row, col)));
                }
                _ => {
                    return Ok(GridCommand::Exit);
                }
            }
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
        /// Changes the state of a cell at `pos` to [`State::Alive`]/[`State::Dead`], as long as it's inside the grid.
        pub fn set_cell(&mut self, pos: &Vec2<isize>) {
            if let State::Alive = self.grid.get_state(pos) {
                self.grid.kill_cell(pos);
            } else {
                self.grid.make_cell(pos);
            }
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
        pub fn run(&mut self) -> Result<(), Box<dyn Error>>{
            self.display();
            
            loop {
                let answer = ConwayEngine::get_next()?;
                match answer {
                    GridCommand::Set(pos) => {
                        self.set_cell(&pos);
                    }
                    _ => {
                        break;
                    }
                }
                self.display();
            }

            loop {
                self.next_generation();
                self.display();
            }
        }
    }
}
