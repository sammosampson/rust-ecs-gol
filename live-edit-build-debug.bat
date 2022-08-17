@echo off

cd game
cargo build --features "gol-internal gol-slow"
cd ..
copy .\game\target\debug\gol_game.dll .\platform\target 
copy .\game\target\debug\gol_game.pdb .\platform\target 
