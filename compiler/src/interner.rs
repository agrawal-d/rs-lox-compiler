use rustc_hash::FxHashMap as HashMap;
use std::mem;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct StrId(u32);

pub struct Interner {
    map: HashMap<&'static str, StrId>,
    vec: Vec<&'static str>,
    buf: String,
    full: Vec<String>,
}

impl Interner {
    /// Create a new interner with the given buffer capacity (in bytes)
    pub fn with_capacity(cap: usize) -> Interner {
        let cap = cap.next_power_of_two();
        Interner {
            map: HashMap::default(),
            vec: Vec::new(),
            buf: String::with_capacity(cap),
            full: Vec::new(),
        }
    }

    /// Intern string, and get it's ID
    pub fn intern(&mut self, name: &str) -> StrId {
        if let Some(&id) = self.map.get(name) {
            return id;
        }
        let name = unsafe { self.alloc(name) };
        let id = StrId { 0: self.map.len() as u32 };
        self.map.insert(name, id);
        self.vec.push(name);

        debug_assert!(self.lookup(&id) == name);
        debug_assert!(self.intern(name) == id);

        id
    }

    /// Get a string, given it's ID
    pub fn lookup(&self, id: &StrId) -> &str {
        self.vec[id.0 as usize]
    }

    unsafe fn alloc(&mut self, name: &str) -> &'static str {
        let cap = self.buf.capacity();
        if cap < self.buf.len() + name.len() {
            let new_cap = (cap.max(name.len()) + 1).next_power_of_two();
            let new_buf = String::with_capacity(new_cap);
            let old_buf = mem::replace(&mut self.buf, new_buf);
            self.full.push(old_buf);
        }

        let interned = {
            let start = self.buf.len();
            self.buf.push_str(name);
            &self.buf[start..]
        };

        &*(interned as *const str)
    }
}
