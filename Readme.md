RBDC

* an database driver abstract

* support zero copy serde-ser/de

Database -> bytes ->rbs::Value-> Struct(User Define)
Struct(User Define) -> rbs::ValueRef -> ref clone() -> Database


### how to define my driver?
should impl trait and load driver
* impl trait rbdc::db::{Driver, MetaData, Row, Connection, ConnectOptions, Placeholder};

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
