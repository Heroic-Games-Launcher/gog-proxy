# gog-proxy

Proxy arround some GOG endpoints for extending API capabilities for better Linux support.

## The target

This project aims to close the gap between features offered for each platform. Starting with cloud saves.

We hope GOG will eventually provide that data themselves, untill then we are on our own.

## API

Currently supported enpoint returns game config from `remote-config.gog.com` endpoint. If there is Linux data in games_data directory that will be extended

```
GET /api/config/client_id
```

## Contributing

The `games_data/` directory contains automatically generated `ClientId.json` files for each game. That are disabled by default. Contributions to that directory are welcome. All json files are formatted using Prettier.

### Obtaining the relevant game data

Let's say, you want to add a new linux native game to suport cloud saves.

1. Go to https://gogdb.org
2. Find the game package you want to add
3. Pick a game build (eg. https://www.gogdb.org/product/1308320804/build/54853937123913458)
4. Copy client id
5. Make a [request to the API](#api) with it
6. Find corresponding paths on Linux
7. Write/Update config file

### Sample config

```json
{
  "overlay": { "supported": false },
  "cloudStorage": {
    "enabled": true,
    "locations": [
      {
        "name": "saves",
        "location": "<?XDG_CONFIG_HOME?>/unity3d/Team Cherry/Hollow Knight",
        "wildcard": "*.dat"
      }
    ]
  }
}
```

- overlay - not used currently, enabled should be set to false for now

- cloudStorage - enabled field indicates whether cloud saves are supported; locations is a array of objects containing name, location and wildcard

- name - location name - this should be set to same value as on Mac or Windows

- location - place where files are stored - this can use env vars like `$HOME`, values that have fallbacks (like `$XDG_CONFIG_HOME` falls back to `$HOME/.config`) should be closed in `<?SOME_VAR?>`

- wildcard - value can be set to `null`. Due to nature of some games on Linux that's required to not push unrelated junk to the cloud (that's usually a case for Unity games on Linux)

**IMPORTANT NOTE**  
Some games can have enabled `cloudStorage` but empty locations array. This is caused by the fact that such games use `__default` location which is only available on Windows and Mac, since it's handled by Galaxy SDK (which doesn't exist on Linux btw). [SDK Documentation related to this](https://docs.gog.com/sdk-storage/#cloud-saves)

In that case config should still be defined if it's compatible, however the name should be set to `__default`

### Common location vars

Used on Windows and Mac

```
INSTALL
SAVED_GAMES
APPLICATION_DATA_LOCAL
APPLICATION_DATA_LOCAL_LOW
APPLICATION_DATA_ROAMING
APPLICATION_SUPPORT
DOCUMENTS
```

## Running the server

```
cargo run --bin gog_proxy
```

## Generate json files

```
cargo run --bin generator
```
