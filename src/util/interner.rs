// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! An "interner" is a data structure that associates values with usize tags and
//! allows bidirectional lookup; i.e. given a value, one can easily find the
//! type, and vice versa.

use syntax::ast::Name;

use std::borrow::Borrow;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::ops::Deref;
use std::rc::Rc;

pub struct Interner<T> {
    map: RefCell<HashMap<T, Name>>,
    vect: RefCell<Vec<T> >,
}

// when traits can extend traits, we should extend index<Name,T> to get []
impl<T: Eq + Hash + Clone + 'static> Interner<T> {
    pub fn new() -> Interner<T> {
        Interner {
            map: RefCell::new(HashMap::new()),
            vect: RefCell::new(Vec::new()),
        }
    }

    pub fn prefill(init: &[T]) -> Interner<T> {
        let rv = Interner::new();
        for v in init {
            rv.intern((*v).clone());
        }
        rv
    }

    pub fn intern(&self, val: T) -> Name {
        let mut map = self.map.borrow_mut();
        match (*map).get(&val) {
            Some(&idx) => return idx,
            None => (),
        }

        let mut vect = self.vect.borrow_mut();
        let new_idx = Name((*vect).len() as u32);
        (*map).insert(val.clone(), new_idx);
        (*vect).push(val);
        new_idx
    }

    pub fn gensym(&self, val: T) -> Name {
        let mut vect = self.vect.borrow_mut();
        let new_idx = Name((*vect).len() as u32);
        // leave out of .map to avoid colliding
        (*vect).push(val);
        new_idx
    }

    pub fn get(&self, idx: Name) -> T {
        let vect = self.vect.borrow();
        (*vect)[idx.usize()].clone()
    }

    pub fn len(&self) -> usize {
        let vect = self.vect.borrow();
        (*vect).len()
    }

    pub fn find<Q: ?Sized>(&self, val: &Q) -> Option<Name>
    where T: Borrow<Q>, Q: Eq + Hash {
        let map = self.map.borrow();
        match (*map).get(val) {
            Some(v) => Some(*v),
            None => None,
        }
    }

    pub fn clear(&self) {
        *self.map.borrow_mut() = HashMap::new();
        *self.vect.borrow_mut() = Vec::new();
    }
}

#[derive(Clone, PartialEq, Hash, PartialOrd)]
pub struct RcStr {
    string: Rc<String>,
}

impl RcStr {
    pub fn new(string: &str) -> RcStr {
        RcStr {
            string: Rc::new(string.to_string()),
        }
    }
}

impl Eq for RcStr {}

impl Ord for RcStr {
    fn cmp(&self, other: &RcStr) -> Ordering {
        self[..].cmp(&other[..])
    }
}

impl fmt::Debug for RcStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::fmt::Debug;
        self[..].fmt(f)
    }
}

impl fmt::Display for RcStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::fmt::Display;
        self[..].fmt(f)
    }
}

impl Borrow<str> for RcStr {
    fn borrow(&self) -> &str {
        &self.string[..]
    }
}

impl Deref for RcStr {
    type Target = str;

    fn deref(&self) -> &str { &self.string[..] }
}

/// A StrInterner differs from Interner<String> in that it accepts
/// &str rather than RcStr, resulting in less allocation.
pub struct StrInterner {
    map: RefCell<HashMap<RcStr, Name>>,
    vect: RefCell<Vec<RcStr> >,
}

/// When traits can extend traits, we should extend index<Name,T> to get []
impl StrInterner {
    pub fn new() -> StrInterner {
        StrInterner {
            map: RefCell::new(HashMap::new()),
            vect: RefCell::new(Vec::new()),
        }
    }

    pub fn prefill(init: &[&str]) -> StrInterner {
        let rv = StrInterner::new();
        for &v in init { rv.intern(v); }
        rv
    }

    pub fn intern(&self, val: &str) -> Name {
        let mut map = self.map.borrow_mut();
        match map.get(val) {
            Some(&idx) => return idx,
            None => (),
        }

        let new_idx = Name(self.len() as u32);
        let val = RcStr::new(val);
        map.insert(val.clone(), new_idx);
        self.vect.borrow_mut().push(val);
        new_idx
    }

    pub fn gensym(&self, val: &str) -> Name {
        let new_idx = Name(self.len() as u32);
        // leave out of .map to avoid colliding
        self.vect.borrow_mut().push(RcStr::new(val));
        new_idx
    }

    // I want these gensyms to share name pointers
    // with existing entries. This would be automatic,
    // except that the existing gensym creates its
    // own managed ptr using to_managed. I think that
    // adding this utility function is the most
    // lightweight way to get what I want, though not
    // necessarily the cleanest.

    /// Create a gensym with the same name as an existing
    /// entry.
    pub fn gensym_copy(&self, idx : Name) -> Name {
        let new_idx = Name(self.len() as u32);
        // leave out of map to avoid colliding
        let mut vect = self.vect.borrow_mut();
        let existing = (*vect)[idx.usize()].clone();
        vect.push(existing);
        new_idx
    }

    pub fn get(&self, idx: Name) -> RcStr {
        (*self.vect.borrow())[idx.usize()].clone()
    }

    pub fn len(&self) -> usize {
        self.vect.borrow().len()
    }

    pub fn find<Q: ?Sized>(&self, val: &Q) -> Option<Name>
    where RcStr: Borrow<Q>, Q: Eq + Hash {
        match (*self.map.borrow()).get(val) {
            Some(v) => Some(*v),
            None => None,
        }
    }

    pub fn clear(&self) {
        *self.map.borrow_mut() = HashMap::new();
        *self.vect.borrow_mut() = Vec::new();
    }

    pub fn reset(&self, other: StrInterner) {
        *self.map.borrow_mut() = other.map.into_inner();
        *self.vect.borrow_mut() = other.vect.into_inner();
    }
}