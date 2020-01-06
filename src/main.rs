use cursive::views::Dialog;
use cursive::views::TextView;
use cursive::Cursive;
use rand::{rngs::StdRng, SeedableRng};

mod ai;
mod game;
mod list;
mod tiles;
use std::sync::mpsc::channel;

fn main() {
    let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
    let (tx_call, rx_call) = channel();
    let (tx_turn, rx_turn) = channel();
    let (tx_game, rx_game) = channel();

    test_print_all_chars();

    std::thread::spawn(move || {
        let mut game = game::Game::new(&mut rng);

        game.play(
            &[
                Box::new(CursiveHuman { rx_call, rx_turn }),
                Box::new(NullBot),
                Box::new(NullBot),
                Box::new(NullBot),
            ],
            tx_game,
        );
    });

    let mut siv = Cursive::default();
    siv.add_global_callback('q', |s| s.quit());

    let game = rx_game.recv().expect("Receive initial state");
    siv.add_layer(TextView::new(game.to_string_repr()));

    let mut dialog = Dialog::text("").title("Hand");
    for (i, hai) in game.player1_te().enumerate() {
        dialog = dialog.button(hai.to_string(), move |s| ())
    }
    if let Some(hai) = game.player1_tsumo() {
        dialog = dialog.button(hai.to_string(), move |s| ());
    }
    siv.add_layer(dialog);

    siv.refresh();
    loop {
        if siv.is_running() {
            siv.step();
            std::thread::sleep(std::time::Duration::new(0, 100_000));
        } else {
            break;
        }
    }
}

struct CursiveHuman {
    rx_call: std::sync::mpsc::Receiver<Option<ai::Call>>,
    rx_turn: std::sync::mpsc::Receiver<ai::TurnResult>,
    // siv
}
struct NullBot;

impl ai::AI for CursiveHuman {
    fn call(
        &self,
        game: &game::Game,
        player: tiles::Fon,
        allowed_calls: &[ai::Call],
    ) -> Option<ai::Call> {
        None
    }
    fn do_turn(&self, game: &game::Game, player: tiles::Fon) -> ai::TurnResult {
        ai::TurnResult::ThrowTsumoHai { riichi: false }
    }
}
impl ai::AI for NullBot {
    fn call(
        &self,
        game: &game::Game,
        player: tiles::Fon,
        allowed_calls: &[ai::Call],
    ) -> Option<ai::Call> {
        None
    }
    fn do_turn(&self, game: &game::Game, player: tiles::Fon) -> ai::TurnResult {
        ai::TurnResult::ThrowTsumoHai { riichi: false }
    }
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
