for i in 60 300 900 1800 3600 7200 14400 21600 43200 86400 259200 604800; do
  cargo run -- bitflyer btcfxjpy $i;
  cargo run -- bitflyer btcjpy $i;
  cargo run -- liquid btcjpy $i;
  cargo run -- ftx btcusd $i;
  cargo run -- bitmex btcusd-perpetual-futures $i;
done
