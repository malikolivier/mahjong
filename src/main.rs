use cursive::Cursive;
use cursive::views::TextView;
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

    let mut siv = Cursive::default();
    siv.add_global_callback('q', |s| s.quit());
    siv.add_layer(TextView::new("Hello cursive! Press <q> to quit."));
    siv.run();
}
