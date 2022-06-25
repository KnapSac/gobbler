# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]
### Changed
- Switch to using `clap` derive macro by @KnapSac in https://github.com/KnapSac/gobbler/pull/14

## [0.4.0] - 2022-06-17
### Added
- Support storing subscriptions file in environment variable by @KnapSac in https://github.com/KnapSac/gobbler/pull/13

## [0.3.0] - 2022-03-24
### Added
- Support filtering feeds by name by @KnapSac in https://github.com/KnapSac/gobbler/pull/7
- Find more links by @KnapSac in https://github.com/KnapSac/gobbler/pull/8
- Add support for exporting and importing subscriptions by @KnapSac in https://github.com/KnapSac/gobbler/pull/10

## [0.2.1] - 2022-01-05
### Added
- `last-ran-at` option for debugging purposes

### Fixed
- `ran_in_past_n_days` was off by one day due to using `>=` instead of `>`

## [0.2.0] - 2022-01-04
### Added
- Store subscriptions file in fixed location (https://github.com/KnapSac/gobbler/pull/2)
- Allow overwriting of subscriptions file (https://github.com/KnapSac/gobbler/pull/3)

## [0.1.0] - 2022-01-04
### Added
- Support for adding and removing RSS feed subscriptions.
- Support for listing the items in a RSS feed in the last specified number of weeks.
- Support using `gobbler` in a shell profile.

[Unreleased]: https://github.com/KnapSac/gobbler/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/KnapSac/gobbler/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/KnapSac/gobbler/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/KnapSac/gobbler/compare/v0.0.2...v0.2.1
[0.2.0]: https://github.com/KnapSac/gobbler/compare/v0.0.1...v0.0.2
[0.1.0]: https://github.com/KnapSac/gobbler/releases/tag/v0.0.1
