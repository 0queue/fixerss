# fixerss

A pair of programs to scrape website and generate an rss feed.

## Binaries

- `fixerss`: The server that reads a config, serves generated rss files,
             and periodically scrapes the configured websites
- `config-runner`: Scrapes the given config and outputs the generated rss.
                   Useful for debugging css selectors.