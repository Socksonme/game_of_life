use game_of_life::life::{ConwayEngine};

fn main() {
    let mut engine = ConwayEngine::new(60, 60);
    if let Err(err) = engine.run() {
        eprintln!("Error while getting input: {}", err);
        std::process::exit(1);
    }
}
