// SPDX-License-Identifier: Apache-2.0
//! YAML 1.2 core schema resolution, offered rather than applied.
//!
//! Q2 rules that scalar types come from the meta-schema and never from the YAML
//! resolver. The loader therefore never calls anything in this module. It exists
//! so that the layer which *does* know a declared type has one implementation of
//! "what does the core schema make of this text", instead of one per call site.
//!
//! The rules are [YAML 1.2.2 §10.2.1](https://yaml.org/spec/1.2.2/#1021-tags),
//! written out rather than pulled from a regex crate. Two properties follow from
//! the core schema that YAML 1.1 does not have, and both are why Q2 named it:
//!
//! - `no`, `yes`, `on` and `off` are strings. YAML 1.1 reads all four as
//!   booleans, which silently rewrites a facet value called `no`.
//! - A leading zero is not an octal escape. `013` is the integer 13.
//!
//! Every function here refuses a non-plain scalar. A quoted `true` is the string
//! `true`, and that is the only way an author has to say so.

use crate::value::Scalar;

/// Whether the core schema resolves this scalar to null.
///
/// The empty case is what `key:` with nothing after it produces, and it is the
/// reason style is kept: `key: ""` is an author who wrote an empty string, and
/// a required-value check has to tell the two apart.
pub fn as_null(scalar: &Scalar) -> bool {
    scalar.is_plain() && matches!(scalar.text.as_str(), "" | "~" | "null" | "Null" | "NULL")
}

pub fn as_bool(scalar: &Scalar) -> Option<bool> {
    if !scalar.is_plain() {
        return None;
    }
    match scalar.text.as_str() {
        "true" | "True" | "TRUE" => Some(true),
        "false" | "False" | "FALSE" => Some(false),
        _ => None,
    }
}

/// `[-+]? [0-9]+` in base ten, `0o[0-7]+`, or `0x[0-9a-fA-F]+`.
pub fn as_int(scalar: &Scalar) -> Option<i64> {
    if !scalar.is_plain() {
        return None;
    }
    let text = scalar.text.as_str();
    if let Some(digits) = text.strip_prefix("0o") {
        return parse_radix(digits, 8);
    }
    if let Some(digits) = text.strip_prefix("0x") {
        return parse_radix(digits, 16);
    }
    let (negative, digits) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let magnitude: i64 = digits.parse().ok()?;
    Some(if negative { -magnitude } else { magnitude })
}

fn parse_radix(digits: &str, radix: u32) -> Option<i64> {
    // The sign forms are base ten only, so `0o-7` is a string and `i64::from_str_radix`
    // would otherwise accept it.
    if digits.is_empty() || !digits.chars().all(|c| c.is_digit(radix)) {
        return None;
    }
    i64::from_str_radix(digits, radix).ok()
}

/// The core-schema float forms, including `.inf` and `.nan`.
pub fn as_float(scalar: &Scalar) -> Option<f64> {
    if !scalar.is_plain() {
        return None;
    }
    let text = scalar.text.as_str();
    let (sign, rest) = match text.strip_prefix('-') {
        Some(rest) => (-1.0, rest),
        None => (1.0, text.strip_prefix('+').unwrap_or(text)),
    };
    if matches!(rest, ".inf" | ".Inf" | ".INF") {
        return Some(sign * f64::INFINITY);
    }
    // `.nan` takes no sign, per the production.
    if matches!(text, ".nan" | ".NaN" | ".NAN") {
        return Some(f64::NAN);
    }
    if !is_core_float(rest) {
        return None;
    }
    text.parse().ok()
}

/// `( \. [0-9]+ | [0-9]+ ( \. [0-9]* )? ) ( [eE] [-+]? [0-9]+ )?`, unsigned.
fn is_core_float(text: &str) -> bool {
    let (mantissa, exponent) = match text.find(['e', 'E']) {
        Some(at) => (&text[..at], Some(&text[at + 1..])),
        None => (text, None),
    };
    if let Some(exponent) = exponent {
        let digits = exponent.strip_prefix(['-', '+']).unwrap_or(exponent);
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return false;
        }
    }
    let mantissa_ok = match mantissa.split_once('.') {
        // `.5`, and `1.` and `1.5`.
        Some((whole, fraction)) => {
            let digits_ok = |s: &str| s.bytes().all(|b| b.is_ascii_digit());
            if whole.is_empty() {
                !fraction.is_empty() && digits_ok(fraction)
            } else {
                digits_ok(whole) && digits_ok(fraction)
            }
        }
        // An exponent form with no point: `1e3`.
        None => !mantissa.is_empty() && mantissa.bytes().all(|b| b.is_ascii_digit()),
    };
    // A bare integer is an integer rather than a float, unless an exponent made
    // it one.
    mantissa_ok && (exponent.is_some() || mantissa.contains('.'))
}

