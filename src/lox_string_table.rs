use std::borrow::Borrow;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::ops;
use std::ptr;

// A lox string is an interned string. Other libs (like servo's string-cache) do
// fancy tricks to encode a single unsafe_data member as multiple things like a
// tag, pointer, etc, along with refcounting via drop.
//
// For now, we'll use an pointer, with the string table having ownership of all
// strings. Strings can only be removed by calling explicit removal or the table being destroyed,
// which will check against the refcount stored.
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

impl Drop for InternalStringEntry {
    fn drop(&mut self) {
        // Sanity check that refcount is 0.
        assert_eq!(
            *self.refcount.borrow(),
            0,
            "Internal string entry being dropped with outstanding references"
        );
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoxString {
    string: *const str,
    entry: Cell<*const InternalStringEntry>,
}

impl LoxString {
    pub fn as_str(&self) -> &str {
        unsafe { &*self.string }
    }
}

impl Drop for LoxString {
    // Drop doesn't actually cleanup anything right now, but it decrements the internal
    // refcount for debugging verification. We check the internal refcount when we
    // remove from the table.
    //
    // We know the pointer is valid if non-null, because the table holds a
    // Box<InternalStringEntry>, which means it will not move in memory.
    fn drop(&mut self) {
        if self.entry.get() != ptr::null_mut() {
            unsafe {
                *(*(self.entry.get() as *mut InternalStringEntry))
                    .refcount
                    .borrow_mut() -= 1;
            }
        }
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
// TODO - Does Rc<Box<str>> work?
//
// Thus, in order to implement a correct interning table, it must be a
// Box<InternalStringEntries> that themselves contain a Box<str>, as the pointers
// to the internal entries cannot moved as the LoxString uses them in drop to
// maintain refcounts and the string itself cannot move as that's the whole point
// of interning.
//
// Note that this is exactly what servo's string-cache does, but it's even better
// by encoding the refcount _and_ pointer in a single u64 tag, utilizing the Entry
// structure's size to know that memory will be aligned, and available for use.
// Obviously if this wasn't a learning project, using that would be the better
// idea.
//
// For now, use a hash set containing boxes of internal strings, with strings that
// will only be removed by the gc.
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
            entry: Cell::new(internal_string.as_ref()),
            string: internal_string.data.as_ref() as *const str,
        }
    }

    // Allocate a string from an existing Box<str>.
    //
    // If the string is not interned, we keep the Box. If it's already interned, we
    // return a LoxString with the Box held by the table.
    fn allocate_string_from_box(&mut self, box_string: Box<str>) -> LoxString {
        // See if we need to insert this Box<str> or not. Hold a pointer to the string
        // because we use it for lookup later, but the box itself becomes invalid if we
        // insert it.
        let string: *const str = box_string.as_ref();
        if !self.table.contains(box_string.as_ref()) {
            self.table.insert(Box::new(InternalStringEntry {
                refcount: RefCell::new(0),
                data: box_string,
            }));
        }

        // The borrow checker rightly complains that this is unsafe, but we
        // know that the string is still valid because the box regardless of being
        // added to the table or not, is not dropped until the end of the function.
        let internal_string = unsafe { self.table.get(&*string).unwrap() };

        // NOTE: The refcount isn't used to determine the hash, but we need to
        // increment it for our bookkeeping. Using RefCell, we check at runtime
        // that this borrow can only happen one at a time.
        {
            *internal_string.refcount.borrow_mut() += 1;
        }

        LoxString {
            entry: Cell::new(internal_string.as_ref()),
            string: internal_string.data.as_ref() as *const str,
        }
    }

    // Concatenate two strings, returning a new LoxString.
    //
    // TODO - To save on the memory copy, one could probably check the table based on
    // both strings (creating hash from both). However, that would require either a custom hashtable,
    // or some fancy impl Borrow<str> function taking a tuple of (&str, &str) that somehow
    // could return a single &str (seems impossible).
    pub fn concatenate(&mut self, left: &LoxString, right: &LoxString) -> LoxString {
        // First, build a new Box containing the result of both strings.
        // TODO - would manually making a box of the right size be better? This potentially goes through one realloc.
        // let new_string = Box::new([left.as_str()..right.as_str().len()]);
        let new_string = format!("{}{}", left.as_str(), right.as_str()).into_boxed_str();

        // Returned the interned string from the table.
        self.allocate_string_from_box(new_string)
    }

    // Remove the interned string from the table, returning true or false if it was removed.
    // The passed in string must be the last owner, and the internal string entry pointer
    // will be removed, so when the LoxString is dropped, there is no dangling pointer.
    //
    // TODO - return Result type vs panic?
    pub fn remove_string(&mut self, string: &LoxString) {
        // Check first that this even exists in the table.
        let entry = self
            .table
            .get(string.as_str())
            .expect("Asked to remove a string not present");

        // This must be the last owner. Manually clear the last owner, and
        // remove the link in LoxString. Sanity check that the pointer stored
        // is actually the pointer of the Box<InternalStringEntry>.
        assert_eq!(entry.refcount.replace(0), 1);
        let entry_ptr = string.entry.replace(ptr::null());
        assert_eq!(entry.as_ref() as *const InternalStringEntry, entry_ptr);

        // Removal must succeed
        assert!(self.table.remove(string.as_str()) == true);
    }
}

#[cfg(test)]
mod tests {

