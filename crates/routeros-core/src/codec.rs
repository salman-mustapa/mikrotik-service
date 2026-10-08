//! Wire format of the RouterOS API.
//!
//! A *sentence* is a list of *words* terminated by an empty word. Each word is
//! prefixed with a variable-length length field (1-5 bytes).

use std::collections::HashMap;

use bytes::{Buf, BytesMut};
use serde::Serialize;
use tokio_util::codec::{Decoder, Encoder};

use crate::error::Error;

/// Largest sentence we are willing to buffer before declaring the stream corrupt.
const MAX_SENTENCE_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Re,
    Done,
    Trap,
    Fatal,
    Empty,
}

/// A decoded reply sentence from the router.
#[derive(Debug, Clone, Serialize)]
pub struct Sentence {
    pub kind: Kind,
    pub tag: Option<u32>,
    pub attrs: HashMap<String, String>,
}

impl Sentence {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.attrs.get(key).map(String::as_str)
    }
}

/// Append the length prefix for a word of `len` bytes.
pub fn encode_len(len: usize, out: &mut Vec<u8>) {
    let l = len as u32;
    if l < 0x80 {
        out.push(l as u8);
    } else if l < 0x4000 {
        out.extend_from_slice(&((l | 0x8000) as u16).to_be_bytes());
    } else if l < 0x20_0000 {
        out.extend_from_slice(&(l | 0xC0_0000).to_be_bytes()[1..]);
    } else if l < 0x1000_0000 {
        out.extend_from_slice(&(l | 0xE000_0000).to_be_bytes());
    } else {
        out.push(0xF0);
        out.extend_from_slice(&l.to_be_bytes());
    }
}

/// Decode a length prefix. Returns `(length, bytes_consumed)`, or `None` if more data is needed.
fn decode_len(buf: &[u8]) -> Result<Option<(usize, usize)>, Error> {
    let Some(&b) = buf.first() else {
        return Ok(None);
    };
    let (need, high) = match b {
        0x00..=0x7F => return Ok(Some((b as usize, 1))),
        0x80..=0xBF => (2, (b & 0x3F) as usize),
        0xC0..=0xDF => (3, (b & 0x1F) as usize),
        0xE0..=0xEF => (4, (b & 0x0F) as usize),
        0xF0 => {
            if buf.len() < 5 {
                return Ok(None);
            }
            let l = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]) as usize;
            return Ok(Some((l, 5)));
        }
        _ => return Err(Error::Protocol(format!("reserved control byte 0x{b:02X}"))),
    };
    if buf.len() < need {
        return Ok(None);
    }
    let len = buf[1..need].iter().fold(high, |acc, &x| (acc << 8) | x as usize);
    Ok(Some((len, need)))
}

/// Try to parse one complete sentence from `buf` without consuming it.
fn parse_sentence(buf: &[u8]) -> Result<Option<(Vec<String>, usize)>, Error> {
    let mut pos = 0;
    let mut words = Vec::new();
    loop {
        let Some((len, n)) = decode_len(&buf[pos..])? else {
            return Ok(None);
        };
        pos += n;
        if len == 0 {
            return Ok(Some((words, pos)));
        }
        if buf.len() < pos + len {
            return Ok(None);
        }
        words.push(String::from_utf8_lossy(&buf[pos..pos + len]).into_owned());
        pos += len;
    }
}

fn into_sentence(words: Vec<String>) -> Result<Sentence, Error> {
    let mut it = words.into_iter();
    let kind = match it.next().as_deref() {
        Some("!re") => Kind::Re,
        Some("!done") => Kind::Done,
        Some("!trap") => Kind::Trap,
        Some("!fatal") => Kind::Fatal,
        Some("!empty") => Kind::Empty,
        other => return Err(Error::Protocol(format!("unexpected reply word: {other:?}"))),
    };
    let mut tag = None;
    let mut attrs = HashMap::new();
    for w in it {
        if let Some(rest) = w.strip_prefix(".tag=") {
            tag = rest.parse().ok();
        } else if let Some(rest) = w.strip_prefix('=') {
            // `=key=value`; the value itself may contain '='.
            match rest.split_once('=') {
                Some((k, v)) => attrs.insert(k.to_owned(), v.to_owned()),
                None => attrs.insert(rest.to_owned(), String::new()),
            };
        } else if kind == Kind::Fatal {
            // `!fatal` carries a bare reason word.
            attrs.insert("message".to_owned(), w);
        }
    }
    Ok(Sentence { kind, tag, attrs })
}

#[derive(Debug, Default)]
pub struct Codec;

impl Decoder for Codec {
    type Item = Sentence;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Sentence>, Error> {
        match parse_sentence(src)? {
            Some((words, used)) => {
                src.advance(used);
                // A lone empty word (keep-alive style) yields no sentence.
                if words.is_empty() {
                    return Ok(None);
                }
                into_sentence(words).map(Some)
            }
            None if src.len() > MAX_SENTENCE_BYTES => {
                Err(Error::Protocol("sentence too large".into()))
            }
            None => Ok(None),
        }
    }
}

impl Encoder<Vec<String>> for Codec {
    type Error = Error;

    fn encode(&mut self, words: Vec<String>, dst: &mut BytesMut) -> Result<(), Error> {
        let mut out = Vec::new();
        for w in &words {
            encode_len(w.len(), &mut out);
            out.extend_from_slice(w.as_bytes());
        }
        out.push(0); // end of sentence
        dst.extend_from_slice(&out);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_roundtrip() {
        for len in [0usize, 1, 0x7F, 0x80, 0x3FFF, 0x4000, 0x1F_FFFF, 0x20_0000, 0x0FFF_FFFF, 0x1000_0000] {
            let mut v = Vec::new();
            encode_len(len, &mut v);
            let (got, used) = decode_len(&v).unwrap().unwrap();
            assert_eq!((got, used), (len, v.len()), "len={len}");
        }
    }

    #[test]
    fn partial_input_waits() {
        let mut c = Codec;
        let mut buf = BytesMut::from(&b"\x05!done"[..]);
        assert!(c.decode(&mut buf).unwrap().is_none()); // no terminator yet
        buf.extend_from_slice(b"\x00");
        let s = c.decode(&mut buf).unwrap().unwrap();
        assert_eq!(s.kind, Kind::Done);
    }

    #[test]
    fn parses_attrs_and_tag() {
        let mut c = Codec;
        let mut out = BytesMut::new();
        c.encode(
            vec!["!re".into(), "=name=ether1".into(), "=comment=a=b".into(), ".tag=7".into()],
            &mut out,
        )
        .unwrap();
        let s = c.decode(&mut out).unwrap().unwrap();
        assert_eq!(s.kind, Kind::Re);
        assert_eq!(s.tag, Some(7));
        assert_eq!(s.get("name"), Some("ether1"));
        assert_eq!(s.get("comment"), Some("a=b"));
    }
}
