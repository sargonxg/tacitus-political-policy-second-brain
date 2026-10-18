//! Opt-in JSON syntax preflight. Original bytes are never rewritten.
use serde::de::{self, DeserializeOwned, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use std::{cell::Cell, collections::BTreeSet, fmt};

pub const MAX_DEPTH: usize = 64;
pub const MAX_NODES: usize = 200_000;
pub const MAX_BYTES: usize = 2 * 1024 * 1024;

struct Scan<'a> {
    depth: usize,
    nodes: &'a Cell<usize>,
}
impl<'de> DeserializeSeed<'de> for Scan<'_> {
    type Value = ();
    fn deserialize<D: de::Deserializer<'de>>(self, d: D) -> Result<(), D::Error> {
        if self.depth > MAX_DEPTH {
            return Err(de::Error::custom("JSON depth limit exceeded"));
        }
        let n = self.nodes.get();
        if n >= MAX_NODES {
            return Err(de::Error::custom("JSON node limit exceeded"));
        }
        self.nodes.set(n + 1);
        d.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Scan<'_> {
    type Value = ();
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("bounded JSON with unique object keys")
    }
    fn visit_bool<E: de::Error>(self, _: bool) -> Result<(), E> {
        Ok(())
    }
    fn visit_i64<E: de::Error>(self, _: i64) -> Result<(), E> {
        Ok(())
    }
    fn visit_u64<E: de::Error>(self, _: u64) -> Result<(), E> {
        Ok(())
    }
    fn visit_f64<E: de::Error>(self, v: f64) -> Result<(), E> {
        if v.is_finite() {
            Ok(())
        } else {
            Err(E::custom("non-finite JSON number"))
        }
    }
    fn visit_str<E: de::Error>(self, _: &str) -> Result<(), E> {
        Ok(())
    }
    fn visit_unit<E: de::Error>(self) -> Result<(), E> {
        Ok(())
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<(), A::Error> {
        while a
            .next_element_seed(Scan {
                depth: self.depth + 1,
                nodes: self.nodes,
            })?
            .is_some()
        {}
        Ok(())
    }
    fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<(), A::Error> {
        let mut keys = BTreeSet::new();
        while let Some(k) = a.next_key::<String>()? {
            if !keys.insert(k) {
                return Err(de::Error::custom("duplicate JSON object key"));
            }
            a.next_value_seed(Scan {
                depth: self.depth + 1,
                nodes: self.nodes,
            })?;
        }
        Ok(())
    }
}
/// Checks syntax, decoded key uniqueness, finite numbers, UTF-8, depth and node bounds.
/// Does not normalize Unicode, canonicalize numbers, or establish semantic validity.
pub fn validate(raw: &[u8]) -> Result<(), String> {
    if raw.len() > MAX_BYTES {
        return Err("JSON byte limit exceeded".into());
    }
    std::str::from_utf8(raw).map_err(|_| "invalid JSON UTF-8".to_string())?;
    let mut d = serde_json::Deserializer::from_slice(raw);
    Scan {
        depth: 0,
        nodes: &Cell::new(0),
    }
    .deserialize(&mut d)
    .map_err(|e| e.to_string())?;
    d.end().map_err(|e| e.to_string())
}
/// Validate then deserialize the same immutable input, never a normalized intermediate.
pub fn from_slice<T: DeserializeOwned>(raw: &[u8]) -> Result<T, String> {
    validate(raw)?;
    serde_json::from_slice(raw).map_err(|e| e.to_string())
}
