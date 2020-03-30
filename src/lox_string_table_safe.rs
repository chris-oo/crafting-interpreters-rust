use std::borrow::Borrow;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::num::Wrapping;
use std::ops;
use std::ptr;
use std::rc::Rc;

// A lox string is an interned string. Other libs (like servo's string-cache) do
// fancy tricks to encode a single unsafe_data member as multiple things like a
// tag, pointer, etc, along with refcounting via drop.
//
// For the safe lib, use an Rc<InternalStringEntry>, but don't just let that be the type as
// we want to control some things about equality and storing hashes for hashes of LoxString.
#[derive(Debug)]
struct InternalStringEntry {
    hash: u32,
    // string is never modified, only read.
    string: String,
}

impl Hash for InternalStringEntry {
    // Only hash the actual string.
    // NOTE - Can't just hash the stored hash, it needs to match the str we supply when using Hashset::get.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.string.hash(state);
    }
}

impl ops::Deref for InternalStringEntry {
    type Target = str;

    fn deref(&self) -> &str {
        self.string.as_str()
    }
}

impl PartialEq for InternalStringEntry {
    fn eq(&self, other: &InternalStringEntry) -> bool {
        self.string == other.string
    }
}

impl Eq for InternalStringEntry {}

#[derive(Debug, PartialEq, Eq, Hash)]
struct InternalStringEntryRc(Rc<InternalStringEntry>);

// See https://stackoverflow.com/questions/45384928/is-there-any-way-to-look-up-in-hashset-by-only-the-value-the-type-is-hashed-on
// This allows us to search the hashmap using &str instead of InternalStringEntries.
//
// NOTE - Due to rules on trait implementation in crates, we can't do this:
// impl Borrow<str> for Rc<InternalStringEntry> { ... }
// Instead, we have to use a wrapper tuple.
impl Borrow<str> for InternalStringEntryRc {
    fn borrow(&self) -> &str {
        &*(self.0)
    }
}

#[derive(Debug, Clone)]
pub struct LoxString {
    entry: Rc<InternalStringEntry>,
}

impl LoxString {
    pub fn as_str(&self) -> &str {
        &*self.entry
    }

    fn get_hash(&self) -> u32 {
        self.entry.hash
    }
}

impl Hash for LoxString {
    // Only hash the hash, no need to hash the whole string as it's precomputed.
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(self.get_hash());
    }
}

impl PartialEq for LoxString {
    fn eq(&self, other: &LoxString) -> bool {
        // Since all Lox strings are interned, a pointer equality check is all that's needed.
        Rc::ptr_eq(&self.entry, &other.entry)
    }
}

impl Eq for LoxString {}

impl fmt::Display for LoxString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// A table containing interned strings. A simple example here:
// https://github.com/rust-lang/rfcs/blob/master/text/1845-shared-from-slice.md
//
// However note that example only works for strings where their size is known at
// compile time, due to how Rc<T> works, as T is declared inline in the Rc struct.
// Which would be fine in a tokenizer, but not fine when the size of a string isn't
// know at runtime, like with string concatenation.
//
// Thus instead we hold Rc<InternalStringEntry> which also contains a String which
// is heap allocated. Note the differences from this one vs the unsafe table, namely
// we're not reimplementing a bastardized Rc ourselves.
//
// Still, string-cache is probably a more optimized thing to use in a language
// runtime, or something like it. Unsafe does allow you to do some tricks with
// inline strings, tag/pointer packing etc which can matter when performance
// is needed, like in a production language runtime.
pub struct LoxStringTable {
    table: HashSet<InternalStringEntryRc>,
}

impl LoxStringTable {
    pub fn new() -> Self {
        Self {
            table: HashSet::new(),
        }
    }

    // TODO - probably a std lib implementation of FNV-1a using hasher
    fn hash_string(string: &str) -> u32 {
        let mut hash = Wrapping(2166136261u32);

        for c in string.bytes() {
            hash ^= Wrapping::<u32>(c.into());
            hash *= Wrapping::<u32>(16777619);
        }

        hash.0
    }

    fn make_new_string_entry(string: String) -> InternalStringEntryRc {
        InternalStringEntryRc(Rc::new(InternalStringEntry {
            hash: LoxStringTable::hash_string(string.as_str()),
            string: string,
        }))
    }

    // Allocate a string from a non owning slice.
    pub fn allocate_string_from_str(&mut self, string: &str) -> LoxString {
        // Insert a new Rc if we don't have one already.
        if !self.table.contains(string) {
            self.table
                .insert(LoxStringTable::make_new_string_entry(string.into()));
        }

        // Get the interned string.
        let internal_string = self.table.get(string).unwrap();

        LoxString {
            entry: internal_string.0.clone(),
        }
    }

