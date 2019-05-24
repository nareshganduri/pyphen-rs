// This file is part of pyphen-rs
//
// Copyright 2008 - Wilbert Berendsen <info@wilbertberendsen.nl>
// Copyright 2012-2013 - Guillaume Ayoub <guillaume.ayoub@kozea.fr>
// Copyright 2019 - Naresh Ganduri <gandurinaresh@gmail.com>
//
// This library is free software.  It is released under the
// GPL 2.0+/LGPL 2.1+/MPL 1.1 tri-license.  See COPYING.GPL, COPYING.LGPL and
// COPYING.MPL for more details.
//
// This library is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more
// details.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader};
use std::rc::Rc;

use regex::Captures;

use super::{AlternativeParser, DataInt, PARSE, PARSE_HEX};

/// Hyphenation patterns.
pub struct HyphDict {
    patterns: HashMap<String, (usize, Vec<DataInt>)>,
    cache: RefCell<HashMap<String, Rc<Vec<DataInt>>>>,
    maxlen: usize,
}

impl HyphDict {
    /// Read a ``hyph_*.dic`` and parse its patterns.
    ///
    /// :param filename: filename of hyph_*.dic to read
    pub fn new(filename: &str) -> Self {
        let mut patterns = HashMap::new();

        let stream = OpenOptions::new().read(true).open(filename).unwrap();
        let stream = BufReader::new(stream);

        for pattern in stream.lines() {
            let pattern = pattern.unwrap();
            if pattern.is_empty() || pattern.starts_with('%') || pattern.starts_with('#') {
                continue;
            }

            // replace ^^hh with the real character
            let mut pattern = PARSE_HEX
                .replace_all(&pattern, |caps: &Captures| {
                    let cap = &caps[1];
                    let num = u8::from_str_radix(cap, 16).unwrap();
                    let ch = num as char;

                    ch.to_string()
                })
                .to_string();

            // read nonstandard hyphen alternatives
            let mut factory = if pattern.contains('/') {
                let idx = pattern.find('/').unwrap();
                let alternative = pattern.split_off(idx + 1);
                pattern.pop();
                Some(AlternativeParser::new(&pattern, &alternative))
            } else {
                None
            };

            let (tags, values): (Vec<_>, Vec<_>) = PARSE
                .captures_iter(&pattern)
                .map(|caps: Captures| {
                    let i = caps
                        .get(1)
                        .map(|m| m.as_str())
                        .filter(|m| !m.is_empty())
                        .unwrap_or("0");
                    let string = caps.get(2).map_or("", |m| m.as_str());

                    let i: usize = i.parse().unwrap();
                    let d = if let Some(factory) = &mut factory {
                        factory.call(&DataInt::new(i, None))
                    } else {
                        DataInt::new(i, None)
                    };

                    (string, d)
                })
                .unzip();

            // if only zeros, skip this pattern
            if values.iter().map(|x| x.value).max().unwrap() == 0 {
                continue;
            }

            // chop zeros from beginning and end, and store start offset
            let start = values.iter().position(|v| v.value != 0).unwrap_or(0);
            let end = values
                .iter()
                .rposition(|v| v.value != 0)
                .unwrap_or(values.len() - 1)
                + 1;

            patterns.insert(tags.concat(), (start, values[start..end].to_vec()));
        }

        let maxlen = patterns.keys().map(String::len).max().unwrap_or(0);

        Self {
            patterns,
            cache: RefCell::new(HashMap::new()),
            maxlen,
        }
    }

    /// Get a list of positions where the word can be hyphenated.
    ///
    /// :param word: unicode string of the word to hyphenate
    ///
    /// E.g. for the dutch word 'lettergrepen' this method returns ``[3, 6,
    /// 9]``.
    ///
    /// Each position is a ``DataInt`` with a data attribute.
    ///
    /// If the data attribute is not ``None``, it contains a tuple with
    /// information about nonstandard hyphenation at that point: ``(change,
    /// index, cut)``.
    ///
    /// change
    ///     a string like ``'ff=f'``, that describes how hyphenation should
    ///     take place.
    ///
    /// index
    ///     where to substitute the change, counting from the current point
    ///
    /// cut
    ///     how many characters to remove while substituting the nonstandard
    ///     hyphenation
    pub fn positions(&self, word: &str) -> Rc<Vec<DataInt>> {
        let word = word.to_lowercase();
        if let Some(points) = self.cache.borrow().get(&word) {
            return points.clone();
        }

        let pointed_word = format!(".{}.", word);
        let mut references = vec![DataInt::new(0, None); pointed_word.len() + 1];

        for i in 0..(pointed_word.len() - 1) {
            for j in (i + 1)..(i + self.maxlen).min(pointed_word.len() + 1) {
                let pattern = self.patterns.get(&pointed_word[i..j]);
                if let Some(pattern) = pattern {
                    let (offset, ref values) = *pattern;
                    let (start, end) = (i + offset, i + offset + values.len());
                    for (x, y) in references[start..end].iter_mut().zip(values.iter()) {
                        if y.value > x.value {
                            *x = y.clone();
                        }
                    }
                }
            }
        }

        let points: Vec<_> = references
            .into_iter()
            .enumerate()
            .filter(|(_, reference)| reference.value % 2 != 0)
            .map(|(i, reference)| DataInt::with_ref(i - 1, &reference))
            .collect();
        let points = Rc::new(points);
        let points2 = Rc::clone(&points);
        self.cache.borrow_mut().insert(word, points);

        points2
    }
}