    use super::LoxStringTable;

    #[test]
    fn basic_test() {
        let mut table = LoxStringTable::new();

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

        // Check refcounts
        unsafe {
            assert_eq!(*(*first.entry.get()).refcount.borrow(), 3);
            assert_eq!(*(*different.entry.get()).refcount.borrow(), 1);
        }

        // Force some hashmap reallocations, then check everything again.
        table.table.shrink_to_fit();
        table.table.reserve(100000);
        table.table.shrink_to_fit();

        assert_eq!(first, second);
        assert_eq!(first, third);
        assert_eq!(second, third);

        // sanity check that the internals match
        assert_eq!(first.as_str(), "abcd");
        assert_eq!(first.entry.get(), second.entry.get());

        assert_ne!(first, different);
        assert_eq!(table.table.len(), 2);

        // Check refcounts
        unsafe {
            assert_eq!(*(*first.entry.get()).refcount.borrow(), 3);
            assert_eq!(*(*different.entry.get()).refcount.borrow(), 1);
        }

        // Every string ref should be dropped, so fair game to cleanup.
    }

    #[test]
    fn test_remove() {
        let mut table = LoxStringTable::new();

        {
            // Allocate the string, then drop it.
            let unique = table.allocate_string_from_str("abcd");

            assert_eq!(table.table.len(), 1);

            table.remove_string(&unique);
            assert_eq!(table.table.len(), 0);
        }

        {
            let first = table.allocate_string_from_str("abcd");
            {
                let second = table.allocate_string_from_str("abcd");
                assert_eq!(table.table.len(), 1);
                assert_eq!(first, second);
            }
            table.remove_string(&first);
            assert_eq!(table.table.len(), 0);
        }
        assert_eq!(table.table.len(), 0);

        // Make a string that should get dropped again.
        let _first = table.allocate_string_from_str("abcd");
        assert_eq!(table.table.len(), 1);
    }

    #[test]
    fn box_allocate() {
        let mut table = LoxStringTable::new();

        // Test inserting a box string
        {
            let raw_string: Box<str> = Box::from("abcd");
            let raw_ptr: *const str = raw_string.as_ref();

            let first = table.allocate_string_from_box(raw_string);
            let second = table.allocate_string_from_str("abcd");

            assert_eq!(raw_ptr, first.string);
            assert_eq!(raw_ptr, second.string);
            assert_eq!(first, second);
            assert_eq!(table.table.len(), 1);

            // sanity check that the internals match
            assert_eq!(first.as_str(), "abcd");

            unsafe {
                assert_eq!(*(*first.entry.get()).refcount.borrow(), 2);
                assert_eq!(*(*second.entry.get()).refcount.borrow(), 2);
            }

            // remove the string from the table.
            drop(first);
            assert_eq!(table.table.len(), 1);
            table.remove_string(&second);
            assert_eq!(table.table.len(), 0);
        }

        // Test box string returning already interened string
        {
            let raw_string: Box<str> = Box::from("asdfasdf");
            let raw_ptr: *const str = raw_string.as_ref();

            let first = table.allocate_string_from_str("asdfasdf");
            let second = table.allocate_string_from_box(raw_string);

            assert_ne!(raw_ptr, first.string);
            assert_ne!(raw_ptr, second.string);
            assert_eq!(first, second);
            assert_eq!(table.table.len(), 1);

            // sanity check that the internals match
            assert_eq!(second.as_str(), "asdfasdf");

            unsafe {
                assert_eq!(*(*first.entry.get()).refcount.borrow(), 2);
                assert_eq!(*(*second.entry.get()).refcount.borrow(), 2);
            }

            // remove the string from the table.
            drop(first);
            assert_eq!(table.table.len(), 1);
            table.remove_string(&second);
            assert_eq!(table.table.len(), 0);
        }
    }

    #[test]
    fn concatenate_test() {
        let mut table = LoxStringTable::new();

        // test inserting new string
        let first = table.allocate_string_from_str("abcd");
        let second = table.allocate_string_from_str("asdf");

        let result = table.concatenate(&first, &second);
        assert_eq!(table.table.len(), 3);
        let third = table.concatenate(&first, &second);
        assert_eq!(table.table.len(), 3);
        assert_eq!(result, third);

        // test creating already interned string
        let first = table.allocate_string_from_str("hello \n");
        let second = table.allocate_string_from_str("world \n");
        assert_eq!(table.table.len(), 5);
        let third = table.allocate_string_from_str("hello \nworld \n");
        let result = table.concatenate(&first, &second);
        assert_eq!(table.table.len(), 6);
        assert_eq!(third, result);
    }
}
