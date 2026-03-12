use std::collections::HashMap;

use crate::types::FS3Error;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeclaredTrailerNames {
    pub names: Vec<String>,
}

impl DeclaredTrailerNames {
    pub fn contains(&self, name: &str) -> bool {
        self.names.iter().any(|v| v.eq_ignore_ascii_case(name))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedTrailerHeaders {
    pub headers: HashMap<String, String>,
}

pub fn parse_declared_trailer_names(value: Option<&str>) -> Result<Option<DeclaredTrailerNames>, FS3Error> {
    let Some(value) = value else {
        return Ok(None);
    };
    let names = value
        .split(',')
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| v.to_ascii_lowercase())
        .collect::<Vec<_>>();
    if names.is_empty() {
        return Err(FS3Error::bad_request("Invalid x-amz-trailer"));
    }
    Ok(Some(DeclaredTrailerNames { names }))
}

pub fn parse_trailer_block(
    block: &[u8],
    declared: Option<&DeclaredTrailerNames>,
) -> Result<ParsedTrailerHeaders, FS3Error> {
    let text = std::str::from_utf8(block).map_err(|_| FS3Error::bad_request("Invalid trailer block"))?;
    let mut headers = HashMap::new();
    for line in text.split("\r\n").filter(|line| !line.is_empty()) {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let name = name.trim().to_ascii_lowercase();
        let value = value.trim().to_string();
        if !name.is_empty() {
            headers.insert(name, value);
        }
    }

    if let Some(declared) = declared {
        for name in &declared.names {
            if !headers.contains_key(name) {
                return Err(FS3Error::bad_request(format!(
                    "Missing declared trailer header: {name}"
                )));
            }
        }
    }

    Ok(ParsedTrailerHeaders { headers })
}
