pub mod grid;
pub mod shape;
pub mod help_text;

pub mod engine {
    use crate::grid::*;
    use crate::shape::*;
    use crate::help_text;

    use colored::*;
    use rand::{distributions::Uniform, prelude::*};
    use std::{
        error::Error,
        fmt::{self, Display},
        io,
        sync::mpsc::*,
        thread,
        time::Duration,
    };

    #[macro_export]
    macro_rules! err {
        () => {eprintln!();};
        ($($t: tt)*) => {
            eprintln!("{}", format!("{}", format_args!($($t)*)).bold().red());
        };
    }

    #[derive(Debug)]
    pub struct ConwayEngine {
        generation: isize,
        grid: Grid,
    }

    #[repr(u8)]
    #[derive(Debug)]
    pub enum GridCommand {
        Quit,
        Start,
        Help,
        Random,
        Clear,
        // Fix this command
        Resize,
        Set((isize, isize), (isize, isize)),
        // Change this to have an orientation
        Shape(((isize, isize), (isize, isize)), Shape),
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
                // U+2588 - Large unicode box
                return write!(f, "\u{2588}");
            }
            write!(f, " ")
        }
    }

    fn spawn_input_thread() -> Receiver<String> {
        let (tx, rx) = channel();
        thread::spawn(move || loop {
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("Invalid input");
            tx.send(input).expect("Couldn't send input through channel");
        });
        rx
    }

    impl ConwayEngine {
        /// Returns a new ConwayEngine with a grid of size `rows * columns`.
        pub fn new(rows: isize, columns: isize, grid_type: GridType) -> ConwayEngine {
            ConwayEngine {
                generation: 0,
                grid: Grid::new(rows, columns, grid_type),
            }
        }

        // TODO: Could most likely refactor this
        pub fn from_input() -> io::Result<ConwayEngine> {
            loop {
                let mut input = String::new();

                println!("{}", help_text::init_text());

                io::stdin().read_line(&mut input)?;
                let inputs: Vec<String> = input
                    .trim()
                    .split('-')
                    .filter_map(|s| (!s.is_empty()).then(|| s.trim().to_lowercase()))
                    .collect();

                if inputs.is_empty() {
                    err!("No command specified.");
                    continue;
                } else if inputs.len() == 1 {
                    err!("No grid type specified.");
                    continue;
                }
                
                let size: Vec<&str> = inputs[0].split(',').filter_map(|s| (!s.is_empty()).then(|| s.trim())).collect();
                
                if size.is_empty() {
                    err!("No grid row specified.");
                    continue;
                } else if size.len() == 1 {
                    err!("No grid column specified.");
                    continue;
                }

                // Set the grid size
                let row: isize = match size[0].parse() {
                    Err(_) => {
                        err!("Invalid row \'{}\'.", size[0]);
                        continue;
                    }
                    Ok(r) => r,
                };
                let col: isize = match size[1].parse() {
                    Err(_) => {
                        err!("Invalid column \'{}\'.", size[1]);
                        continue;
                    }
                    Ok(c) => c,
                };

                if row < 1 || col < 1 {
                    err!("Grid cannot have less than one row/column.");
                    continue;
                }

                // Set the grid type
                let grid_type = match inputs[1].as_str() {
                    "open" => GridType::Open,
                    "closed" => GridType::Closed,
                    _ => {
                        err!("Invalid grid type.");
                        continue;
                    }
                };

                return Ok(Self::new(row, col, grid_type));
            }
        }

        // This is scuffed (size is fucked and it should throw more errors when exceeding size limits)
        /// This function assumes that `input` has at least a len of 1
        fn handle_input(input: String) -> Result<GridCommand, String> {
            let commands = input
                .split(' ')
                .filter_map(|s| (!s.is_empty()).then(|| s.to_lowercase()))
                .collect::<Vec<String>>();

            if commands.is_empty() {
                return Err(String::from("No command specified."));
            }

            let len = commands.len();

            match commands[0].as_str() {
                _ => {
                    return Err(format!("Invalid command {}", commands[0]));
                }
            }

            // !!!! DO NOT REMOVE UNTIL FIXED !!!!!
            // if len > 1 {
            //     match commands[0].as_str() {
            //         "glider" => {
            //             let (row_range, col_range) = Self::get_ranges(&commands[1])?;

            //             match Self::parse_ranges(row_range, col_range) {
            //                 Ok((pos1, pos2)) => {
            //                     return Some(GridCommand::Shape((pos1, pos2), Shape::Glider));
            //                 }
            //                 Err(e) => {
            //                     err!(e);
            //                     return None;
            //                 }
            //             }
            //         }
            //         "square" => {
            //             let (row_range, col_range) = Self::get_ranges(&commands[1])?;

            //             match Self::parse_ranges(row_range, col_range) {
            //                 Ok((pos1, pos2)) => {
            //                     return Some(GridCommand::Shape((pos1, pos2), Shape::Square));
            //                 }
            //                 Err(e) => {
            //                     err!(e);
            //                     return None;
            //                 }
            //             }
            //         }
            //         "circle" => {
            //             let (row_range, col_range) = Self::get_ranges(&commands[1])?;

            //             match Self::parse_ranges(row_range, col_range) {
            //                 Ok((pos1, pos2)) => {
            //                     return Some(GridCommand::Shape((pos1, pos2), Shape::Circle));
            //                 }
            //                 Err(e) => {
            //                     err!(e);
            //                     return None
            //                 }
            //             }
            //         }
            //         "resize" => {}
            //         _ => {}
            //     }
            // }

            // match commands[0].as_str() {
            //     "start" => {
            //         return Some(GridCommand::Start);
            //     }
            //     "random" => {
            //         return Some(GridCommand::Random);
            //     }
            //     "clear" => {
            //         return Some(GridCommand::Clear);
            //     }
            //     "exit" | "quit" | "q" | "e" => {
            //         return Some(GridCommand::Quit);
            //     }
            //     "help" => {
            //         return Some(GridCommand::Help);
            //     }
            //     _ => {}
            // }

            // let coords: Vec<&str> = commands[0].split(',').map(|s| s.trim()).collect();

            // match coords.len() {
            //     len if len == 2 => {
            //         let (row_ranges, col_ranges) = Self::get_ranges(&commands[0])?;

            //         match Self::parse_ranges(row_ranges, col_ranges) {
            //             Err(e) => {
            //                 err!(e);
            //                 None
            //             }

            //             Ok((row_range, col_range)) => Some(GridCommand::Set(row_range, col_range)),
            //         }
            //     }
            //     len if len > 2 => {
            //         err!("Unexpected position ", "\'", coords[2], "\'");
            //         None
            //     }
            //     _ => {
            //         err!("Invalid command ", "\'", input, "\'");
            //         None
            //     }
            // }
        }

        /// Parses (row_ranges, col_ranges) -> Vec<String>s into Vec2<isize>s, or returns Err(String)
        fn parse_ranges(
            row_ranges: Vec<String>,
            col_ranges: Vec<String>,
        ) -> Result<((isize, isize), (isize, isize)), String> {
            // TODO: This shit is WET as fuck
            let row_range1: isize = if !row_ranges.is_empty() {
                match row_ranges[0].parse() {
                    Err(_) => {
                        return Err(format!("Invalid first row index \'{}\'", row_ranges[0]));
                    }
                    Ok(row) => row,
                }
            } else {
                return Err(String::from("No first row index"));
            };

            let row_range2: isize = if row_ranges.len() > 1 {
                match row_ranges[1].parse() {
                    Err(_) => {
                        return Err(format!("Invalid second row index \'{}\'", row_ranges[1]));
                    }
                    Ok(row) => row,
                }
            } else {
                row_range1
            };

            let col_range1: isize = if !col_ranges.is_empty() {
                match col_ranges[0].parse() {
                    Err(_) => {
                        return Err(format!("Invalid first column index \'{}\'", col_ranges[0]));
                    }
                    Ok(col) => col,
                }
            } else {
                return Err(String::from("No first column index"));
            };

            let col_range2: isize = if col_ranges.len() > 1 {
                match col_ranges[1].parse() {
                    Err(_) => {
                        return Err(format!("Invalid second column index \'{}\'", col_ranges[1]));
                    }
                    Ok(col) => col,
                }
            } else {
                col_range1
            };
            Ok(((row_range1, row_range2), (col_range1, col_range2)))
        }

        fn get_ranges(coords: &str) -> Result<(Vec<String>, Vec<String>), String> {
            let coords: Vec<&str> = coords.split(',').map(|s| s.trim()).collect();

            match coords.len() {
                len if len > 2 => {
                    return Err(format!("Unexpected position \'{}\'", coords[2]));
                }
                len if len <= 1 => {
                    return Err(String::from("Not enough co-ordinates"));
                }
                _ => {}
            }

            // TODO: WET code?
            // .then returns Some(F), where F: FnMut if the bool is true, else None
            let row_ranges: Vec<String> = coords[0]
                .split('-')
                .filter_map(|s| (!s.is_empty()).then(|| s.trim().to_string()))
                .collect();
            let col_ranges: Vec<String> = coords[1]
                .split('-')
                .filter_map(|s| (!s.is_empty()).then(|| s.trim().to_string()))
                .collect();
            Ok((row_ranges, col_ranges))
        }

        // split by spaces
        pub fn get_command() -> io::Result<Result<GridCommand, String>> {
            let mut input = String::new();

            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            match Self::handle_input(String::from(input)) {
                Ok(command) => Ok(Ok(command)),
                Err(e) => Ok(Err(e)),
            }
        }

        /// Updates the grid and increments the generation counter.
        pub fn next_generation(&mut self) {
            let mut next_grid = self.grid.clone();

            self.generation += 1;
            thread::sleep(Duration::from_millis(150));
            for row in 1..=self.grid.rows {
                for col in 1..=self.grid.columns {
                    let pos = GridVec2::new(row, col);

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
        /// Changes the state of a cell at `pos` to [`State::Alive`]/[`State::Dead`], only if it's inside the grid.
        pub fn set_cell(&mut self, pos: &GridVec2<isize>) {
            if self.grid.get_index_closed(pos).is_some() {
                if let State::Alive = self.grid.get_state_closed(pos) {
                    self.grid.kill_cell(pos);
                } else {
                    self.grid.make_cell(pos);
                }
            }
        }

        /// Displays the current grid.
        pub fn display(&self) {
            // Clears the terminal
            print!("{esc}c", esc = 27 as char);
            let mut result: String = String::new();

            result.push_str(&format!("\nGeneration: {}", self.generation));

            // For the row and column numbers
            let get_coloured = |ind: isize| {
                let ind = ind % 10;

                let mut ind_str = ind.to_string();
                if ind == 0 {
                    ind_str = format!("{}", ind_str.green().bold());
                }
                ind_str
            };

            result.push_str("\n ");
            for col in 0..self.grid.columns {
                result.push_str(&get_coloured(col));
            }
            // .enumerate creates a new iterator which also keeps track of it's current iteration count
            for (counter, cell) in self.grid.cells.iter().enumerate() {
                if counter as isize % self.grid.columns == 0 {
                    result.push('\n');
                    result.push_str(&get_coloured(counter as isize / self.grid.columns));
                }
                result.push_str(&format!("{}", cell));
            }

            result.push('\n');
            print!("{}", result);
        }

        /// Main game loop.
        /// Displays the current grid and goes to the next generation.
        pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
            self.display();

            // TODO: Fix this absoulte cancer, seperate match data into files? better game running logic, allowing for restarts
            // TODO: Seperate into functions?
            loop {
                println!("Give a single co-ordinate in the format {}, {} to set/remove a cell or\n{}, {} where {}, {}, etc. are indices that will set/remove cells at those specified locations.\nType in {} to get the list of commands.",
                    "row".cyan().bold(),
                    "column".cyan().bold(),
                    "row1-row2".cyan().bold(),
                    "col1-col2".cyan().bold(),
                    "row1".cyan(),
                    "row2".cyan(),
                    "help".green().bold());

                let answer;

                loop {
                    if let Ok(com) = Self::get_command()? {
                        answer = com;
                        break;
                    }
                }

                match answer {
                    GridCommand::Set(rr, cr) => {
                        for row in rr.0.min(rr.1)..=self.grid.rows.min(rr.0.max(rr.1)) {
                            for col in cr.0.min(cr.1)..=self.grid.columns.min(cr.0.max(cr.1)) {
                                self.set_cell(&GridVec2::new(row, col));
                            }
                        }
                    }
                    // Make the cody dry by making the offsets constant
                    GridCommand::Shape((rr, cr), shape) => match shape {
                        Shape::Square => {
                            self.offset_set(rr, cr, &SQUARE);
                        }
                        Shape::Circle => {
                            self.offset_set(rr, cr, &CIRCLE);
                        }
                        Shape::Glider => {
                            self.offset_set(rr, cr, &GLIDER);
                        }
                    },
                    GridCommand::Random => {
                        let mut rng = thread_rng();
                        let uniform = Uniform::from(0.0..=1.0);
                        for r in 1..=self.grid.rows {
                            for c in 1..=self.grid.columns {
                                if uniform.sample(&mut rng) > 0.5 {
                                    self.set_cell(&GridVec2::new(r, c));
                                }
                            }
                        }
                    }
                    GridCommand::Help => {
                        println!("{}\n{} to start the game\n{} to randomize the board\n{} to clear the board\n{} for help\n{} to quit\n",
                            "Commands:".blue(),
                            "start".green().bold(),
                            "random".green().bold(),
                            "clear".green().bold(),
                            "help".green().bold(),
                            "q[uit]/e[xit]".green().bold());
                        continue;
                    }
                    GridCommand::Clear => {
                        self.grid =
                            Grid::new(self.grid.rows, self.grid.columns, self.grid.grid_type)
                    }
                    // TODO: Proper reszing?
                    GridCommand::Resize => {
                        self.grid = Self::from_input()?.grid;
                    }
                    GridCommand::Start => {
                        break;
                    }
                    GridCommand::Quit => {
                        return Ok(());
                    }
                }
                self.display();
            }

            let rx = spawn_input_thread();

            loop {
                let mut stopped = false;
                loop {
                    match rx.try_recv() {
                        Ok(key) => {
                            let key = key.trim();
                            match key {
                                "quit" | "exit" | "q" | "e" => {
                                    return Ok(());
                                }
                                "stop" => stopped = true,
                                "start" => stopped = false,
                                _ => {}
                            }
                        }
                        Err(TryRecvError::Empty) => {}
                        Err(e) => {
                            return Err(Box::new(e));
                        }
                    }
                    if !stopped {
                        break;
                    }
                    // Sleep so you don't eat all of the cpu on one core (you can get input since it's on another thread)
                    thread::sleep(Duration::from_millis(50));
                }
                self.next_generation();
                self.display();
            }
        }

        fn offset_set(&mut self, rr: (isize, isize), cr: (isize, isize), off: &[Offset]) {
            for offset in off {
                // To avoid unnecessary sets
                for row in rr.0.min(rr.1)..=self.grid.rows.min(rr.0.max(rr.1)) {
                    for col in cr.0.min(cr.1)..=self.grid.columns.min(cr.0.max(cr.1)) {
                        self.set_cell(&GridVec2::new(row, col).from_offset(*offset));
                    }
                }
            }
        }
    }
}
