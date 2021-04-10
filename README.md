# fixerss

A pair of programs to scrape website and generate an rss feed.

## Building

SQLx is used for the persistence layer, so make sure to set `DATABASE_URL`
appropriately.  Recommended to add `.env` file in the project root with the
contents

```
DATABASE_URL=sqlite:./target/build.db
```

Then, when using the SQLx CLI, make sure to execute commands from the project
root while pointing towards the sub project's sources. Example:

```
$ cargo sqlx migrate --source server/migrations info
```

## Binaries

- `server`: The server that reads a config, serves generated rss files,
            and periodically scrapes the configured websites
- `oneshot`: Scrapes the given config and outputs the generated rss once.
             Useful for debugging css selectors.
  
## Docker

To build, run this in the project root:
```shell
docker build -t fixerss .
```

And to run with the repo's example file, from the project root:
```shell
docker run --rm -it -v "$(pwd)/fixerss.toml:/fixerss.toml:ro" fixerss
```