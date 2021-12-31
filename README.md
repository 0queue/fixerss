# fixerss

A pair of programs to scrape a website and generate an rss feed.

## Building

SQLx is used for the persistence layer, so make sure to set `DATABASE_URL`
appropriately.  Recommended to add `.env` file in the project root with the
contents

```
DATABASE_URL=sqlite:./target/build.db
```

Then, when using the SQLx CLI, make sure to execute commands from the project
root while pointing towards the sub project's sources. Example:

```shell
cargo sqlx migrate --source server/migrations info
```

Actually building is then standard cargo fare:

```shell
cargo b --release
```

## Binaries

- `server`: The server that reads a config, serves generated rss files,
            and periodically scrapes the configured websites
- `oneshot`: Scrapes the given config and outputs the generated rss once.
             Useful for debugging css selectors
  
## Docker

To build, run this in the project root:
```shell
docker build -t fixerss:<version> .
```

And to run with the repo's example file with an in memory database,
from the project root:

```shell
docker run --rm -it -v "$(pwd)/fixerss.toml:/fixerss.toml:ro" -p 8000:8000 fixerss
```

## Config

- `FIXERSS_SETTINGS_FILE`: location of the [fixerss.toml](./fixerss.toml) file. 
                           Defaults to `fixerss.toml`
- `FIXERSS_HISTORY_FILE`: location of the SQLite database that holds generated
                          items.  Defaults to `:memory:`, which does not persist
                          anything to disk.  If anything other than `:memory:` is
                          used, the file will be created
- `FIXERSS_ADDRESS`: address to bind to, defaults to `0.0.0.0`
- `FIXERSS_PORT`: port to bind to, defaults to `8080`

See [`server_config.rs`](server/src/server_config.rs) for more up to date info.

For more info on the settings file, see the annotated [`fixerss.toml`](./fixerss.toml).