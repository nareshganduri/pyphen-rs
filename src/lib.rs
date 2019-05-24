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

//! A pure Rust port of Python's [Pyphen][1].
//!
//! [1]: https://pyphen.org/

#![warn(clippy::all)]
#![warn(missing_docs)]

mod alternative_parser;
mod data_int;
mod hyph_dict;
mod pyphen;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::thread_local;

use alternative_parser::AlternativeParser;
use data_int::DataInt;
use hyph_dict::HyphDict;
pub use pyphen::{builder::Builder, iter::Iter, Pyphen};

#[macro_use]
extern crate lazy_static;

use regex::Regex;

// precompile some stuff
lazy_static! {
    static ref PARSE_HEX: Regex = Regex::new(r"\^{2}([0-9a-f]{2})").unwrap();
    static ref PARSE: Regex = Regex::new(r"(\d?)(\D?)").unwrap();
}

thread_local! {
    // cache of per-file HyphDict objects
    static HD_CACHE: RefCell<HashMap<String, Rc<HyphDict>>> = RefCell::new(HashMap::new());

    /// A thread-local copy of all available languages
    pub static LANGUAGES: RefCell<HashMap<String, Rc<String>>> = {
        let mut dict = HashMap::new();
        let dir = format!("{}/dictionaries", env!("CARGO_MANIFEST_DIR"));

        if let Ok(read_dir) = std::fs::read_dir(dir) {
            for entry in read_dir {
                if let Ok(entry) = entry {
                    if let Some(filepath) = entry.path().to_str() {
                        let filename = entry.file_name();
                        let filename = filename
                            .to_str()
                            .unwrap()
                            .trim_start_matches("hyph_")
                            .trim_end_matches(".dic");
                        dict.insert(filename.to_string(), Rc::new(filepath.to_string()));
                    }
                }
            }
        }

        RefCell::new(dict)
    }
}

