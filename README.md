# Log Grapher
Just a small utility that parses log information into rust structs, and then uses that to draw a graph:

```sh
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
```

![example graph](https://github.com/Toqozz/tf2-log-grapher/blob/master/example.png)
