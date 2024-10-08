# Changelog

All notable changes to the 'Kanshi' project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
as well as [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).

## [Unreleased]

### Added

- Changelog Command with SemVer query support
- Optimize Database queries by using a Connection Pool via `r2d2`
- Registry of edited messages from when the bot wasn't online yet

### Changed

- Migrated from `rusqlite` to `diesel` for the sake of having a proper ORM

### Fixed

- Migrated from `dotenv` to `dotenvy` ([RUSTSEC-2021-0141](https://rustsec.org/advisories/RUSTSEC-2021-0141.html))
- Patched [0.1.0] tag date.

## [0.1.0] - 2024-09-19

### Added

- Discord message edit and deletion logging via Discord Bot and Embeds
- Preservation of deleted messages within a SQLite Database
- Console & File logging of Discord Events

[Unreleased]: https://github.com/QueenOfDoom/kanshi/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/QueenOfDoom/kanshi/releases/tag/v0.1.0