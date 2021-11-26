use game_of_life::life::ConwayEngine;
fn main() {
    let mut engine = ConwayEngine::new(1, 1);
    engine.run();
}
