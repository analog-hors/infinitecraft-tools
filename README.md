# `infinitecraft-tools`

Miscellaneous tools for InfiniteCraft routing. Thanks to @StarGazingHomies for figuring out much of the optimizations implemented in this program.

```
$ cargo run --release -- --help
   Compiling infinitecraft-tools v0.1.0 (/home/analog_hors/Documents/GitHub/infinitecraft-tools)
    Finished release [optimized] target(s) in 1.49s
     Running `target/release/infinitecraft-tools --help`
Miscellaneous tools for InfiniteCraft routing

Usage: infinitecraft-tools <COMMAND>

Commands:
  bfs    Do a Breadth-First Search of the state space to find optimal routes (memory intensive, prefer IDDFS)
  iddfs  Do an Iterative Deepening Depth-First Search of the state space to find optimal routes
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```
