use super::game::{GameRequest, PossibleActions, Request};
use log::trace;

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
                _ => {}
            }
        });

        server
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
