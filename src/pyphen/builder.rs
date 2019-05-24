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
}

impl Builder<Rc<String>> {
    /// Constructs a new Builder for a given language
    ///
    /// - *lang* - lang of the included dict to use if no filename is given
    pub fn lang(lang: &str) -> Self {
        let mut filename = None;
        LANGUAGES.with(|l| {
            let cpy = l.borrow().get(&language_fallback(lang)).unwrap().clone();
            filename = Some(cpy);
        });
        let filename = filename.unwrap();

        Self {
            filename,
            left: 2,
            right: 2,
            cache: true,
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
    pub fn build(&self) -> Pyphen {
        let Self {
            ref filename,
            left,
            right,
            cache,
        } = *self;
        let filename: &str = &*filename;
        let mut hd = None;

        HD_CACHE.with(|hc| {
            if !cache || !hc.borrow().contains_key(filename) {
                hc.borrow_mut()
                    .insert(filename.to_string(), Rc::new(HyphDict::new(filename)));
            }

            let clone = hc.borrow()[filename].clone();
            hd = Some(clone);
        });
        let hd = hd.unwrap();

        Pyphen { hd, left, right }
    }
}
