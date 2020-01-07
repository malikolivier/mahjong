use env_logger;
use log::debug;

use cursive::views::Dialog;
use cursive::views::TextView;
use cursive::Cursive;
use rand::{rngs::StdRng, SeedableRng};

mod ai;
mod game;
mod list;
mod tiles;

fn main() {
    let mut log_builder = env_logger::Builder::from_default_env();
    log_builder.target(env_logger::Target::Stderr).init();

    let mut rng: StdRng = SeedableRng::from_seed([0; 32]);

    test_print_all_chars();

    let mut game = game::Game::new(&mut rng);
    game.play([
        cursive_human(),
        ai::null_bot(),
        ai::null_bot(),
        ai::null_bot(),
    ]);
}

fn cursive_human() -> ai::AiServer {
    let (server, client) = ai::channel();
    std::thread::spawn(move || {
        debug!("Init..");
        let rx = client.rx;
        let tx_call = client.tx_call;
        let tx_turn = client.tx_turn;

        let quit = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let mut instant;
        debug!("Done..");
        'quit: loop {
            debug!("Start frame...");

            let mut siv = Cursive::default();
            let quit_ptr = quit.clone();
            siv.add_global_callback('q', move |_s| {
                quit_ptr.store(true, std::sync::atomic::Ordering::SeqCst)
            });

            debug!("Waiting for request...");
            let game::GameRequest { game, request } = rx.recv().expect("Receive state");
            debug!(
                "Received {:?} for game: \n{}\n",
                &request,
                game.to_string_repr()
            );

            siv.add_layer(TextView::new(game.to_string_repr()));

            match request {
                game::Request::Refresh => {
                    instant = Some(std::time::Instant::now());
                }
                game::Request::Call(calls) => {
                    tx_call.send(None).expect("Sent call!");
                    let mut dialog = Dialog::text("").title("Call?");
                    let tx_call_ = tx_call.clone();
                    dialog = dialog.button("Pass", move |s| {
                        tx_call_.send(None).expect("Sent call result!");
                        s.quit();
                    });

                    for call in calls {
                        let tx_call = tx_call.clone();
                        match call {
                            ai::PossibleCall::Chi => {
                                dialog = dialog.button("Chi", move |s| {
                                    tx_call
                                        .send(Some(ai::Call::Chi { index: 0 })) // TODO
                                        .expect("Sent call result!");
                                    s.quit();
                                });
                            }
                            ai::PossibleCall::Pon => {
                                dialog = dialog.button("Pon", move |s| {
                                    tx_call
                                        .send(Some(ai::Call::Pon))
                                        .expect("Sent call result!");
                                    s.quit();
                                });
                            }
                            ai::PossibleCall::Kan => {
                                dialog = dialog.button("Kan", move |s| {
                                    tx_call
                                        .send(Some(ai::Call::Kan))
                                        .expect("Sent call result!");
                                    s.quit();
                                });
                            }
                            ai::PossibleCall::Ron => {
                                dialog = dialog.button("Ron", move |s| {
                                    tx_call
                                        .send(Some(ai::Call::Ron))
                                        .expect("Sent call result!");
                                    s.quit();
                                });
                            }
                        };
                    }
                    siv.add_layer(dialog);

                    instant = None;
                }
                game::Request::DoTurn => {
                    let mut dialog = Dialog::text("").title("Hand");
                    for (i, hai) in game.player_te(tiles::Fon::Ton).enumerate() {
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
                    if let Some(hai) = game.player_tsumo(tiles::Fon::Ton) {
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
                }
            }

            loop {
                if quit.load(std::sync::atomic::Ordering::SeqCst) {
                    debug!("Quitting...");
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
        debug!("Quitted...");
    });

    server
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
