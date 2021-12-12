use game_of_life::engine::ConwayEngine;

fn main() {
    match ConwayEngine::from_input() {
        Ok(mut engine) => {
            if let Err(err) = engine.run() {
                eprintln!("Error while getting input: {}", err);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error while getting input: {}", e);
            std::process::exit(1);
        }
    }
}
