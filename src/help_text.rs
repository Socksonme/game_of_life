use colored::*;

pub fn init_text() -> String {
    format!("Give the number of rows and columns in the format of {}, {} that you want to be in the grid\nand then the type of the grid ({}/{})\n{},{} - {}",
        "row".cyan().bold(),
        "column".cyan().bold(),
        "open".cyan().bold(),
        "closed".cyan().bold(),
        "row".cyan().bold(),
        "column".cyan().bold(),
        "type".cyan().bold())
}