# Log Grapher
Just a small utility that parses log information into rust structs, and then uses that to draw a graph:

```
$ cargo build --release

$ ./target/release/log-grapher --help
Usage: log_grapher --log_id LOGID --steamid STEAMID3

Options:
    -h, --help          print this help menu
        --log-id LOGID  download and process a log given an id
        --log-file FILE process a log file from disk
        --steamid STEAMID3
                        the SteamID3 of the player to graph for
        --steamids STEAMID3_1, STEAMID3_2, ..
                        a comma separated list of ids to search for in the log
                        and generate a graph
        --alias ALIAS   the alias of the player to graph for
        --batching SECONDS
                        the batching period of events

$ ./target/release/log-grapher --log-id 3013926 --alias 'FROYO b4nny' --batching 10
Processed log in 91.60ms
Making timeline for player: FROYO b4nny, batching: 10

$ cat ./out.txt
Highlights:
0: tick=2333, 1: tick=8333, 2: tick=9400, 3: tick=14600, 4: tick=23733, 5: tick=25133, 6: tick=28467, 7: tick=29400, 8: tick=32600, 9: tick=41933, 10: tick=42867, 11: tick=45067, 1
2: tick=45600, 13: tick=46133, 14: tick=48800, 15: tick=53467, 16: tick=54867, 17: tick=55600, 18: tick=56733
```

![example graph](https://github.com/Toqozz/tf2-log-grapher/blob/master/example.png)
