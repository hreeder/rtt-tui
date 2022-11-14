# RTT TUI
A small TUI around RealTimeTrains (https://www.realtimetrains.co.uk/).

Mainly used as a way to learn Rust, with the added benefit of having an auto-refreshing, terminal application to show progress of a chosen train.

## Setup
1. Make an account with RealTimeTrains' API - https://api.rtt.io
2. Set your credentials in `~/.config/rtt.yaml` as such:
   ```yaml
   username: YOUR_USERNAME_FROM_RTT
   password: YOUR_PASSWORD_FROM_RTT
   ```
3. Run the app:
   ```
   ./rtttui 1200 EDB KGX
   ╭─────────────────────Train Info─────────────────────╮
   │ LNER                                               │
   │ 1200 Edinburgh to London Kings Cross               │
   │                                                    │
   ╰────────────────────────────────────────────────────╯
   ╭────────────────────Train Status────────────────────╮
   │ Edinburgh [5] (-1)                            1159 │
   │ Alnmouth [1] (-1/0)                      1257 1300 │
   │ Newcastle [3] (-1/0)                     1325 1329 │
   │ Durham [1] (0/0)                         1341 1343 │
   │ Darlington [1] (0/0)                     1400 1401 │
   │ York [3] (0/0)                           1428 1432 │
   │ Doncaster [1] (0/0)                      1453 1455 │
   │ Newark Northgate [2] (0/0)               1517 1519 │
   │ Peterborough [3] (2/2)                   1550 1552 │
   │ London Kings Cross [4] (-1)              1638      │
   ╰────────────────────────────────────────────────────╯
   ╭───────────────Additional Information───────────────╮
   │ Data Sourced From Realtime Trains                  │
   │ Controls: [q]uit                                   │
   │ Last Update: 20s                                   │
   ╰────────────────────────────────────────────────────╯
   ```

## Usage
```
❯ ./rtttui --help
Usage: rtttui <departs> <source> <dest> [--tick-rate <tick-rate>] [--refresh-rate <refresh-rate>]

Track a train

Positional Arguments:
  departs           departure time, ie 0830
  source            three letter source station (crs), ie EDB, or the TIPLOC
                    code source station
  dest              three letter destination station (crs), ie KGX, or the
                    TIPLOC code destination station

Options:
  --tick-rate       time in ms between two ticks (default 250)
  --refresh-rate    time in seconds between remote API updates (default 30)
  --help            display usage information
```

## Known Issues
### App cannot handle stations with multiple TIPLOC codes:
If a station, such as Glasgow Queen Street, has multiple TIPLOC codes (ie `GLGQHL` & `GLGQLL`), the application cannot currently decode this, and will crash. I'm still learning how to handle this, and the beginning has already started taking shape in `src/rtt.rs` with the introduction of the `MultiTiploc` enum.

This only affects trains bound TO Glasgow Queen Street, as the only place where we need to be aware of the TIPLOC is in finding the destination.

### Credential Storage is Plaintext
A future improvement would be to use the system's keychain to store the RTT credentials, and provide a `login` command to set these.

This would also remove the dependency on the hardcoded configuration path.
