// TODO: Use a graphics library like druid?
// Could also change the initial display too include numbers for the rows and columns like how they are on a chessboard, then remove them later
pub mod life {
    const NUMS: &str = "0123456789";

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
    // {..., fs::File}
    // use serde::Deserialize;
    // use ron::de::from_reader;

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

    #[derive(Debug)]
    pub enum GridCommand {
        Quit,
        Start,
        Random,
        Clear,
        Set((isize, isize), (isize, isize)),
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

    impl Display for State {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if *self == State::Alive {
                // U+2588 - Large unicode box
                return write!(f, "\u{2588}");
            }
            write!(f, " ")
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

        // Maybe add to getting input function?
        pub fn from_input() -> Result<ConwayEngine, Box<dyn Error>> {
            loop {
                let mut input = String::new();

                println!("Give the number of rows and columns in the format of {}, {} that you want to be in the grid.",
                    "row".cyan().bold(), 
                    "column".cyan().bold());

                io::stdin().read_line(&mut input)?;
                let size: Vec<&str> = input
                    .split(',')
                    .map(|s| s.trim_matches(|c| !NUMS.contains(c)))
                    .collect();

                if size.len() > 1 {
                    let row: isize = match size[0].parse() {
                        Err(_) => {
                            continue;
                        }
                        Ok(r) => r,
                    };
                    let col: isize = match size[1].parse() {
                        Err(_) => {
                            continue;
                        }
                        Ok(c) => c,
                    };
                    if row < 1 || col < 1 {
                        println!("Grid cannot have less than one row/column.");
                        continue;
                    }
                    return Ok(Self::new(row, col));
                }

                continue;
            }
        }

        // something like this
        // if let Some(commnand) = get_command(input) { return command; }
        fn handle_input(input: &str) -> Option<GridCommand> {
            match input {
                "random" => Some(GridCommand::Random),
                "clear" => Some(GridCommand::Clear),
                "exit" | "quit" | "q" | "e" => Some(GridCommand::Quit),
                _ => None,
            }
        }

        pub fn get_command() -> io::Result<GridCommand> {
            let mut input = String::new();

            println!("Give a single co-ordinate in the format {}, {} to set/remove a cell or\n{}, {} where {}, {}, etc. are indices that will set/remove cells at those specified locations.\nType in anything else to start the game.", 
                "row".cyan().bold(),
                "column".cyan().bold(),
                "row1-row2".cyan().bold(),
                "col1-col2".cyan().bold(),
                "row1".cyan(),
                "row2".cyan());

            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if let Some(command) = Self::handle_input(input.to_lowercase().as_str()) {
                return Ok(command);
            }

            let coords: Vec<&str> = input
                .split(',')
                .map(|s| s.trim_matches(|c| !NUMS.contains(c)))
                .collect();

            match coords.len() {
                len if len > 1 => {
                    // .then returns Some(F), where F: FnMut if the bool is true, else None
                    let row_ranges: Vec<&str> = coords[0]
                        .split('-')
                        .filter_map(|s| {
                            (!s.is_empty()).then(|| s.trim_matches(|c| !NUMS.contains(c)))
                        })
                        .take(2)
                        .collect();
                    let col_ranges: Vec<&str> = coords[1]
                        .split('-')
                        .filter_map(|s| {
                            (!s.is_empty()).then(|| s.trim_matches(|c| !NUMS.contains(c)))
                        })
                        .take(2)
                        .collect();

                    let row_range1: isize = if !row_ranges.is_empty() {
                        match row_ranges[0].parse() {
                            Err(_) => {
                                return Ok(GridCommand::Start);
                            }
                            Ok(row) => row,
                        }
                    } else {
                        return Ok(GridCommand::Start);
                    };

                    let row_range2: isize = if row_ranges.len() > 1 {
                        match row_ranges[1].parse() {
                            Err(_) => row_range1,
                            Ok(row) => row,
                        }
                    } else {
                        row_range1
                    };

                    let col_range1: isize = if !col_ranges.is_empty() {
                        match col_ranges[0].parse() {
                            Err(_) => {
                                return Ok(GridCommand::Start);
                            }
                            Ok(col) => col,
                        }
                    } else {
                        return Ok(GridCommand::Start);
                    };

                    let col_range2: isize = if col_ranges.len() > 1 {
                        match col_ranges[1].parse() {
                            Err(_) => col_range1,
                            Ok(col) => col,
                        }
                    } else {
                        col_range1
                    };

                    Ok(GridCommand::Set(
                        (row_range1, row_range2),
                        (col_range1, col_range2),
                    ))
                }
                _ => Ok(GridCommand::Start),
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
        // Seperate logic for handling commands?
        pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
            self.display();

            loop {
                let answer = Self::get_command()?;
                match answer {
                    GridCommand::Set((row1, row2), (col1, col2)) => {
                        for r in row1.min(row2)..=row1.max(row2) {
                            for c in col1.min(col2)..=col1.max(col2) {
                                self.set_cell(&Vec2::new(r, c));
                            }
                        }
                    }
                    GridCommand::Random => {
                        let mut rng = thread_rng();
                        let uniform = Uniform::from(0.0..=1.0);
                        for r in 1..=self.grid.rows {
                            for c in 1..=self.grid.columns {
                                if uniform.sample(&mut rng) > 0.5 {
                                    self.set_cell(&Vec2::new(r, c));
                                }
                            }
                        }
                    }
                    GridCommand::Clear => self.grid = Grid::new(self.grid.rows, self.grid.columns),
                    GridCommand::Start => {
                        break;
                    }
                    GridCommand::Quit => {
                        return Ok(());
                    }
                }
                self.display();
            }

            let rx = Self::spawn_input_thread();

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
                    thread::sleep(Duration::from_millis(100));
                }
                self.next_generation();
                self.display();
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
    }
}
