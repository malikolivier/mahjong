use cursive::views::TextView;
use cursive::Cursive;
use rand::{rngs::StdRng, SeedableRng};

mod game;
mod tiles;

fn main() {
    let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
    let mut game = game::Game::new(&mut rng);
    println!("{:?}", &game);
    println!("{}", game.to_string_repr());

    test_print_all_chars();

    let mut siv = Cursive::default();
    siv.add_global_callback('q', |s| s.quit());
    // siv.add_layer(TextView::new("Hello cursive! Press <q> to quit."));
    game.deal();
    siv.add_layer(TextView::new(game.to_string_repr()));
    siv.run();
}

fn test_print_all_chars() {
    for hai in tiles::make_all_tiles().iter() {
        print!("{}", hai.to_string());
    }
    print!("{}", tiles::Hai::back_char());
    println!("{}", tiles::Hai::back_char());

    print!("{}", game::Dice::One.into_char());
    print!("{}", game::Dice::Two.into_char());
    print!("{}", game::Dice::Three.into_char());
    print!("{}", game::Dice::Four.into_char());
    print!("{}", game::Dice::Five.into_char());
    println!("{}", game::Dice::Six.into_char());
}
