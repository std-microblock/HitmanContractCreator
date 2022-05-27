# HitmanContractCreator
Create whatever contract you want in both HITMAN2 and HITMAN3. Written in rust.

## Usage
```
USAGE:
    hitman_contract_creator.exe <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    help       Print this message or the help of the given subcommand(s)
    publish    Publish contracts
```

Basic usage:
```
hitman_contract_creator.exe publish --bearer [your oauth token] [path to your contract json file] [your player id, or a random uuid] [--hitman3/--hitman2]
```
example:
```
hitman_contract_creator.exe publish --bearer eyxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx.uxxxxxxxxxxxxxxx_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx-xxxxxxxxxx_xxxxxxx_xxxu-xxxxxxxxxxxxxxxx-xxxxxxxxxxxxxxxxxxxxxxxxxxxxx testpost.json fe76faee-ecdc-4dd7-a6d5-c5b84054a87c --hitman3
```