/// The text, whatever it looks like. Every scalar resolves to a string, which is
/// the core schema's fallback and the only resolution that never fails.
pub fn as_str(scalar: &Scalar) -> &str {
    &scalar.text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Style;

    fn plain(text: &str) -> Scalar {
        Scalar {
            text: text.into(),
            style: Style::Plain,
        }
    }

    fn quoted(text: &str) -> Scalar {
        Scalar {
            text: text.into(),
            style: Style::DoubleQuoted,
        }
    }

    #[test]
    fn no_stays_the_string_no() {
        // The Q2 ruling, executable. YAML 1.1 reads all four as booleans.
        for text in ["no", "No", "NO", "yes", "on", "off", "y", "n"] {
            assert_eq!(as_bool(&plain(text)), None, "{text} resolved to a bool");
            assert_eq!(as_str(&plain(text)), text);
        }
    }

    #[test]
    fn core_schema_bools_are_the_six_spellings() {
        for text in ["true", "True", "TRUE"] {
            assert_eq!(as_bool(&plain(text)), Some(true));
        }
        for text in ["false", "False", "FALSE"] {
            assert_eq!(as_bool(&plain(text)), Some(false));
        }
    }

    #[test]
    fn a_quoted_scalar_carries_no_type() {
        assert_eq!(as_bool(&quoted("true")), None);
        assert_eq!(as_int(&quoted("42")), None);
        assert_eq!(as_float(&quoted("1.5")), None);
        assert!(!as_null(&quoted("")));
        assert!(!as_null(&quoted("~")));
    }

    #[test]
    fn null_is_plain_and_empty_or_one_of_four_spellings() {
        for text in ["", "~", "null", "Null", "NULL"] {
            assert!(as_null(&plain(text)), "{text} did not resolve to null");
        }
        for text in ["nul", "none", "NuLl"] {
            assert!(!as_null(&plain(text)), "{text} resolved to null");
        }
    }

    #[test]
    fn integers_have_no_octal_leading_zero() {
        assert_eq!(as_int(&plain("013")), Some(13));
        assert_eq!(as_int(&plain("0o13")), Some(11));
        assert_eq!(as_int(&plain("0x1f")), Some(31));
        assert_eq!(as_int(&plain("-7")), Some(-7));
        assert_eq!(as_int(&plain("+7")), Some(7));
        // Forms the core schema does not have.
        assert_eq!(as_int(&plain("1_000")), None);
        assert_eq!(as_int(&plain("0b101")), None);
        assert_eq!(as_int(&plain("0o-7")), None);
        assert_eq!(as_int(&plain("")), None);
        assert_eq!(as_int(&plain("1.0")), None);
    }

    #[test]
    fn floats_take_the_core_forms_and_no_others() {
        assert_eq!(as_float(&plain("1.5")), Some(1.5));
        assert_eq!(as_float(&plain(".5")), Some(0.5));
        assert_eq!(as_float(&plain("1.")), Some(1.0));
        assert_eq!(as_float(&plain("-1.5e3")), Some(-1500.0));
        assert_eq!(as_float(&plain("1e3")), Some(1000.0));
        assert_eq!(as_float(&plain("-.inf")), Some(f64::NEG_INFINITY));
        assert!(as_float(&plain(".nan")).is_some_and(f64::is_nan));
        // A bare integer is an integer. Whether a schema that asked for a float
        // accepts one is the meta-schema's ruling, not this module's.
        assert_eq!(as_float(&plain("3")), None);
        assert_eq!(as_float(&plain("1.2.3")), None);
        assert_eq!(as_float(&plain("1e")), None);
        assert_eq!(as_float(&plain(".")), None);
    }
}
