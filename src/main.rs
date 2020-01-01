use cursive::views::Dialog;
use cursive::views::TextView;
use cursive::Cursive;
use rand::{rngs::StdRng, SeedableRng};

mod ai;
mod game;
mod list;
mod tiles;

static mut GAME: Option<game::Game> = None;

fn game() -> &'static mut game::Game {
    unsafe { GAME.as_mut().unwrap() }
}
fn init() {
    let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
    let game = game::Game::new(&mut rng);
    unsafe {
        GAME = Some(game);
    }
}

fn main() {
    init();
    let game = game();
    println!("{:?}", &game);
    println!("{}", game.to_string_repr());

    test_print_all_chars();

    let mut siv = Cursive::default();
    siv.add_global_callback('q', |s| s.quit());
    // siv.add_layer(TextView::new("Hello cursive! Press <q> to quit."));
    game.deal();
    run(&mut siv);

    siv.run();
}

fn run(siv: &mut Cursive) {
    let game = game();
    siv.add_layer(TextView::new(game.to_string_repr()));

    let mut dialog = Dialog::text("").title("Hand");
    for (i, hai) in game.player1_te().enumerate() {
        dialog = dialog.button(hai.to_string(), move |s| discard(s, i))
    }
    if let Some(hai) = game.player1_tsumo() {
        dialog = dialog.button(hai.to_string(), move |s| discard_tsumo(s));
    }
    siv.add_layer(dialog);
}

fn discard(s: &mut Cursive, i: usize) {
    let game = game();
    game.throw_tile(i, false);
    // TODO: Do stuff for game to continue
    s.pop_layer();
    run(s);
}
fn discard_tsumo(s: &mut Cursive) {
    let game = game();
    game.throw_tsumo(false);
    s.pop_layer();
    run(s);
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
