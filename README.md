# lastfm-rust

a lastfm "proxy" written in rust, similar to https://github.com/circularsprojects/lastfm-status but with websockets

this app polls your lastfm listening status every 5 seconds, and when a change is detected publishes an update to all clients connected to the websocket

## environment variables

```
PORT=3000 (this is optional, but by default is 3000)
LASTFM_API_KEY=whatever (self explanatory. required)
LASTFM_USERNAME=circular_ (also self explanatory. also required)
```

setting `RUST_LOG` to `"info,lastfm_rust=debug,tower_http=debug"` will also give you some useful debug logs