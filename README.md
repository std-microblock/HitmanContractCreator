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

publish:

Publish contracts

USAGE:
    hitman_contract_creator.exe publish [OPTIONS] [USERID]

ARGS:
    <USERID>    [default: fe76faee-ecdc-4dd7-a6d5-c5b84054a87c]

OPTIONS:
    -b, --bearer <BEARER>
    -f, --file <FILE>        The file to submit
    -h, --help               Print help information
        --hitman2
        --hitman3
```

Basic usage:
```
hitman_contract_creator.exe publish --bearer [your oauth token] -f [path to your contract json file] [your player id] [--hitman3/--hitman2]
```

example:
```
hitman_contract_creator.exe publish -b <oauthtoken>  -f "./final rest.json" -f "./new zealand.json" --hitman2 --hitman3
```
```
hitman_contract_creator.exe publish --bearer <oauthtoken> -f testpost.json fe76faee-ecdc-4dd7-a6d5-c5b84054a87c --hitman3
```

