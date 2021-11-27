pub mod life {
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

    type Offset = (isize, isize);

    const SQUARE: [Offset; 8] = [
        (0, 0),
        (0, 1),
        (0, 2),
        (1, 0),
        (1, 2),
        (2, 0),
        (2, 1),
        (2, 2),
    ];
    const CIRCLE: [Offset; 4] = [(0, 0), (1, -1), (1, 1), (2, 0)];
    const GLIDER: [Offset; 5] = [(0, 0), (0, 1), (1, 0), (1, 2), (2, 0)];

    #[derive(Debug, Clone)]
    pub struct Grid {
        cells: Vec<State>,
        columns: isize,
        rows: isize,
        grid_type: GridType,
    }

    #[derive(Debug)]
    pub struct Vec2<T> {
        first: T,
        second: T,
    }

    impl<T> Vec2<T> {
        pub fn new(first: T, second: T) -> Vec2<T> {
            Vec2 { first, second }
        }
    }

    #[derive(Debug)]
    pub struct GridVec2<T> {
        row: T,
        column: T,
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

    #[derive(Debug)]
    pub struct ConwayEngine {
        generation: isize,
        grid: Grid,
    }

    #[derive(Debug, Clone, Copy)]
    pub enum GridType {
        Closed,
        Open,
    }

    #[derive(Debug)]
    pub enum GridCommand {
        Quit,
        Start,
        Help,
        Random,
        Clear,
        // Fix this command
        Resize,
        Set(Vec2<isize>, Vec2<isize>),
        // Change this to have an orientation
        Shape((Vec2<isize>, Vec2<isize>), Shape),
    }

    // Setting the underlying type of the enum (So instead of beaing an i32, let's say, we can make it into a u8.)
    #[repr(u8)]
    #[derive(Debug, PartialEq, Copy, Clone)]
    pub enum State {
        Dead,
        Alive,
    }

    #[derive(Debug)]
    pub enum Shape {
        Square,
        Circle,
        Glider,
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
            if (pos.row > self.rows || pos.row <= 0)
                || (pos.column > self.columns || pos.column <= 0)
            {
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

        fn kill_cell(&mut self, pos: &GridVec2<isize>) {
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

        fn make_cell(&mut self, pos: &GridVec2<isize>) {
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

    impl Display for State {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if *self == State::Alive {
                // U+2588 - Large unicode box
                return write!(f, "\u{2588}");
            }
            write!(f, " ")
        }
    }

    impl ConwayEngine {
        /// Returns a new ConwayEngine with a grid of size `rows * columns`.
        pub fn new(rows: isize, columns: isize, grid_type: GridType) -> ConwayEngine {
            ConwayEngine {
                generation: 0,
                grid: Grid::new(rows, columns, grid_type),
            }
        }

        pub fn from_input() -> Result<ConwayEngine, Box<dyn Error>> {
            loop {
                let mut input = String::new();
                let mut grid_type = None;

                println!("Give the number of rows and columns in the format of {}, {} that you want to be in the grid\nand then (optionally - the default is {}) the type of the grid ({}/{})\n{},{} {}",
                    "row".cyan().bold(),
                    "column".cyan().bold(),
                    "closed".cyan().bold(),
                    "open".cyan().bold(),
                    "closed".cyan().bold(),
                    "row".cyan().bold(),
                    "column".cyan().bold(),
                    "type".cyan().bold());

                io::stdin().read_line(&mut input)?;
                let inputs: Vec<String> = input
                    .split(' ')
                    .filter_map(|s| (!s.is_empty()).then(|| s.trim().to_lowercase()))
                    .collect();
                if inputs.len() > 1 {
                    grid_type = match inputs[1].as_str() {
                        "open" => Some(GridType::Open),
                        "closed" => Some(GridType::Closed),
                        _ => {
                            eprintln!("{}", "Invalid grid type".red().bold());
                            continue;
                        }
                    };
                }
                if !inputs.is_empty() {
                    let size: Vec<&str> = inputs[0].split(',').map(|s| s.trim()).collect();

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
                            eprintln!("Grid cannot have less than one row/column.");
                            continue;
                        }
                        let grid_type = if let Some(gr_type) = grid_type {
                            gr_type
                        } else {
                            GridType::Closed
                        };

                        return Ok(Self::new(row, col, grid_type));
                    }
                }

                continue;
            }
        }

        // This is scuffed (size is fucked and it should throw more errors when exceeding size limits)
        /// This function assumes that `input` has at least a len of 1
        fn handle_input(input: String) -> Result<GridCommand, String> {
            let len = input.len();

            let commands = input
                .split(' ')
                .filter_map(|s| (!s.is_empty()).then(|| s.to_lowercase()))
                .collect::<Vec<String>>();

            if commands.is_empty() {
                return Err(String::from("No command specified."));
            }

            if len > 1 {
                match commands[0].as_str() {
                    "glider" => {
                        let (row_range, col_range) = Self::get_ranges(&commands[1])?;

                        match Self::parse_ranges(row_range, col_range) {
                            Ok((pos1, pos2)) => {
                                return Ok(GridCommand::Shape((pos1, pos2), Shape::Glider));
                            }
                            Err(e) => return Err(e),
                        }
                    }
                    "square" => {
                        let (row_range, col_range) = Self::get_ranges(&commands[1])?;

                        match Self::parse_ranges(row_range, col_range) {
                            Ok((pos1, pos2)) => {
                                return Ok(GridCommand::Shape((pos1, pos2), Shape::Square));
                            }
                            Err(e) => return Err(e),
                        }
                    }
                    "circle" => {
                        let (row_range, col_range) = Self::get_ranges(&commands[1])?;

                        match Self::parse_ranges(row_range, col_range) {
                            Ok((pos1, pos2)) => {
                                return Ok(GridCommand::Shape((pos1, pos2), Shape::Circle));
                            }
                            Err(e) => return Err(e),
                        }
                    }
                    "resize" => {}
                    _ => {}
                }
            }

            match commands[0].as_str() {
                "start" => {
                    return Ok(GridCommand::Start);
                }
                "random" => {
                    return Ok(GridCommand::Random);
                }
                "clear" => {
                    return Ok(GridCommand::Clear);
                }
                "exit" | "quit" | "q" | "e" => {
                    return Ok(GridCommand::Quit);
                }
                "help" => {
                    return Ok(GridCommand::Help);
                }
                _ => {}
            }

            let coords: Vec<&str> = commands[0].split(',').map(|s| s.trim()).collect();

            match coords.len() {
                len if len == 2 => {
                    let (row_ranges, col_ranges) = match Self::get_ranges(&commands[0]) {
                        Ok((rr, cr)) => (rr, cr),
                        Err(e) => {
                            return Err(e);
                        }
                    };

                    match Self::parse_ranges(row_ranges, col_ranges) {
                        Err(e) => Err(e),

                        Ok((row_range, col_range)) => Ok(GridCommand::Set(row_range, col_range)),
                    }
                }
                len if len > 2 => Err(format!("Unexpected position \'{}\'", coords[2])),
                _ => Err(format!("Invalid command \'{}\'", input)),
            }
        }

        /// Parses (row_ranges, col_ranges) -> Vec<String>s into Vec2<isize>s, or returns Err(String)
        fn parse_ranges(
            row_ranges: Vec<String>,
            col_ranges: Vec<String>,
        ) -> Result<(Vec2<isize>, Vec2<isize>), String> {
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
            Ok((
                Vec2::new(row_range1, row_range2),
                Vec2::new(col_range1, col_range2),
            ))
        }

        fn get_ranges(coords: &str) -> Result<(Vec<String>, Vec<String>), String> {
            let coords: Vec<&str> = coords.split(',').map(|s| s.trim()).collect();

            match coords.len() {
                len if len > 2 => {
                    return Err(format!("Unexpected position \'{}\'", coords[2]));
                }
                len if len <= 1 => {
                    return Err(format!("Not enough co-ordinates"));
                }
                _ => {}
            }

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
        // remove io::Result holy shit you retard
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
            if let Some(_) = self.grid.get_index_closed(pos) {
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

            loop {
                println!("Give a single co-ordinate in the format {}, {} to set/remove a cell or\n{}, {} where {}, {}, etc. are indices that will set/remove cells at those specified locations.\nType in {} to get the list of commands.",
                    "row".cyan().bold(),
                    "column".cyan().bold(),
                    "row1-row2".cyan().bold(),
                    "col1-col2".cyan().bold(),
                    "row1".cyan(),
                    "row2".cyan(),
                    "help".green().bold());
                let answer = Self::get_command()?;
                // Maybe only the setting of things should be seperated?
                match answer {
                    Ok(command) => match command {
                        GridCommand::Set(rr, cr) => {
                            for row in rr.first.min(rr.second)
                                ..=self.grid.rows.min(rr.first.max(rr.second))
                            {
                                for col in cr.first.min(cr.second)
                                    ..=self.grid.columns.min(cr.first.max(cr.second))
                                {
                                    self.set_cell(&GridVec2::new(row, col));
                                }
                            }
                        }
                        // Make the cody dry by making the offsets constant
                        GridCommand::Shape((rr, cr), shape) => match shape {
                            Shape::Square => {
                                for offset in SQUARE {
                                    for row in rr.first.min(rr.second)
                                        ..=self.grid.rows.min(rr.first.max(rr.second))
                                    {
                                        for col in cr.first.min(cr.second)
                                            ..=self.grid.columns.min(cr.first.max(cr.second))
                                        {
                                            self.set_cell(
                                                &GridVec2::new(row, col).from_offset(offset),
                                            );
                                        }
                                    }
                                }
                            }
                            Shape::Circle => {
                                for offset in CIRCLE {
                                    for row in rr.first.min(rr.second)
                                        ..=self.grid.rows.min(rr.first.max(rr.second))
                                    {
                                        for col in cr.first.min(cr.second)
                                            ..=self.grid.columns.min(cr.first.max(cr.second))
                                        {
                                            self.set_cell(
                                                &GridVec2::new(row, col).from_offset(offset),
                                            );
                                        }
                                    }
                                }
                            }
                            Shape::Glider => {
                                for offset in GLIDER {
                                    for row in rr.first.min(rr.second)
                                        ..=self.grid.rows.min(rr.first.max(rr.second))
                                    {
                                        for col in cr.first.min(cr.second)
                                            ..=self.grid.columns.min(cr.first.max(cr.second))
                                        {
                                            self.set_cell(
                                                &GridVec2::new(row, col).from_offset(offset),
                                            );
                                        }
                                    }
                                }
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
                        GridCommand::Resize => {
                            self.grid = Self::from_input()?.grid;
                        }
                        GridCommand::Start => {
                            break;
                        }
                        GridCommand::Quit => {
                            return Ok(());
                        }
                    },
                    Err(e) => {
                        eprintln!("{}", e.red().bold());
                        continue;
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
    }
}
