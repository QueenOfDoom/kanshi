# Kanshi Bot

<!--suppress CheckImageSize -->
<div style="text-align: center;">
    <img src="./img/FukuKanshi-Logo.png" width="128" style="border-radius: 50%; margin-inline: 10px;" alt="Bot Icon"/>
    <img src="./img/Kanshi-Logo.png" width="128" style="border-radius: 50%; margin-inline: 10px;" alt="Bot Icon"/>
</div>
<div style="text-align: center; margin-top: 15px;">
    <i>The artwork is <del>self-made</del> created by AI</i>
</div>

## Functionality

This bot logs message edits and deletions to `env:LOG_CHANNEL`.

For those who are interested, the bots name originates from:
監視 (monitoring, watching, surveillance).

## Development

### Persistence

The persistence is now managed by [Diesel](https://diesel.rs), which is a Rust
ORM.
The schema file can be found at [`src/persistence/schema.rs`](./src/persistence/schema.rs).

In order to update the migrations *from modifications in the schema*, you'd run:
```bash
diesel migration generate --diff-schema [migration-name]
```

For now, in order to not make DevOps over-bearing, so that people who do not
enjoy figuring out DevOps configurations for their Dev-Environments and
the integrations of services, still can contribute to this project - I use
SQLite.
This may change in the future as the need in a server database becomes apparent.