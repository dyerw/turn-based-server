### multi-chess

My next foray into Rust game dev is multiplayer networking. The goal is a cheat-proof server-authoritative implementation of chess.

Of course, turn based games don't have all the problems of prediction that real-time games do which is a blessing.

The backend is a hand-rolled game binary protocol over TCP implemented in tokio. I got tired of dealing with mutexes pretty quick so it's going to implement an actor model using actix. 

This should work?