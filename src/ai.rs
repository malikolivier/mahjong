use super::game::{GameRequest, KyokuResult, PossibleActions, Request, ThrowableOnRiichi};
use log::trace;

mod naive;

#[derive(Debug, Copy, Eq, PartialEq, PartialOrd, Ord, Clone)]
pub enum Call {
    /// Call a Chi. Includes the index of the tiles in the chi.
    Chi { index: [usize; 2] },
    /// Call a Pon
    Pon,
    /// Call a Kan
    Kan,
    /// Call a Ron
    Ron,
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone)]
pub enum PossibleCall {
    Chi { indices: Vec<[usize; 2]> },
    Pon,
    Kan,
    Ron,
}

pub enum TurnResult {
    ThrowHai { index: TehaiIndex, riichi: bool },
    Tsumo,
    Kyusyukyuhai,
    Kakan { index: TehaiIndex },
    Ankan { index: TehaiIndex },
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum TehaiIndex {
    Tehai(usize),
    Tsumohai,
}

pub struct AiServer {
    pub tx: std::sync::mpsc::Sender<GameRequest>,
    pub rx_call: std::sync::mpsc::Receiver<Option<Call>>,
    pub rx_turn: std::sync::mpsc::Receiver<TurnResult>,
}
pub struct AiClient {
    pub rx: std::sync::mpsc::Receiver<GameRequest>,
    pub tx_call: std::sync::mpsc::Sender<Option<Call>>,
    pub tx_turn: std::sync::mpsc::Sender<TurnResult>,
}

pub fn channel() -> (AiServer, AiClient) {
    let (tx_call, rx_call) = std::sync::mpsc::channel();
    let (tx_turn, rx_turn) = std::sync::mpsc::channel();
    let (tx, rx) = std::sync::mpsc::channel();
    (
        AiServer {
            tx,
            rx_call,
            rx_turn,
        },
        AiClient {
            rx,
            tx_call,
            tx_turn,
        },
    )
}

pub type CallHandler = fn(possible_calls: &[PossibleCall], request: &GameRequest) -> Option<Call>;
pub type TurnHandler = fn(possible_actions: &PossibleActions, request: &GameRequest) -> TurnResult;

impl AiServer {
    pub fn new(handle_call: CallHandler, handle_turn: TurnHandler) -> AiServer {
        let (server, client) = channel();

        std::thread::spawn(move || loop {
            let request = client.rx.recv().unwrap();
            trace!("State: \n{}\n", &request.game.to_string_repr());

            match &request.request {
                Request::Call(possible_calls) => {
                    let call = handle_call(possible_calls, &request);
                    client.tx_call.send(call).expect("Sent!");
                }
                Request::DoTurn(possible_actions) => {
                    let result = handle_turn(possible_actions, &request);
                    client.tx_turn.send(result).expect("Sent!")
                }
                Request::EndGame => return,
                Request::Refresh => println!("{}", request.game),
                Request::DisplayScore(result) => match result {
                    KyokuResult::Agari { winners, .. } => {
                        for winner in winners {
                            println!("KYOKU END: Player {} won with {:?}", winner.0 as usize, &winner.1);
                        }
                    }
                    KyokuResult::Ryukyoku { .. } => {
                        println!("KYOKU END: Ryukyoku");
                    }
                },
            }
        });

        server
    }
}

impl Default for AiServer {
    fn default() -> Self {
        null_bot()
    }
}

/// Dumb AI that is a simple drawing machine
pub fn null_bot() -> AiServer {
    AiServer::new(
        // Never call
        |_, _| None,
        // Always throw the drawn tile without doing anything
        |_, _| TurnResult::ThrowHai {
            index: TehaiIndex::Tsumohai,
            riichi: false,
        },
    )
}

/// Dumb AI that calls whenever it can
///
/// Convenient for testing calls.
pub fn dump_caller_bot() -> AiServer {
    AiServer::new(
        // Always call
        |possible_calls, _| {
            let mut calls = Vec::from(possible_calls);

            // Order possibles calls by priority
            calls.sort_by_key(|c| match c {
                PossibleCall::Ron => 1,
                PossibleCall::Kan => 2,
                PossibleCall::Pon => 3,
                PossibleCall::Chi { .. } => 4,
            });

            // Always call the call with the highest priority
            calls.first().map(|c| match c {
                PossibleCall::Ron => Call::Ron,
                PossibleCall::Kan => Call::Kan,
                PossibleCall::Pon => Call::Pon,
                PossibleCall::Chi { indices } => Call::Chi { index: indices[0] },
            })
        },
        // Always do whatever they can co, else, just throw the drawn tile
        |PossibleActions {
             can_tsumo,
             can_riichi,
             can_kyusyukyuhai,
             can_shominkan,
             can_ankan,
         },
         GameRequest { game, player, .. }| {
            if *can_tsumo {
                return TurnResult::Tsumo;
            }

            if !can_riichi.is_empty() {
                let index = match can_riichi[0] {
                    ThrowableOnRiichi::Te(index) => TehaiIndex::Tehai(index),
                    ThrowableOnRiichi::Tsumohai => TehaiIndex::Tsumohai,
                };
                return TurnResult::ThrowHai {
                    index,
                    riichi: true,
                };
            }

            if *can_kyusyukyuhai {
                return TurnResult::Kyusyukyuhai;
            }

            if !can_ankan.is_empty() {
                let hai = can_ankan[0];
                let index = game.player_te_(*player).index(hai).expect("Has ankan tile");
                return TurnResult::Ankan { index };
            }

            if let Some(hai) = can_shominkan.first() {
                let index = game
                    .player_te_(*player)
                    .index(*hai)
                    .expect("Has kakan tile");
                return TurnResult::Kakan { index };
            }

            // Else, throw a tile (tsumohai first)
            let te = game.player_te_(*player);
            let index = if te.get(TehaiIndex::Tsumohai).is_some() {
                // We can only throw tsumo hai if it exists!
                TehaiIndex::Tsumohai
            } else {
                // Throw the first tile
                TehaiIndex::Tehai(0)
            };
            TurnResult::ThrowHai {
                index,
                riichi: false,
            }
        },
    )
}

/// WIP
pub fn naive() -> AiServer {
    AiServer::new(naive::handle_call, naive::handle_turn)
}