/// Get a fallback language available in our dictionaries.
///
/// <http://www.unicode.org/reports/tr35/#Locale_Inheritance>
///
/// We use the normal truncation inheritance. This function needs aliases
/// including scripts for languages with multiple regions available.
pub fn language_fallback(language: &str) -> String {
    let language = language.replace('-', "_");
    let mut parts: Vec<_> = language.split('_').collect();

    while !parts.is_empty() {
        let language = parts.join("_");
        let mut flag = false;
        LANGUAGES.with(|l| {
            if l.borrow().contains_key(&language) {
                flag = true;
            }
        });
        if flag {
            return language;
        }

        parts.pop();
    }

    panic!("No language fallback!")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Deref;

    fn match_tuple<T, U>(tup1: (T, U), s1: &str, s2: &str)
    where
        T: Deref<Target = str>,
        U: Deref<Target = str>,
    {
        let (a, b) = tup1;

        assert_eq!(&*a, s1);
        assert_eq!(&*b, s2);
    }

    fn match_iter<T>(iter: Option<(T, T)>, s1: &str, s2: &str)
    where
        T: Deref<Target = str>,
    {
        assert!(iter.is_some());
        let x = iter.unwrap();
        match_tuple(x, s1, s2);
    }

    ///Test the ``inserted`` method.
    #[test]
    fn test_inserted() {
        let dic = Builder::lang("nl_NL").build();
        assert_eq!(dic.inserted("lettergrepen"), "let-ter-gre-pen");
    }

    /// Test the ``wrap`` method.
    #[test]
    fn test_wrap() {
        let dic = Builder::lang("nl_NL").build();
        match_tuple(
            dic.wrap("autobandventieldopje", 11).unwrap(),
            "autoband-",
            "ventieldopje",
        );
    }

    /// Test the ``iterate`` method.
    #[test]
    fn test_iterate() {
        let dic = Builder::lang("nl_NL").build();
        let mut iter = dic.iterate("Amsterdam");
        match_iter(iter.next(), "Amster", "dam");
        match_iter(iter.next(), "Am", "sterdam");
        assert_eq!(iter.next(), None);
    }

    /// Test the ``iterate`` method with a fallback dict.
    #[test]
    fn test_fallback_dict() {
        let dic = Builder::lang("nl_NL-variant").build();
        let mut iter = dic.iterate("Amsterdam");
        match_iter(iter.next(), "Amster", "dam");
        match_iter(iter.next(), "Am", "sterdam");
        assert_eq!(iter.next(), None);
    }

    /// Test a missing dict.
    #[test]
    #[should_panic]
    fn test_missing_dict() {
        Builder::lang("mi_SS").build();
    }

    /// Test a personal dict.
    #[test]
    fn test_personal_dict() {
        let dic = Builder::lang("fr").build();
        assert_ne!(
            dic.inserted("autobandventieldopje"),
            "au-to-band-ven-tiel-dop-je"
        );
        LANGUAGES.with(|l| {
            let nl = {
                let l = l.borrow();
                l["nl_NL"].clone()
            };
            let mut l = l.borrow_mut();
            let fr = l.get_mut("fr").unwrap();
            *fr = nl;
        });
        let dic = Builder::lang("fr").build();
        assert_eq!(
            dic.inserted("autobandventieldopje"),
            "au-to-band-ven-tiel-dop-je"
        );
    }

    /// Test the ``left`` and ``right`` parameters.
    #[test]
    fn test_left_right() {
        let dic = Builder::lang("nl_NL").build();
        assert_eq!(dic.inserted("lettergrepen"), "let-ter-gre-pen");
        let dic = Builder::lang("nl_NL").left(4).build();
        assert_eq!(dic.inserted("lettergrepen"), "letter-gre-pen");
        let dic = Builder::lang("nl_NL").right(4).build();
        assert_eq!(dic.inserted("lettergrepen"), "let-ter-grepen");
        let dic = Builder::lang("nl_NL").left(4).right(4).build();
        assert_eq!(dic.inserted("lettergrepen"), "letter-grepen");
    }

    /// Test the ``filename`` parameter.
    #[test]
    fn test_filename() {
        LANGUAGES.with(|l| {
            let l = l.borrow();
            let filename = l["nl_NL"].clone();

            let dic = Builder::filename(filename).build();
            assert_eq!(dic.inserted("lettergrepen"), "let-ter-gre-pen");
        });
    }

    /// Test the alternative Parser.
    #[test]
    fn test_alternative() {
        let dic = Builder::lang("hu").left(1).right(1).build();
        let mut iter = dic.iterate("kulissza");
        match_iter(iter.next(), "kulisz", "sza");
        match_iter(iter.next(), "ku", "lissza");
        assert_eq!(iter.next(), None);
        assert_eq!(dic.inserted("kulissza"), "ku-lisz-sza");
    }

    /// Test uppercase.
    #[test]
    fn test_upper() {
        let dic = Builder::lang("nl_NL").build();
        assert_eq!(dic.inserted("LETTERGREPEN"), "LET-TER-GRE-PEN");
    }

    /// Test uppercase with alternative Parser.
    #[test]
    fn test_upper_alternative() {
        let dic = Builder::lang("hu").left(1).right(1).build();
        let mut iter = dic.iterate("KULISSZA");
        match_iter(iter.next(), "KULISZ", "SZA");
        match_iter(iter.next(), "KU", "LISSZA");
        assert_eq!(iter.next(), None);
        assert_eq!(dic.inserted("KULISSZA"), "KU-LISZ-SZA");
    }

    /// Test that all included dictionaries can be parsed.
    #[test]
    fn test_all_dictionaries() {
        LANGUAGES.with(|l| {
            for lang in l.borrow().keys() {
                Builder::lang(lang).build();
            }
        });
    }

    /// Test the language fallback algorithm.
    #[test]
    fn test_fallback() {
        assert_eq!(language_fallback("en"), "en");
        assert_eq!(language_fallback("en_US"), "en_US");
        assert_eq!(language_fallback("en_FR"), "en");
        assert_eq!(language_fallback("en-Latn-US"), "en_Latn_US");
        assert_eq!(language_fallback("en-Cyrl-US"), "en");
        assert_eq!(language_fallback("fr-Latn-FR"), "fr");
        assert_eq!(language_fallback("en-US_variant1-x"), "en_US");
    }
}
