pub mod grid;
pub mod help_text;
pub mod shape;

pub mod engine {
    use crate::grid::*;
    use crate::help_text;
    use crate::shape::*;

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
        Random(f64),
        Clear,
        // Fix this command
        Resize,
        Set(Range, Range),
        // Change this to have an orientation
        Shape((Range, Range), Shape),
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

                help_text::init_text();

                io::stdin().read_line(&mut input)?;
                let inputs: Vec<String> = input
                    .trim()
                    .split('-')
                    .filter_map(|s| (!s.is_empty()).then(|| s.trim().to_lowercase()))
                    .collect();

                // Throw no grid type error later
                if inputs.is_empty() {
                    err!("No command specified.");
                    continue;
                }

                let size: Vec<&str> = inputs[0]
                    .split(',')
                    .filter_map(|s| (!s.is_empty()).then(|| s.trim()))
                    .collect();

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

                if inputs.len() == 1 {
                    err!("No grid type specified.");
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
                "help" => {
                    if len > 1 {
                        return Err(format!("Unexpected arguments: {:?}", &commands[1..]));
                    }
                    Ok(GridCommand::Help)
                }
                "clear" => {
                    if len > 1 {
                        return Err(format!("Unexpected arguments: {:?}", &commands[1..]));
                    }

                    Ok(GridCommand::Clear)
                }
                "resize" => {
                    if len > 1 {
                        return Err(format!("Unexpected arguments: {:?}", &commands[1..]));
                    }

                    Ok(GridCommand::Resize)
                }
                "start" => {
                    if len > 1 {
                        return Err(format!("Unexpected arguments: {:?}", &commands[1..]));
                    }

                    Ok(GridCommand::Start)
                }
                "random" => {
                    if len > 2 {
                        return Err(format!("Unexpected arguments: {:?}", &commands[2..]));
                    }

                    let set_chance = if len == 2 {
                        match commands[1].parse::<f64>() {
                            Ok(n) => {
                                if n < 0.0 {
                                    return Err(String::from("Chance can't be below 0.0."));
                                } else if n > 1.0 {
                                    return Err(String::from("Chance can't be higher than 1.0."));
                                }

                                n
                            }
                            Err(_) => {
                                return Err(format!("Invalid set chance \'{}\'", commands[1]));
                            }
                        }
                    } else {
                        0.5
                    };

                    Ok(GridCommand::Random(set_chance))
                }
                "quit" | "exit" | "e" | "q" => {
                    if len > 1 {
                        return Err(format!("Unexpected arguments: {:?}", &commands[1..]));
                    }

                    Ok(GridCommand::Quit)
                }
                "glider" | "square" | "circle" => {
                    if len < 2 {
                        return Err(String::from("Not enough arguments."));
                    }

                    Ok(GridCommand::Shape(
                        Self::get_ranges(&commands[1])?,
                        commands[0].parse()?,
                    ))
                }

                _ => match Self::get_ranges(&commands[0]) {
                    Ok((rr, cr)) => Ok(GridCommand::Set(rr, cr)),
                    Err(e) => Err(e),
                },
            }
        }

        fn get_ranges(coords: &str) -> Result<(Range, Range), String> {
            let coords: Vec<&str> = coords.split(',').map(|s| s.trim()).collect();

            if coords.len() > 2 {
                return Err(format!("Unexpected position \'{}\'", coords[2]));
            } else if coords.len() <= 1 {
                return Err(String::from("Not enough co-ordinates."));
            }

            let mut rr: Vec<isize> = vec![];
            let mut cr: Vec<isize> = vec![];

            // .then returns Some(F), where F: FnMut if the bool is true, else None
            // only take 2 as ranges are defined as two isizes
            for row in coords[0]
                .split('-')
                .filter_map(|s| (!s.is_empty()).then(|| s.trim()))
                .take(2)
            {
                match row.parse() {
                    Ok(n) => {
                        rr.push(n);
                    }
                    _ => {
                        return Err(format!("Invalid row {}", row));
                    }
                }
            }

            for col in coords[1]
                .split('-')
                .filter_map(|s| (!s.is_empty()).then(|| s.trim()))
                .take(2)
            {
                match col.parse() {
                    Ok(n) => {
                        cr.push(n);
                    }
                    _ => {
                        return Err(format!("Invalid column {}", col));
                    }
                }
            }

            let (r1, c1) = (rr[0], cr[0]);

            let r2 = if rr.len() == 1 { r1 } else { rr[1] };

            let c2 = if cr.len() == 1 { c1 } else { cr[1] };

            Ok(((r1, r2), (c1, c2)))
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
                help_text::run_text();

                let answer;

                loop {
                    match Self::get_command()? {
                        Ok(com) => {
                            answer = com;
                            break;
                        }
                        Err(e) => {
                            err!("{}", e);
                        }
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
                    GridCommand::Random(set_chance) => {
                        let mut rng = thread_rng();
                        let uniform = Uniform::from(0.0..=1.0);
                        for r in 1..=self.grid.rows {
                            for c in 1..=self.grid.columns {
                                if uniform.sample(&mut rng) > set_chance {
                                    self.set_cell(&GridVec2::new(r, c));
                                }
                            }
                        }
                    }
                    GridCommand::Help => {
                        help_text::command_help_text();
                        continue;
                    }
                    GridCommand::Clear => {
                        self.grid =
                            Grid::new(self.grid.rows, self.grid.columns, self.grid.grid_type)
                    }
                    // TODO: Proper reszing?
                    GridCommand::Resize => {
                        // Temporary grid so you stop using any extra memory
                        self.grid = Grid::new(1, 1, GridType::Open);
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

        fn offset_set(&mut self, rr: Range, cr: Range, off: &[Offset]) {
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
