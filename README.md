# multi-chess

My next foray into Rust game dev is multiplayer networking. The goal is a cheat-proof server-authoritative implementation of chess.

Of course, turn based games don't have all the problems of prediction that real-time games do which is a blessing.

## Using
- actix for actor model support
- msgpack for binary protocol
- nom for TCP message parsing
- serde for msg serialization/deserializtion

This should work?