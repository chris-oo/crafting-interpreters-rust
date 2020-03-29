use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::ops;

// A lox string is an interned string. Other libs (like servo's string-cache) do
// fancy tricks to encode a single unsafe_data member as multiple things like a
// tag, pointer, etc, along with refcounting via drop.
//
// For now, we'll use an pointer, with the string table having ownership of all
// strings. Strings can only be removed by calling explicit removal which will check against the refcount stored.
struct InternalStringEntry {
    // NOTE - this isn't thread safe at all, but it's okay because Lox is single threaded.
    refcount: RefCell<u64>,
    // Data is never modified, only read.
    data: Box<str>,
}

impl Hash for InternalStringEntry {
    // Can't derive hash because the refcount can change. Only hash the actual string.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

impl ops::Deref for InternalStringEntry {
    type Target = str;

    fn deref(&self) -> &str {
        self.data.as_ref()
    }
}

impl PartialEq for InternalStringEntry {
    fn eq(&self, other: &InternalStringEntry) -> bool {
        *self.data == *other.data
    }
}

// See https://stackoverflow.com/questions/45384928/is-there-any-way-to-look-up-in-hashset-by-only-the-value-the-type-is-hashed-on
// This allows us to search the hashmap using &str instead of InternalStringEntries.
impl Borrow<str> for Box<InternalStringEntry> {
    fn borrow(&self) -> &str {
        &*&*self
    }
}

impl Eq for InternalStringEntry {}

#[derive(Debug, Clone, PartialEq)]
pub struct LoxString {
    string: *const str,
    unsafe_ptr: *const InternalStringEntry,
}

impl LoxString {
    // Concatenate two strings, returning a new LoxString.
    fn concatenate(table: &mut LoxStringTable, left: &LoxString, right: &LoxString) -> Self {
        // // First, build a new Rc containing the result of both strings.
        // let refcount: Rc<String> = Rc::new(format!("{}{}", left.refcount, right.refcount));

        // // Returned the interned string from the table.
        // table.allocate_string_from_rc(&refcount)
        unimplemented!()
    }

    pub fn as_str(&self) -> &str {
        unsafe { &*self.string }
    }
}

impl Drop for LoxString {
    // Drop doesn't actually cleanup anything, but it decrements the internal refcount for debugging verification.
    // We check the internal refcount when we remove from the table.
    fn drop(&mut self) {
        unsafe {
            *(*self.unsafe_ptr).refcount.borrow_mut() -= 1;
        }
    }
}

// A table containing interned strings. A simple example here:
// https://github.com/rust-lang/rfcs/blob/master/text/1845-shared-from-slice.md
//
// However note that example only works for strings where their size is known at
// compile time, due to how Rc works. Thus, in order to implement a correct interning table, it must be a Box<InternalStringEntries> that themselves contain a Box<str>
//
// Note that one could do better by not using Rc and having LoxString have a raw
// pointer to a Box<str> owned by the table with a refcount managed by implementing
// drop, but that's a bit trickier to implement. (This is what servo's string-cache
// does)
//
// For now, use a hash set containing boxes of internal strings, with strings that
// will only be removed by the gc.
//

pub struct LoxStringTable {
    table: HashSet<Box<InternalStringEntry>>,
}

impl LoxStringTable {
    pub fn new() -> Self {
        Self {
            table: HashSet::new(),
        }
    }

    // Allocate a string from a non owning slice.
    pub fn allocate_string_from_str(&mut self, string: &str) -> LoxString {
        // Insert a new Rc if we don't have one already.
        if !self.table.contains(string) {
            self.table.insert(Box::new(InternalStringEntry {
                refcount: RefCell::new(0),
                data: string.into(),
            }));
        }

        // Get the interned string.
        let internal_string = self.table.get(string).unwrap();

        // NOTE: The refcount isn't used to determine the hash, but we need to
        // increment it for our bookkeeping. Using RefCell, we check at runtime
        // that this borrow can only happen one at a time.
        {
            *internal_string.refcount.borrow_mut() += 1;
        }

        LoxString {
            unsafe_ptr: internal_string,
            string: internal_string.data.as_ref() as *const str,
        }
    }

    // Allocate a string from an existing Box<str>.
    //
    // If the string is not interned, we keep the Box. If it's already interned, we
    // return a LoxString with the Box held by the table.
    //
    fn allocate_string_from_box(&mut self, box_string: Box<str>) -> LoxString {
        // See if we need to insert this Box<str> or not.
        // Hold a ref to the string because we use it for lookup later, but the box becomes invalid if we insert.
        let string: &str = box_string.as_ref();
        if !self.table.contains(string) {
            self.table.insert(InternalStringEntry {
                refcount: RefCell::new(0),
                data: box_string,
            });
        }

        // Get the interned string.
        let internal_string = self.table.get(string).unwrap();

        // NOTE: The refcount isn't used to determine the hash, but we need to
        // increment it for our bookkeeping. Using RefCell, we check at runtime
        // that this borrow can only happen one at a time.
        {
            *internal_string.refcount.borrow_mut() += 1;
        }

        LoxString {
            unsafe_ptr: internal_string,
            string: internal_string.data.as_ref() as *const str,
        }
    }

    // Remove the interned string from the table, returning true or false if it was removed.
    pub fn remove_string(&mut self, string: &LoxString) -> bool {
        self.table.remove(string.as_str())
    }
}

#[cfg(test)]
mod tests {

    use super::LoxStringTable;

    #[test]
    fn basic_test() {
        let mut table = LoxStringTable::new();

        {
            let first = table.allocate_string_from_str("abcd");
            let second = table.allocate_string_from_str("abcd");
            let third = table.allocate_string_from_str("abcd");
            let different = table.allocate_string_from_str("abcde");
            assert_eq!(first, second);
            assert_eq!(first, third);
            assert_eq!(second, third);

            // sanity check that the internals match
            assert_eq!(first.as_str(), "abcd");

            assert_ne!(first, different);
            assert_eq!(table.table.len(), 2);

            // assert_eq!(Rc::strong_count(&first.refcount), 4);
            // assert_eq!(Rc::strong_count(&second.refcount), 4);
        }
    }

    // #[test]
    // fn rc_allocate() {
    //     let mut table = LoxStringTable::new();

    //     let raw_string = Rc::from("abcd");

    //     let second = table.allocate_string_from_rc(&raw_string);
    //     let third = table.allocate_string_from_str("abcd");

    //     assert!(Rc::ptr_eq(&raw_string, &second.refcount));
    //     assert!(Rc::ptr_eq(&raw_string, &third.refcount));
    //     assert_eq!(second, third);

    //     // sanity check that the internals match
    //     assert_eq!(third.refcount.as_ref(), "abcd");

    //     assert_eq!(Rc::strong_count(&raw_string), 4);
    //     assert_eq!(Rc::strong_count(&second.refcount), 4);
    // }
}
