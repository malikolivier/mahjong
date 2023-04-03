use std::{
    hash::{Hash, Hasher},
    mem::MaybeUninit,
    ops::DerefMut,
    rc::Rc,
    sync::{mpsc::Sender, Arc, Mutex},
    thread::JoinHandle, io::Write,
};

use rand::{rngs::StdRng, SeedableRng};
use rurel::{
    mdp::{Agent, State},
    strategy::{explore::RandomExploration, learn::QLearning, terminate::FixedIterations},
    AgentTrainer,
};
use serde::{Deserialize, Serialize};

use crate::{
    game::{self, Game, GameRequest, PossibleActions, Request, ThrowableOnRiichi, count_shanten},
    tiles::Fon,
};

use super::{null_bot, AiClient, AiServer, Call, PossibleCall, TehaiIndex, TurnResult};

#[derive(Clone, Serialize, Deserialize)]
struct MyState {
    request: GameRequest,
}

impl PartialEq for MyState {
    /// Compare only what the current player knows of the state
    fn eq(&self, other: &Self) -> bool {
        let p = self.request.player;
        self.request
            .game
            .known_game(p)
            .eq(&other.request.game.known_game(p))
    }
}
impl Eq for MyState {}
impl Hash for MyState {
    /// Only take into account what the current player knows about the state
    fn hash<H: Hasher>(&self, state: &mut H) {
        let p = self.request.player;
        self.request.game.known_game(p).hash(state)
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
enum MyAction {
    NormalTurn(TurnResult),
    Call(Option<Call>),
    // wait for other players to do their turn
    Wait,
}

impl State for MyState {
    type A = MyAction;
    fn reward(&self) -> f64 {
        let score = self.request.game.player_score(self.request.player) as f64;

        let te = self.request.game.player_te_(self.request.player);
        let hai_all: Vec<_> = te.hai_closed_all().collect();
        let shanten = count_shanten(&hai_all) as f64;
        dbg!(score - shanten)
    }
    fn actions(&self) -> Vec<MyAction> {
        // List possible, legal actions for each game state
        let GameRequest {
            game,
            request,
            player,
        } = &self.request;
        match request {
            Request::Call(possible_calls) => {
                let mut actions = vec![MyAction::Call(None)];
                for call in possible_calls {
                    match call {
                        PossibleCall::Ron => {
                            actions.push(MyAction::Call(Some(Call::Ron)));
                        }
                        PossibleCall::Kan => {
                            actions.push(MyAction::Call(Some(Call::Kan)));
                        }
                        PossibleCall::Pon => {
                            actions.push(MyAction::Call(Some(Call::Pon)));
                        }
                        PossibleCall::Chi { indices } => {
                            for index in indices {
                                actions.push(MyAction::Call(Some(Call::Chi { index: *index })));
                            }
                        }
                    };
                }
                actions
            }
            Request::DoTurn(PossibleActions {
                can_tsumo,
                can_riichi,
                can_kyusyukyuhai,
                can_shominkan,
                can_ankan,
            }) => {
                let mut actions = vec![];
                if *can_tsumo {
                    actions.push(MyAction::NormalTurn(TurnResult::Tsumo));
                }

                if !can_riichi.is_empty() {
                    for throwable_tile in can_riichi {
                        let index = match throwable_tile {
                            ThrowableOnRiichi::Te(index) => TehaiIndex::Tehai(*index),
                            ThrowableOnRiichi::Tsumohai => TehaiIndex::Tsumohai,
                        };
                        actions.push(MyAction::NormalTurn(TurnResult::ThrowHai {
                            index,
                            riichi: true,
                        }));
                    }
                }

                if *can_kyusyukyuhai {
                    actions.push(MyAction::NormalTurn(TurnResult::Kyusyukyuhai));
                }

                if !can_ankan.is_empty() {
                    for tile in can_ankan {
                        let index = game
                            .player_te_(*player)
                            .index(*tile)
                            .expect("Has ankan tile");
                        actions.push(MyAction::NormalTurn(TurnResult::Ankan { index }));
                    }
                }

                if let Some(hai) = can_shominkan.first() {
                    let index = game
                        .player_te_(*player)
                        .index(*hai)
                        .expect("Has kakan tile");
                    actions.push(MyAction::NormalTurn(TurnResult::Kakan { index }));
                }

                let te = game.player_te_(*player);
                for i in 0..te.hai.len() {
                    actions.push(MyAction::NormalTurn(TurnResult::ThrowHai {
                        index: TehaiIndex::Tehai(i),
                        riichi: false,
                    }));
                }
                if te.get(TehaiIndex::Tsumohai).is_some() {
                    actions.push(MyAction::NormalTurn(TurnResult::ThrowHai {
                        index: TehaiIndex::Tsumohai,
                        riichi: false,
                    }));
                }

                actions
            }
            // Otherwise, do nothing
            _ => vec![MyAction::Wait],
        }
    }
}

struct MyAgent {
    state: MyState,
    client: AiClient,
    hanchan_done: usize,
    // tx_call: Sender<Option<Call>>,
    // tx_turn: Sender<TurnResult>,
    // game_thread: MaybeUninit<JoinHandle<()>>,
    // ai_thread: MaybeUninit<JoinHandle<()>>,
}
impl Agent<MyState> for MyAgent {
    fn current_state(&self) -> &MyState {
        &self.state
    }
    fn take_action(&mut self, action: &MyAction) {
        // Change state according to action

        let request = &self.state.request;
        println!("RUREL::TRAIN: action={:?}", action);
        if let MyAction::Wait = action {
            // let the game be played without interfering
        } else {
            println!("RUREL::TRAIN: request={:?}", request.request);
            println!(
                "RUREL::TRAIN: te=    {:?}",
                request.game.player_te_(request.player)
            );
            match &request.request {
                Request::Call(_possible_calls) => {
                    if let MyAction::Call(call) = action {
                        self.client.tx_call.send(*call).expect("Sent!");
                    } else {
                        unreachable!()
                    }
                }
                Request::DoTurn(_possible_actions) => {
                    if let MyAction::NormalTurn(result) = &action {
                        self.client.tx_turn.send(result.clone()).expect("Sent!");
                    } else {
                        unreachable!()
                    }
                }
                Request::EndGame => {}
                Request::Refresh => {}
                Request::DisplayScore { .. } => {}
            }
        }

        // Advance until next turn
        let request = self.client.rx.recv().unwrap();
        self.state.request = request;

        if let GameRequest {
            request: Request::EndGame,
            ..
        } = &self.state.request
        {
            // Reset game to do another hanchan
            self.hanchan_done += 1;

            let (server, client) = crate::ai::channel();
            self.client = client;

            let mut rng: StdRng = SeedableRng::from_seed([self.hanchan_done as u8; 32]);
            let channels = [server, null_bot(), null_bot(), null_bot()];
            let mut game = Game::new(&mut rng);

            std::thread::spawn(move || {
                game.play_hanchan(channels, &mut rng);
            });
        }
    }
}

pub fn handle_call(possible_calls: &[PossibleCall], _: &GameRequest) -> Option<Call> {
    unimplemented!()
}

pub fn handle_turn(possible_actions: &PossibleActions, request: &GameRequest) -> TurnResult {
    unimplemented!()
}

pub fn train() {
    let mut trainer = AgentTrainer::new();

    let (server, client) = crate::ai::channel();
    // let rx = client.rx;
    // let mut agent = //Arc::new(Mutex::new(
    //     MyAgent {
    //     state: MyState {
    //         request: GameRequest {
    //             game: Game::default(),
    //             request: Request::Refresh,
    //             player: Fon::Ton,
    //         },
    //     },
    //     client,
    //     // tx_call: client.tx_call,
    //     // tx_turn: client.tx_turn,
    // }
    //))

    // let agent2 = agent.clone();
    // let ai_thread = std::thread::spawn(move || {
    //     let request = rx.recv().unwrap();
    //     let mut agent = agent2.lock().unwrap();
    //     agent.state.request = request;
    //     // Now game thread waits for response!
    //     // We will send it with Agent::take_action
    // });

    let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
    let channels = [server, null_bot(), null_bot(), null_bot()];
    let mut game = Game::new(&mut rng);

    let game_thread = std::thread::spawn(move || {
        game.play_hanchan(channels, &mut rng);
    });

    // wait for first receipt
    let request = client.rx.recv().unwrap();
    let mut agent = MyAgent {
        state: MyState { request },
        client,
        hanchan_done: 0,
    };

    // let mut a = agent.lock().unwrap();
    // a.ai_thread.write(ai_thread);
    // unsafe { a.ai_thread.assume_init() };
    // a.game_thread.write(game_thread);
    // unsafe { a.game_thread.assume_init() };
    trainer.train(
        // a.deref_mut(),
        &mut agent,
        &QLearning::new(0.2, 0.01, 2.),
        &mut FixedIterations::new(100000),
        &RandomExploration::new(),
    );

    let learnings = trainer.export_learned_values();
    let out = ron::ser::to_string(&learnings).expect("Saved content");
    let mut f = std::fs::File::create("learnings.ron").expect("Create file");
    f.write_all(out.as_bytes()).unwrap();

    println!("END SUCCESS");
}