    // Allocate a string from an existing String.
    //
    // If the string is not interned, we clone the Rc. If it's already interned, we
    // return a LoxString with the Rc held by the table.
    fn allocate_string_from_string(&mut self, string: String) -> LoxString {
        // See if we need to insert this String or not. Hold a pointer to the string
        // because we use it for lookup later, but the string itself is invalid post insertion
        // due to the borrow.
        let string_ptr: *const str = string.as_str();
        if !self.table.contains(string.as_str()) {
            self.table
                .insert(LoxStringTable::make_new_string_entry(string));
        }

        // The borrow checker rightly complains that this is unsafe, but we
        // know that the string is still valid because the string regardless of being
        // added to the table or not, is not dropped until the end of the function.
        let internal_string = unsafe { self.table.get(&*string_ptr).unwrap() };

        LoxString {
            entry: internal_string.0.clone(),
        }
    }

    // Concatenate two strings, returning a new LoxString.
    //
    // TODO - To save on the memory copy, one could probably check the table based on
    // both strings (creating hash from both). However, that would require either a custom hashtable,
    // or some fancy impl Borrow<str> function taking a tuple of (&str, &str) that somehow
    // could return a single &str (seems impossible).
    pub fn concatenate(&mut self, left: &LoxString, right: &LoxString) -> LoxString {
        let left_str = left.as_str();
        let right_str = right.as_str();
        let mut new_string = String::with_capacity(left_str.len() + right_str.len());
        new_string.push_str(left_str);
        new_string.push_str(right_str);

        // Returned the interned string from the table.
        self.allocate_string_from_string(new_string)
    }

    // Remove the interned string from the table, returning true or false if it was removed.
    // The passed in string must be the last owner, and the internal string entry pointer
    // will be removed, so when the LoxString is dropped, there is no dangling pointer.
    //
    // TODO - return Result type vs panic?
    //     pub fn remove_string(&mut self, string: &LoxString) {
    //         // Take the string, it should exist.
    //         let entry = self
    //             .table
    //             .take(string.as_str())
    //             .expect("Asked to remove a string not present");

    //         // This must be the last owner. Manually clear the last owner, and
    //         // remove the link in LoxString. Sanity check that the pointer stored
    //         // is actually the pointer of the Box<InternalStringEntry>.
    //         assert_eq!(entry.refcount.replace(0), 1);
    //         let entry_ptr = string.entry.replace(ptr::null());
    //         assert_eq!(entry.0.as_ref() as *const InternalStringEntry, entry_ptr);
    //     }
}

// #[cfg(test)]
// mod tests {

//     use super::LoxStringTable;

//     #[test]
//     fn basic_test() {
//         let mut table = LoxStringTable::new();

//         let first = table.allocate_string_from_str("abcd");
//         let second = table.allocate_string_from_str("abcd");
//         let third = table.allocate_string_from_str("abcd");
//         let different = table.allocate_string_from_str("abcde");
//         assert_eq!(first, second);
//         assert_eq!(first, third);
//         assert_eq!(second, third);

//         // sanity check that the internals match
//         assert_eq!(first.as_str(), "abcd");

//         assert_ne!(first, different);
//         assert_eq!(table.table.len(), 2);

//         // Check refcounts
//         unsafe {
//             assert_eq!(*(*first.entry.get()).refcount.borrow(), 3);
//             assert_eq!(*(*different.entry.get()).refcount.borrow(), 1);
//         }

//         let raw_entry_ptr = first.entry.get();

//         // Force some hashmap reallocations, then check everything again.
//         table.table.shrink_to_fit();
//         table.table.reserve(100000);
//         table.table.shrink_to_fit();

//         let fourth = table.allocate_string_from_str("abcd");

//         assert_eq!(first, second);
//         assert_eq!(first, third);
//         assert_eq!(second, third);
//         assert_eq!(first, fourth);

//         // sanity check that the internals match
//         assert_eq!(first.as_str(), "abcd");
//         assert_eq!(first.entry.get(), second.entry.get());
//         assert_eq!(first.entry.get(), raw_entry_ptr);
//         assert_eq!(first, fourth);
//         assert_eq!(first.entry.get(), fourth.entry.get());

//         assert_ne!(first, different);
//         assert_eq!(table.table.len(), 2);

//         // Check refcounts
//         unsafe {
//             assert_eq!(*(*first.entry.get()).refcount.borrow(), 4);
//             assert_eq!(*(*fourth.entry.get()).refcount.borrow(), 4);
//             assert_eq!(*(*different.entry.get()).refcount.borrow(), 1);
//         }

