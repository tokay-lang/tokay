# Contribution guidelines

Thanks that you want to contribute to Tokay!

## Issues

In case you encounter a bug, or you miss a feature, please [file an issue](https://github.com/tokay-lang/tokay/issues/new).

## Pull Requests

If you created a solution for a problem or added a feature, please make a [pull request](https://github.com/tokay-lang/tokay/compare).<br>
This can also be made as a draft, in case you want to discuss a change or feature, aren't finished or aren't sure if your change is ready to merge. Any pull request is welcome!

## Versioning

Tokay uses the semantic versioning scheme.<br>
Any major/minor/bugfix release is being published to [crates.io](https://crates.io).

As Tokay is unfinished, and under development, it currently has major version 0.<br>
The minor-version define huger milestones. The bugfix-version is not limited to bugfixes, but might also contain smaller milestones within a minoor-version, to allow for continous releasing of new features.

# Creating a new release

The `tokay-macros` crate is located in the folder `macros/`. Preliminarily to a Tokay release, it is necessary to release `tokay-macros` in a separate version to `crates.io`. Otherwise, during development, `tokay-macros` can be a local dependency in `Cargo.toml` - it depends on the use-case.

Next are some instructions on how to create a new release.

- Release `tokay-macros` (only required when version changed)
  - `cd macros`
  - In `Cargo.toml`, check and/or bump version number
  - `cargo publish`
- Release `tokay`
  - Make sure `CHANGELOG.md` is updated
  - Check for updated copyright year in `LICENSE`, `README.md`, `src/lib.rs` and `src/main.rs`
  - `Cargo.toml`
    - Check and/or bump version number
    - Uncomment line to use `tokay-macros` version from crates.io (obviously `tokay-macros = "x.x"`)
    - Comment line to use local `tokay-macros` (should be `# tokay-macros = { version = "x.x", path = "macros" }`)
  - `git commit` this state
  - `cargo publish`

## Maintenance

Maintainer of this project is [@phorward](https://github.com/phorward).
