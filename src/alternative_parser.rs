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

use std::rc::Rc;

use super::DataInt;

/// Parser of nonstandard hyphen pattern alternative.
///
/// The instance returns a special int with data about the current position in
/// the pattern when called with an odd value.
pub struct AlternativeParser {
    change: Rc<String>,
    index: isize,
    cut: usize,
}

impl AlternativeParser {
    pub fn new(pattern: &str, alternative: &str) -> Self {
        let alternative: Vec<_> = alternative.split(',').collect();
        let mut ap = Self {
            change: Rc::new(alternative[0].to_string()),
            index: alternative[1].parse().unwrap(),
            cut: alternative[2].parse().unwrap(),
        };

        if pattern.starts_with('.') {
            ap.index += 1;
        }

        ap
    }

    pub fn call(&mut self, value: &DataInt) -> DataInt {
        self.index -= 1;
        if value.value & 1 != 0 {
            let Self {
                ref change,
                index,
                cut,
            } = *self;

            DataInt::new(value.value, Some((Rc::clone(change), index, cut)))
        } else {
            DataInt::new(value.value, None)
        }
    }
}