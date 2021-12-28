use colored::*;

pub fn init_text() {
    println!("Give the number of rows and columns in the format of {}, {} that you want to be in the grid\nand then the type of the grid ({}/{})\n{},{} - {}",
        "row".cyan().bold(),
        "column".cyan().bold(),
        "open".cyan().bold(),
        "closed".cyan().bold(),
        "row".cyan().bold(),
        "column".cyan().bold(),
        "type".cyan().bold())
}

pub fn command_help_text() {
    println!("{}\n{} to start the game\n{} to randomize the board\n{} to clear the board\n{} for help\n{} to quit\n",
        "Commands:".blue(),
        "start".green().bold(),
        "random".green().bold(),
        "clear".green().bold(),
        "help".green().bold(),
        "q[uit]/e[xit]".green().bold())
}

pub fn run_text() {
    println!("Give a single co-ordinate in the format {}, {} to set/remove a cell or\n{}, {} where {}, {}, etc. are indices that will set/remove cells at those specified locations.\nType in {} to get the list of commands.",
        "row".cyan().bold(),
        "column".cyan().bold(),
        "row1-row2".cyan().bold(),
        "col1-col2".cyan().bold(),
        "row1".cyan(),
        "row2".cyan(),
        "help".green().bold())
}
