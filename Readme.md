RBDC-Oracle

* Oracle driver for rbatis


# Requirements
- C linker. For example:
```
sudo apt install build-essential
```

- Rbatis. See [Rbatis](https://github.com/rbatis/rbatis)

- Oracle client 11.2 or later. See [ODPI-C installation document](https://oracle.github.io/odpi/doc/installation.html)

### OracleConnectOptions

When using `rbdc-oracle`, construct `OracleConnectOptions` with `new()` /
`default()` plus builder methods, or `with_credentials(...)`.

Do not use a struct literal in downstream crates. The type is non-exhaustive so
new fields can be added without breaking every caller.

```rust
use rbdc_oracle::OracleConnectOptions;

let options = OracleConnectOptions::with_credentials(
    "user",
    "password",
    "//localhost:1521/XE",
);

let tuned = OracleConnectOptions::new()
    .username("user")
    .password("password")
    .connect_string("//localhost:1521/XE")
    .statement_cache_capacity(100)
    .row_channel_size(50)
    .command_channel_size(50);
```
