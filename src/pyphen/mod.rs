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

use std::borrow::Cow;
use std::rc::Rc;

use super::{DataInt, HyphDict};
use iter::Iter;

pub mod builder;
pub mod iter;

/// Hyphenation class, with methods to hyphenate strings in various ways.
pub struct Pyphen {
    left: usize,
    right: usize,
    hd: Rc<HyphDict>,
}

impl Pyphen {
    /// Get a list of positions where the word can be hyphenated.
    /// The points that are too far to the left or right are removed.
    ///
    /// - *word* - unicode string of the word to hyphenate
    pub fn positions(&self, word: &str) -> Vec<DataInt> {
        let right = word.len() - self.right;
        self.hd
            .positions(word)
            .iter()
            .cloned()
            .filter(|i| i.value >= self.left && i.value <= right)
            .collect()
    }

    /// Iterate over all hyphenation possibilities, the longest first.
    ///
    /// - *word* - unicode string of the word to hyphenate
    pub fn iterate<'b>(&self, word: &'b str) -> Iter<'b> {
        Iter {
            iter: self.positions(word).into_iter().rev(),
            word,
            is_upper: word == word.to_uppercase(),
        }
    }

    /// Get the longest possible first part and the last part of a word.
    ///
    /// The first part has the hyphen already attached.
    ///
    /// Returns ``None`` if there is no hyphenation point before ``width``, or
    /// if the word could not be hyphenated.
    ///
    /// - *word* - unicode string of the word to hyphenate
    /// - *width* - maximum length of the first part
    /// - *hyphen* - unicode string used as hyphen character
    pub fn wrap_with<'b>(
        &self,
        word: &'b str,
        mut width: usize,
        hyphen: &str,
    ) -> Option<(String, Cow<'b, str>)> {
        width -= hyphen.len();
        for (w1, w2) in self.iterate(word) {
            if w1.len() <= width {
                let w1 = w1.into_owned();
                return Some((w1 + hyphen, w2));
            }
        }

        None
    }

    /// Get the longest possible first part and the last part of a word.
    ///
    /// The first part has the hyphen already attached.
    ///
    /// Returns ``None`` if there is no hyphenation point before ``width``, or
    /// if the word could not be hyphenated.
    ///
    /// - *word* - unicode string of the word to hyphenate
    /// - *width* - maximum length of the first part
    pub fn wrap<'b>(&self, word: &'b str, width: usize) -> Option<(String, Cow<'b, str>)> {
        self.wrap_with(word, width, "-")
    }

    /// Get the word as a string with all the possible hyphens inserted.
    ///
    /// - *word* - unicode string of the word to hyphenate
    /// - *hyphen* - unicode string used as hyphen character
    ///
    /// # Example
    /// ```
    /// use pyphen_rs::Builder;
    ///
    /// let dic = Builder::lang("nl_NL").build();
    ///
    /// assert_eq!(dic.inserted_with("lettergrepen", "."), "let.ter.gre.pen");
    /// ```
    pub fn inserted_with(&self, word: &str, hyphen: &str) -> String {
        let mut word_list: Vec<_> = word.chars().collect();
        let is_upper = word == word.to_uppercase();

        for position in self.positions(word).into_iter().rev() {
            if let Some(data) = position.data {
                // get the nonstandard hyphenation data
                let (change, mut index, cut) = data;
                let change = if is_upper {
                    change.to_uppercase()
                } else {
                    change.to_string()
                };
                index += position.value as isize;

                let index = if index < 0 {
                    word_list.len() - index as usize
                } else {
                    index as usize
                };

                word_list.splice(index..(index + cut), change.replace('=', hyphen).chars());
            } else {
                word_list.splice(position.value..position.value, hyphen.chars());
            }
        }

        word_list.into_iter().collect()
    }

    /// Get the word as a string with all the possible hyphens inserted.
    ///
    /// - *word* - unicode string of the word to hyphenate
    ///
    /// # Example
    /// ```
    /// use pyphen_rs::Builder;
    ///
    /// let dic = Builder::lang("nl_NL").build();
    ///
    /// assert_eq!(dic.inserted("lettergrepen"), "let-ter-gre-pen");
    /// ```
    pub fn inserted(&self, word: &str) -> String {
        self.inserted_with(word, "-")
    }
}
