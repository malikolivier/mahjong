use env_logger;
use log::{debug, info};
use ron;

use cursive::views::Dialog;
use cursive::views::TextView;
use cursive::Cursive;
use rand::{rngs::StdRng, SeedableRng};

mod ai;
mod game;
mod list;
mod tiles;
mod yaku;

use ai::TehaiIndex;

fn main() {
    let mut log_builder = env_logger::Builder::from_default_env();
    log_builder.target(env_logger::Target::Stderr).init();

    test_print_all_chars();

    let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
    let mut game = game::Game::new(&mut rng);
    // let mut game: game::Game =
    //     ron::de::from_reader(std::fs::File::open("in.ron").unwrap()).unwrap();
    game.play_hanchan(
        [
            cursive_human(),
            ai::null_bot(),
            ai::null_bot(),
            ai::null_bot(),
        ],
        &mut rng,
    );
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

            let score = game.score_repr();
            let board = game.to_string_repr();
            let display = format!("{}\n\n{}", score, board);
            siv.add_layer(TextView::new(display));

            match request {
                game::Request::Refresh => {
                    snapshot(&game);
                    instant = Some(std::time::Instant::now());
                }
                game::Request::Call(calls) => {
                    let mut dialog = Dialog::text("").title("Call?");
                    let tx_call_ = tx_call.clone();
                    dialog = dialog.button("Pass", move |s| {
                        tx_call_.send(None).expect("Sent call result!");
                        s.quit();
                    });

                    for call in calls {
                        match call {
                            ai::PossibleCall::Chi {
                                indices: possible_chis,
                            } => {
                                for chi in possible_chis {
                                    let tx_call = tx_call.clone();
                                    let tile1 = game
                                        .player_te(tiles::Fon::Ton)
                                        .nth(chi[0])
                                        .expect("Has tile");
                                    let tile2 = game
                                        .player_te(tiles::Fon::Ton)
                                        .nth(chi[1])
                                        .expect("Has tile");
                                    dialog = dialog.button(
                                        format!("Chi {}{}", tile1.to_string(), tile2.to_string()),
                                        move |s| {
                                            tx_call
                                                .send(Some(ai::Call::Chi { index: chi }))
                                                .expect("Sent call result!");
                                            s.quit();
                                        },
                                    );
                                }
                            }
                            ai::PossibleCall::Pon => {
                                let tx_call = tx_call.clone();
                                dialog = dialog.button("Pon", move |s| {
                                    debug!("Send pon call");
                                    tx_call
                                        .send(Some(ai::Call::Pon))
                                        .expect("Sent call result!");
                                    s.quit();
                                });
                            }
                            ai::PossibleCall::Kan => {
                                let tx_call = tx_call.clone();
                                dialog = dialog.button("Kan", move |s| {
                                    tx_call
                                        .send(Some(ai::Call::Kan))
                                        .expect("Sent call result!");
                                    s.quit();
                                });
                            }
                            ai::PossibleCall::Ron => {
                                let tx_call = tx_call.clone();
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
                game::Request::DoTurn {
                    can_tsumo,
                    can_riichi,
                    can_kyusyukyuhai,
                    can_shominkan,
                    can_ankan,
                } => {
                    let mut dialog = Dialog::text("").title("Hand");
                    if can_tsumo {
                        let tx_turn = tx_turn.clone();
                        dialog = dialog.button("Tsumo", move |s| {
                            tx_turn
                                .send(ai::TurnResult::Tsumo)
                                .expect("Sent turn result!");
                            s.quit();
                        })
                    }
                    if can_kyusyukyuhai {
                        let tx_turn = tx_turn.clone();
                        dialog = dialog.button("Kyushukyuhai", move |s| {
                            tx_turn
                                .send(ai::TurnResult::Kyusyukyuhai)
                                .expect("Sent turn result!");
                            s.quit();
                        })
                    }
                    if !can_riichi.is_empty() {
                        for throwable in can_riichi {
                            let tx_turn = tx_turn.clone();
                            match throwable {
                                game::ThrowableOnRiichi::Te(index) => {
                                    let tile = game
                                        .player_te(tiles::Fon::Ton)
                                        .nth(index)
                                        .expect("Has tile");
                                    dialog = dialog.button(
                                        format!("Riichi {}", tile.to_string()),
                                        move |s| {
                                            tx_turn
                                                .send(ai::TurnResult::ThrowHai {
                                                    index: TehaiIndex::Tehai(index),
                                                    riichi: true,
                                                })
                                                .expect("Sent turn result!");
                                            s.quit();
                                        },
                                    )
                                }
                                game::ThrowableOnRiichi::Tsumohai => {
                                    let tile =
                                        game.player_tsumo(tiles::Fon::Ton).expect("Has tsumohai");
                                    dialog = dialog.button(
                                        format!("Riichi {}", tile.to_string()),
                                        move |s| {
                                            tx_turn
                                                .send(ai::TurnResult::ThrowHai {
                                                    index: TehaiIndex::Tsumohai,
                                                    riichi: true,
                                                })
                                                .expect("Sent turn result!");
                                            s.quit();
                                        },
                                    )
                                }
                            };
                        }
                    }
                    if !can_ankan.is_empty() {
                        for hai in can_ankan {
                            let tx_turn = tx_turn.clone();
                            let index = game
                                .player_te_(tiles::Fon::Ton)
                                .index(hai)
                                .expect("Has ankan tile");
                            dialog = dialog.button(format!("AnKan {}", hai.to_string()), move |s| {
                                tx_turn
                                    .send(ai::TurnResult::Ankan { index })
                                    .expect("Sent turn result!");
                                s.quit();
                            })
                        }
                    }
                    if !can_shominkan.is_empty() {
                        for hai in can_shominkan {
                            let tx_turn = tx_turn.clone();
                            let index = game
                                .player_te_(tiles::Fon::Ton)
                                .index(hai)
                                .expect("Has ankan tile");
                            dialog = dialog.button(format!("Kakan {}", hai.to_string()), move |s| {
                                tx_turn
                                    .send(ai::TurnResult::Kakan { index })
                                    .expect("Sent turn result!");
                                s.quit();
                            })
                        }
                    }
                    if !game.player_is_riichi(tiles::Fon::Ton) {
                        for (i, hai) in game.player_te(tiles::Fon::Ton).enumerate() {
                            let tx_turn = tx_turn.clone();
                            dialog = dialog.button(hai.to_string(), move |s| {
                                tx_turn
                                    .send(ai::TurnResult::ThrowHai {
                                        index: TehaiIndex::Tehai(i),
                                        riichi: false,
                                    })
                                    .expect("Sent turn result!");
                                s.quit();
                            })
                        }
                    }
                    if let Some(hai) = game.player_tsumo(tiles::Fon::Ton) {
                        let tx_turn = tx_turn.clone();
                        dialog = dialog.button(hai.to_string(), move |s| {
                            tx_turn
                                .send(ai::TurnResult::ThrowHai {
                                    index: TehaiIndex::Tsumohai,
                                    riichi: false,
                                })
                                .expect("Sent turn result!");
                            s.quit();
                        });
                    }
                    siv.add_layer(dialog);

                    instant = None;
                }

                game::Request::DisplayScore(result) => {
                    use game::KyokuResult;
                    let mut display = String::new();
                    match result {
                        KyokuResult::Agari { winners, .. } => {
                            for winner in winners {
                                info!("Player {} won with {:?}", winner.0 as usize, &winner.1);
                                let yaku: Vec<_> =
                                    winner.1.into_iter().map(|yaku| yaku.name()).collect();
                                display.push_str(&format!(
                                    "{} won with {:?}\n",
                                    winner.0.to_char(),
                                    yaku
                                ));
                            }
                        }
                        KyokuResult::Ryukyoku => {
                            display.push_str("Ryukyoku");
                        }
                    }
                    let mut dialog = Dialog::text(display).title("End");
                    dialog = dialog.button("OK", |s| s.quit());
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

fn snapshot(game: &game::Game) {
    use std::io::Write;

    let time = std::time::SystemTime::now();
    if let Ok(mut f) = std::fs::File::create(format!("snapshot/{:?}.ron", time)) {
        if let Ok(s) = ron::ser::to_string_pretty(&game, Default::default()) {
            let _ = f.write_all(s.as_bytes());
        }
    }
}
