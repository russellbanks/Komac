## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

### Creating a Pull Request

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feat/new-feature`)
3. Commit your Changes (`git commit -m 'Add some feature'`)
4. Push to the Branch (`git push origin feat/new-feature`)
5. Open a Pull Request

### Checks

CI runs these on every pull request, so run them locally before pushing:

```sh
cargo +nightly fmt --check                          # Formatting
cargo clippy --locked --all-targets -- -D warnings  # Lints
cargo test --locked                                 # Tests
```

Formatting needs the nightly toolchain (`rustup toolchain install nightly --component rustfmt`) because
`rustfmt.toml` uses unstable options to group and merge imports. Stable `rustfmt` ignores them and
formats imports differently. Run `cargo +nightly fmt` to fix formatting.

### Translating komac

komac's text is translated with [Fluent](https://projectfluent.org/fluent/guide/). Each language has a file at
`i18n/<language>/komac.ftl`, and `i18n/en-US/komac.ftl` is the source that every other language is translated from.

To add a language:

1. Copy `i18n/en-US/komac.ftl` to `i18n/<language>/komac.ftl`, where `<language>` is a language tag such as `de-DE`
   or `fr`.
2. Translate the text after each `=`. Keep the message IDs (before the `=`) and arguments such as `{ $identifier }`
   as they are, though arguments can move anywhere in the sentence.
3. Remove any messages you haven't translated. komac shows those in English.
4. Try it with `KOMAC_LANG=<language> cargo run -- --help`.

Many messages choose their wording with a [selector](https://projectfluent.org/fluent/guide/selectors.html), such
as `{ $count -> [one] ... *[other] ... }`. Use the plural categories your language needs, which may be more or fewer
than English has.

`cargo test` checks that every translation file parses and only has messages that are in `en-US`.

#### Adding text in code

Text that users read in the terminal goes in `i18n/en-US/komac.ftl` and is looked up with `fl!`, which checks at
compile time that the message and its arguments exist:

```rust
println!("{}", fl!("latest-version", identifier = identifier.as_str(), version = version.to_string()));
```

- Write whole sentences, with the values that change passed as arguments. Don't build sentences out of separately
  translated pieces, as word order differs between languages.
- Use a selector for anything that depends on a number or a state, rather than choosing between messages in code.
- Name arguments in snake_case, as they're also Rust identifiers.
- Keep logs, errors from internal failures and anything sent to GitHub, such as pull request bodies, in English.

### Testing your changes

Using Docker is the easiest way to test your code before submitting a pull request.

> [!NOTE]
> When using the Docker container on Windows, the WSL engine does not support the default collection for keys or tokens. This means that when testing inside the container GitHub tokens will not be stored, even when `komac token update` is used.
> 
> This is a [known issue](https://github.com/hwchen/keyring-rs/blob/47c8daf3e6178a2282ae3e8670d1ea7fa736b8cb/src/secret_service.rs#L73-L77) which is documented in the keyring crate.
>
> As a workaround, you can set the `GITHUB_TOKEN` environment variable from within the container, in the `docker run` command, or in the Dockerfile itself

1. Ensure you have docker installed and the docker engine is running.
2. Run `docker build ./ --tag komac_dev:latest`.
3. Wait for the build to complete.
4. Start the container using `docker run -it komac_dev bash`.
5. Test out any commands. Use the `exit` command to quit the container