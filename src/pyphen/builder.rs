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

use std::ops::Deref;
use std::rc::Rc;

use super::{HyphDict, Pyphen};
use crate::{language_fallback, HD_CACHE, LANGUAGES};

/// Builder struct to create a hyphenation instance
pub struct Builder<T> {
    left: usize,
    right: usize,
    cache: bool,
    filename: T,
    error: bool,
}

impl Builder<Rc<String>> {
    /// Constructs a new Builder for a given language
    ///
    /// - *lang* - lang of the included dict to use if no filename is given
    pub fn lang(lang: &str) -> Self {
        let mut filename = None;
        let mut error = false;
        LANGUAGES.with(|l| {
            if let Some(fallback) = language_fallback(lang) {
                if let Some(cpy) = l.borrow().get(&fallback) {
                    filename = Some(Rc::clone(&cpy));
                } else {
                    error = true;
                }
            } else {
                error = true;
            }
        });
        let filename = filename.unwrap_or_default();

        Self {
            filename,
            left: 2,
            right: 2,
            cache: true,
            error,
        }
    }
}

impl<T> Builder<T> {
    /// Constructs a new Builder for a given dictionary file
    ///
    /// - *filename* - filename of hyph_*.dic to read
    pub fn filename(filename: T) -> Self {
        Self {
            filename,
            left: 2,
            right: 2,
            cache: true,
            error: false,
        }
    }

    /// Sets the minimum number of characters in the first syllable
    pub fn left(&mut self, left: usize) -> &mut Self {
        self.left = left;
        self
    }

    /// Sets the minimum number of characters in the last syllable
    pub fn right(&mut self, right: usize) -> &mut Self {
        self.right = right;
        self
    }

    /// Sets whether to use a cached copy of the hyphenation patterns
    pub fn cache(&mut self, cache: bool) -> &mut Self {
        self.cache = cache;
        self
    }
}

impl<T> Builder<T>
where
    T: Deref<Target = String>,
{
    /// Create an hyphenation instance for given lang or filename.
    ///
    /// Returns `Err` if the given lang or filename does not exist.
    pub fn build(&self) -> Result<Pyphen, ()> {
        let Self {
            ref filename,
            left,
            right,
            cache,
            mut error,
        } = *self;
        let filename: &str = &*filename;
        let mut hd = None;

        HD_CACHE.with(|hc| {
            if !cache || !hc.borrow().contains_key(filename) {
                if let Ok(hd) = HyphDict::new(filename) {
                    hc.borrow_mut().insert(filename.to_string(), Rc::new(hd));
                } else {
                    error = true;
                }
            }

            if let Some(x) = hc.borrow().get(filename) {
                hd = Some(Rc::clone(x));
            } else {
                error = true;
            }
        });

        if error {
            Err(())
        } else {
            let hd = hd.unwrap();

            Ok(Pyphen { hd, left, right })
        }
    }
}
