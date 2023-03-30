## How to build

```sh
# Install ncurses
sudo apt install libncurses-dev
# Build and run
cargo run --release
```

## How to debug?

    RUST_BACKTRACE=1 RUST_LOG=debug cargo run 2> err.out

## See AI plays against each other

1. Print the board on your terminal on every refresh:

```diff
diff --git a/src/ai.rs b/src/ai.rs
index 88b2b92..546c9c4 100644
--- a/src/ai.rs
+++ b/src/ai.rs
@@ -85,6 +85,7 @@ impl AiServer {
                     client.tx_turn.send(result).expect("Sent!")
                 }
                 Request::EndGame => return,
+                Request::Refresh => println!("{}", request.game),
                 _ => {}
             }
         });
```

2. Run it

```sh
cargo run --release -- --p1 dumb-caller-bot --p2 dumb-caller-bot
```

## Convenient for testing yaku

http://tobakushi.net/mahjang/keisanex.html
