use game_of_life::life::{ConwayEngine, Vec2};

fn main() {
    // let mut engine = ConwayEngine::from_file("conway.ron");
    let mut engine = ConwayEngine::new(80, 80);
    engine.set_cell(&Vec2::new(40, 40));
    engine.set_cell(&Vec2::new(40, 41));
    engine.set_cell(&Vec2::new(41, 41));
    engine.set_cell(&Vec2::new(43, 39));
    engine.set_cell(&Vec2::new(43, 38));
    engine.set_cell(&Vec2::new(42, 38));
    engine.run();
}
