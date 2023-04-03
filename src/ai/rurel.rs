use std::hash::{Hash, Hasher};

use rand::{rngs::StdRng, SeedableRng};
use rurel::{
    mdp::{Agent, State},
    strategy::{explore::RandomExploration, learn::QLearning, terminate::FixedIterations},
    AgentTrainer,
};

use crate::{
    game::{Game, GameRequest, PossibleActions, Request, ThrowableOnRiichi},
    tiles::Fon,
};

use super::{null_bot, AiServer, Call, PossibleCall, TehaiIndex, TurnResult};

#[derive(Clone)]
struct MyState {
    request: GameRequest,
}

impl PartialEq for MyState {
    /// Compare only what the current player knows of the state
    fn eq(&self, other: &Self) -> bool {
        unimplemented!("PartialEq")
    }
}
impl Eq for MyState {}
impl Hash for MyState {
    /// Only take into account what the current player knows about the state
    fn hash<H: Hasher>(&self, state: &mut H) {
        unimplemented!("Hash")
    }
}

#[derive(Clone, Hash, Eq, PartialEq)]
enum MyAction {
    NormalTurn(TurnResult),
    Call(Option<Call>),
    // wait for other players to do their turn
    Wait,
}

impl State for MyState {
    type A = MyAction;
    fn reward(&self) -> f64 {
        unimplemented!("reward")
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
}
impl Agent<MyState> for MyAgent {
    fn current_state(&self) -> &MyState {
        &self.state
    }
    fn take_action(&mut self, action: &MyAction) {
        // Change state according to action
        let game = &mut self.state.request.game;
        // Advance until next turn
        let mut channels = [null_bot(), null_bot(), null_bot(), null_bot()];
        let player = self.state.request.player;
        // Set training server
        let (server, client) = crate::ai::channel();

        let action = action.clone();
        std::thread::spawn(move || loop {
            let request = client.rx.recv().unwrap();
            match &request.request {
                Request::Call(_possible_calls) => {
                    if let MyAction::Call(call) = action {
                        client.tx_call.send(call).expect("Sent!");
                    } else {
                        unreachable!()
                    }
                }
                Request::DoTurn(_possible_actions) => {
                    if let MyAction::NormalTurn(result) = &action {
                        client.tx_turn.send(result.clone()).expect("Sent!");
                    } else {
                        unreachable!()
                    }
                }
                Request::EndGame => return,
                Request::Refresh => {}
                Request::DisplayScore { .. } => {}
            }
        });

        channels[player as usize] = server;
        let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
        game.play_hanchan(channels, &mut rng);
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
    let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
    let game = Game::new(&mut rng);

    let mut agent = MyAgent {
        state: MyState {
            request: GameRequest {
                game,
                request: Request::Refresh,
                player: Fon::Ton,
            },
        },
    };

    trainer.train(
        &mut agent,
        &QLearning::new(0.2, 0.01, 2.),
        &mut FixedIterations::new(100000),
        &RandomExploration::new(),
    );
}
