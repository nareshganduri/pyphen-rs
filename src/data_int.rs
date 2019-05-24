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

use std::fmt;
use std::rc::Rc;

/// ``int`` with some other data can be stuck to in a ``data`` attribute.
#[derive(Clone)]
pub struct DataInt {
    pub value: usize,
    pub data: Option<(Rc<String>, isize, usize)>,
}

impl DataInt {
    /// Create a new ``DataInt``.
    pub fn new(value: usize, data: Option<(Rc<String>, isize, usize)>) -> Self {
        Self { value, data }
    }

    // Create a new with ``DataInt`` to using the data from another
    /// ``DataInt``.
    pub fn with_ref(value: usize, reference: &DataInt) -> Self {
        Self {
            value,
            data: reference.data.clone(),
        }
    }
}

impl fmt::Debug for DataInt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
