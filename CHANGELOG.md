# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased] - ReleaseDate

### Added

- Better documentation

### Changed 

- Put the `async_tokio_handle` macro behind the `tokio` feature flag.

### Security

- Upgraded tokio dependency version to 1.23.1. Due to dependabot security
  recommendation.

## [0.1.0] - 2023-01-14

### Added

- Initial repository release.
- `async_tokio_handle` macro to generate client handle for tokio.

<!-- next-url -->
[Unreleased]: https://github.com/stedmeister/client-handle/compare/v0.1.0...HEAD