//         // Clone a string
//         let fifth = fourth.clone();

//         assert_eq!(first, fifth);
//         assert_eq!(fourth, fifth);
//         assert_eq!(table.table.len(), 2);

//         // Check refcounts
//         unsafe {
//             assert_eq!(*(*first.entry.get()).refcount.borrow(), 5);
//             assert_eq!(*(*fourth.entry.get()).refcount.borrow(), 5);
//             assert_eq!(*(*fifth.entry.get()).refcount.borrow(), 5);
//             assert_eq!(*(*different.entry.get()).refcount.borrow(), 1);
//         }

//         // Every string ref should be dropped, so fair game to cleanup.
//     }

//     #[test]
//     fn test_remove() {
//         let mut table = LoxStringTable::new();

//         {
//             // Allocate the string, then drop it.
//             let unique = table.allocate_string_from_str("abcd");

//             assert_eq!(table.table.len(), 1);

//             table.remove_string(&unique);
//             assert_eq!(table.table.len(), 0);
//         }

//         {
//             let first = table.allocate_string_from_str("abcd");
//             {
//                 let second = table.allocate_string_from_str("abcd");
//                 assert_eq!(table.table.len(), 1);
//                 assert_eq!(first, second);
//             }
//             table.remove_string(&first);
//             assert_eq!(table.table.len(), 0);
//         }
//         assert_eq!(table.table.len(), 0);

//         // Make a string that should get dropped again.
//         let _first = table.allocate_string_from_str("abcd");
//         assert_eq!(table.table.len(), 1);
//     }

//     #[test]
//     fn box_allocate() {
//         let mut table = LoxStringTable::new();

//         // Test inserting a box string
//         {
//             let raw_string: Box<str> = Box::from("abcd");
//             let raw_ptr: *const str = raw_string.as_ref();

//             let first = table.allocate_string_from_box(raw_string);
//             let second = table.allocate_string_from_str("abcd");

//             assert_eq!(raw_ptr, first.string);
//             assert_eq!(raw_ptr, second.string);
//             assert_eq!(first, second);
//             assert_eq!(table.table.len(), 1);

//             // sanity check that the internals match
//             assert_eq!(first.as_str(), "abcd");

//             unsafe {
//                 assert_eq!(*(*first.entry.get()).refcount.borrow(), 2);
//                 assert_eq!(*(*second.entry.get()).refcount.borrow(), 2);
//             }

//             // remove the string from the table.
//             drop(first);
//             assert_eq!(table.table.len(), 1);
//             table.remove_string(&second);
//             assert_eq!(table.table.len(), 0);
//         }

//         // Test box string returning already interened string
//         {
//             let raw_string: Box<str> = Box::from("asdfasdf");
//             let raw_ptr: *const str = raw_string.as_ref();

//             let first = table.allocate_string_from_str("asdfasdf");
//             let second = table.allocate_string_from_box(raw_string);

//             assert_ne!(raw_ptr, first.string);
//             assert_ne!(raw_ptr, second.string);
//             assert_eq!(first, second);
//             assert_eq!(table.table.len(), 1);

//             // sanity check that the internals match
//             assert_eq!(second.as_str(), "asdfasdf");

//             unsafe {
//                 assert_eq!(*(*first.entry.get()).refcount.borrow(), 2);
//                 assert_eq!(*(*second.entry.get()).refcount.borrow(), 2);
//             }

//             // remove the string from the table.
//             drop(first);
//             assert_eq!(table.table.len(), 1);
//             table.remove_string(&second);
//             assert_eq!(table.table.len(), 0);
//         }
//     }

//     #[test]
//     fn concatenate_test() {
//         let mut table = LoxStringTable::new();

//         // test inserting new string
//         let first = table.allocate_string_from_str("abcd");
//         let second = table.allocate_string_from_str("asdf");

//         let result = table.concatenate(&first, &second);
//         assert_eq!(table.table.len(), 3);
//         let third = table.concatenate(&first, &second);
//         assert_eq!(table.table.len(), 3);
//         assert_eq!(result, third);

//         // test creating already interned string
//         let first = table.allocate_string_from_str("hello \n");
//         let second = table.allocate_string_from_str("world \n");
//         assert_eq!(table.table.len(), 5);
//         let third = table.allocate_string_from_str("hello \nworld \n");
//         let result = table.concatenate(&first, &second);
//         assert_eq!(table.table.len(), 6);
//         assert_eq!(third, result);
//     }
// }
