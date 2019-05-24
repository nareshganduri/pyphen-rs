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

use crate::DataInt;

/// Iterator over all hyphenation possibilities
pub struct Iter<'a> {
    pub(super) iter: std::iter::Rev<std::vec::IntoIter<DataInt>>,
    pub(super) word: &'a str,
    pub(super) is_upper: bool,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (Cow<'a, str>, Cow<'a, str>);

    fn next(&mut self) -> Option<Self::Item> {
        let position = self.iter.next()?;

        if let Some(data) = position.data {
            // get the nonstandard hyphenation data
            let (change, mut index, cut) = data;
            let change = if self.is_upper {
                change.to_uppercase()
            } else {
                change.to_string()
            };
            index += position.value as isize;
            let (c1, c2) = {
                let mut x = change.split('=');
                (x.next().unwrap(), x.next().unwrap())
            };

            let index = if index < 0 {
                self.word.len() - index as usize
            } else {
                index as usize
            };

            let first = self.word[..index].to_string() + c1;
            let second = c2.to_string() + &self.word[(index + cut)..];
            Some((Cow::Owned(first), Cow::Owned(second)))
        } else {
            let first = &self.word[..position.value];
            let second = &self.word[position.value..];
            Some((Cow::Borrowed(first), Cow::Borrowed(second)))
        }
    }
}