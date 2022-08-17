@echo off

cd game
cargo build --release --features "gol-internal gol-slow"
cd ..
copy .\game\target\release\gol_game.dll .\platform\target 
