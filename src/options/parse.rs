use crate::OracleConnectOptions;
use rbdc::Error;
use std::str::FromStr;
use url::Url;

impl FromStr for OracleConnectOptions {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim();

        if value.starts_with('{') {
            return serde_json::from_str(value).map_err(|e| Error::from(e.to_string()));
        }

        Self::from_uri(value)
    }
}

impl OracleConnectOptions {
    pub(crate) fn from_uri(uri: &str) -> Result<Self, Error> {
        let parsed = Url::parse(uri).map_err(|e| Error::from(format!("Invalid URL: {e}")))?;

        if parsed.scheme() != "oracle" {
            return Err(Error::from("URL scheme must be 'oracle'"));
        }

        let username = parsed.username().to_owned();
        let password = parsed
            .password()
            .ok_or_else(|| Error::from("Password is required"))?
            .to_owned();
        let host = parsed
            .host_str()
            .ok_or_else(|| Error::from("Host is required"))?;
        let port = parsed.port().unwrap_or(1521);
        let service = parsed.path().trim_start_matches('/');
        let connect_string = if service.is_empty() {
            format!("//{host}:{port}")
        } else {
            format!("//{host}:{port}/{service}")
        };

        let mut options = Self::new();
        options.username = username;
        options.password = password;
        options.connect_string = connect_string;
        Ok(options)
    }
}

#[cfg(test)]
mod tests {
    use crate::OracleConnectOptions;

    #[test]
    fn test_parse_uri() {
        let options: OracleConnectOptions = "oracle://scott:tiger@localhost:1521/XE"
            .parse()
            .expect("oracle uri");
        assert_eq!(options.username, "scott");
        assert_eq!(options.password, "tiger");
        assert_eq!(options.connect_string, "//localhost:1521/XE");
    }

    #[test]
    fn test_parse_json() {
        let options: OracleConnectOptions =
            r#"{"username":"a","password":"b","connect_string":"//localhost/XE"}"#
                .parse()
                .expect("oracle json");
        assert_eq!(options.username, "a");
        assert_eq!(options.password, "b");
        assert_eq!(options.connect_string, "//localhost/XE");
    }
}
