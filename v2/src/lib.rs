//! Offline execution truth: own create-custom-wordle.ts at 5c7c4792f534a45001e7b71b1b41ead825914f80.
//! No external site, JavaScript runtime, preferences, or storage are needed.
use serde_json::Value;

pub const COMMAND: &str = "create-custom-wordle";
pub const URL_PREFIX: &str = "https://mywordle.strivemath.com/?word=";
/// Explicit resource boundary, measured BEFORE trimming. Nothing is truncated.
pub const MAX_UTF16_UNITS: usize = 1_048_576;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Generated {
    pub word: String,
    pub url: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    MalformedArgument,
    Empty,
    Spaces,
    Letters,
    Resource,
}

impl Failure {
    pub fn message(self) -> &'static str {
        match self {
            Self::MalformedArgument => "The word argument must be a string",
            Self::Empty => "Please provide a word",
            Self::Spaces => "Word cannot contain spaces",
            Self::Letters => "Word can only contain letters",
            Self::Resource => "Word exceeds the resource limit of 1048576 UTF-16 code units",
        }
    }
}

// ECMAScript WhiteSpace + LineTerminator, deliberately excluding U+0085 / U+180E.
fn js_trim(unit: u16) -> bool {
    matches!(unit, 0x0009..=0x000d | 0x0020 | 0x00a0 | 0x1680 | 0x2000..=0x200a | 0x2028 | 0x2029 | 0x202f | 0x205f | 0x3000 | 0xfeff)
}

/// Native string API. UTF-16 API below also models lone JS surrogate code units.
pub fn generate(input: &str) -> Result<Generated, Failure> {
    if input.encode_utf16().count() > MAX_UTF16_UNITS {
        return Err(Failure::Resource);
    }
    generate_utf16(&input.encode_utf16().collect::<Vec<_>>())
}

/// Exact accepted domain of trim().toLowerCase() followed by /[^a-zA-Z]/.
/// Of Unicode non-ASCII scalars, only Kelvin sign lowercases to ASCII (k).
/// Other scalars, expansions, surrogate pairs and lone surrogates fail the regex.
pub fn generate_utf16(input: &[u16]) -> Result<Generated, Failure> {
    if input.len() > MAX_UTF16_UNITS {
        return Err(Failure::Resource);
    }
    let start = input
        .iter()
        .position(|&c| !js_trim(c))
        .unwrap_or(input.len());
    let end = input
        .iter()
        .rposition(|&c| !js_trim(c))
        .map_or(start, |i| i + 1);
    let input = &input[start..end];
    if input.is_empty() {
        return Err(Failure::Empty);
    }
    let mut word = String::with_capacity(input.len());
    for &c in input {
        let c = match c {
            0x0041..=0x005a => (c + 32) as u8,
            0x0061..=0x007a => c as u8,
            0x212a => b'k',
            _ => {
                return Err(if input.contains(&0x20) {
                    Failure::Spaces
                } else {
                    Failure::Letters
                })
            }
        };
        word.push(char::from(c));
    }
    let mut url = String::with_capacity(URL_PREFIX.len() + word.len());
    url.push_str(URL_PREFIX);
    for (i, c) in word.bytes().enumerate() {
        url.push(char::from((c - b'a' + b"wordle"[i % 6] - b'a') % 26 + b'a'));
    }
    Ok(Generated { word, url })
}

/// Beta's payload carries arguments in argumentValues, never formValues or preferences.
pub fn from_payload(payload: &Value) -> Result<Generated, Failure> {
    let word = payload
        .get("argumentValues")
        .and_then(Value::as_object)
        .and_then(|args| args.get("word"))
        .and_then(Value::as_str)
        .ok_or(Failure::MalformedArgument)?;
    generate(word)
}

/// Every mutating OS boundary must acknowledge success explicitly.
pub fn acknowledged(value: &Value) -> bool {
    value.as_object().is_some_and(|o| {
        o.get("ok") == Some(&Value::Bool(true))
            && o.get("error").is_none_or(Value::is_null)
            && o.get("success").is_none_or(|v| *v == Value::Bool(true))
    })
}
