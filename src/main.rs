extern crate rand;

use rand::{rngs::StdRng, SeedableRng};

mod game;
mod tiles;

fn main() {
    let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
    let game = game::Game::new(&mut rng);
    println!("{:?}", game);

    for hai in tiles::make_all_tiles().iter() {
        print!("{}", hai.into_char());
    }
    print!("{}", tiles::Hai::back_char());
    println!("{}", tiles::Hai::back_char());
}
