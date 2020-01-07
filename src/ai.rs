use super::game::{GameRequest, Request};

#[derive(Debug, Copy, Eq, PartialEq, PartialOrd, Ord, Clone)]
pub enum Call {
    /// Call a Chi. Includes the index of a tile in the chi.
    Chi { index: usize },
    /// Call a Pon
    Pon,
    /// Call a Kan
    Kan,
    /// Call a Ron
    Ron,
}

#[derive(Debug, Copy, Eq, PartialEq, PartialOrd, Ord, Clone)]
pub enum PossibleCall {
    Chi,
    Pon,
    Kan,
    Ron,
}

pub enum TurnResult {
    ThrowHai { index: usize, riichi: bool },
    ThrowTsumoHai { riichi: bool },
    Tsumo,
    Kyusyukyuhai,
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

pub fn null_bot() -> AiServer {
    let (server, client) = channel();
    std::thread::spawn(move || loop {
        let request = client.rx.recv().unwrap();
        match request {
            GameRequest {
                request: Request::Call(..),
                ..
            } => {
                client.tx_call.send(None).expect("Sent!");
            }
            GameRequest {
                request: Request::DoTurn,
                ..
            } => client
                .tx_turn
                .send(TurnResult::ThrowTsumoHai { riichi: false })
                .expect("Sent!"),
            _ => {}
        }
    });
    server
}
