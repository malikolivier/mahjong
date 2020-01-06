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

    let quit = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let mut instant;
    'quit: loop {
        let mut siv = Cursive::default();
        let quit_ptr = quit.clone();
        siv.add_global_callback('q', move |_s| {
            quit_ptr.store(true, std::sync::atomic::Ordering::SeqCst)
        });

        let game = rx_game.recv().expect("Receive state");
        siv.add_layer(TextView::new(game.to_string_repr()));

        if game.turn == tiles::Fon::Ton {
            let mut dialog = Dialog::text("").title("Hand");
            for (i, hai) in game.player1_te().enumerate() {
                let tx_turn = tx_turn.clone();
                dialog = dialog.button(hai.to_string(), move |s| {
                    tx_turn
                        .send(ai::TurnResult::ThrowHai {
                            index: i,
                            riichi: false,
                        })
                        .expect("Sent turn result!");
                    s.quit();
                })
            }
            if let Some(hai) = game.player1_tsumo() {
                let tx_turn = tx_turn.clone();
                dialog = dialog.button(hai.to_string(), move |s| {
                    tx_turn
                        .send(ai::TurnResult::ThrowTsumoHai { riichi: false })
                        .expect("Sent turn result!");
                    s.quit();
                });
            }
            siv.add_layer(dialog);

            instant = None;
        } else {
            instant = Some(std::time::Instant::now());
        }

        loop {
            if quit.load(std::sync::atomic::Ordering::SeqCst) {
                break 'quit;
            }
            if let Some(instant) = instant {
                if std::time::Instant::now().duration_since(instant)
                    >= std::time::Duration::from_secs(1)
                {
                    siv.quit();
                }
            }
            if siv.is_running() {
                siv.step();
                std::thread::sleep(std::time::Duration::from_millis(30));
                siv.refresh();
            } else {
                break;
            }
        }
    }
}

struct CursiveHuman {
    rx_call: std::sync::mpsc::Receiver<Option<ai::Call>>,
    rx_turn: std::sync::mpsc::Receiver<ai::TurnResult>,
}
struct NullBot;

impl ai::AI for CursiveHuman {
    fn call(
        &self,
        _game: &game::Game,
        _player: tiles::Fon,
        _allowed_calls: &[ai::Call],
    ) -> Option<ai::Call> {
        None
    }
    fn do_turn(&self, _game: &game::Game, _player: tiles::Fon) -> ai::TurnResult {
        self.rx_turn.recv().expect("Receive result!")
    }
}
impl ai::AI for NullBot {
    fn call(
        &self,
        _game: &game::Game,
        _player: tiles::Fon,
        _allowed_calls: &[ai::Call],
    ) -> Option<ai::Call> {
        None
    }
    fn do_turn(&self, _game: &game::Game, _player: tiles::Fon) -> ai::TurnResult {
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
