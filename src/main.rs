struct StyleSheetBuf {
    source_ptr: *mut String,
    sheet: StyleSheet<'static>,
}

impl StyleSheetBuf {
    // "Safely" (?) creates a new `StyleSheetBuf`. To do so, we require an owned string type which
    // will then be "leaked" after moving it to the heap. This is safe because since we own it it
    // should never be dropped without our consent.
    //
    // We then get a raw pointer and use it to get a static reference, which is used to create a
    // "normal" style sheet. Such reference is, of course, static. This means that it may live up
    // until the end of the program.
    pub fn new(source: String) -> StyleSheetBuf {
        let boxed_source = Box::new(source);
        let source_ptr = Box::into_raw(boxed_source);

        // SAFETY: The deref is safe since we just obtained the pointer by using `into_raw`,
        // which must return a "safe" pointer that fulfills all the deref requirements.
        let str_ref: &'static str = unsafe { (*source_ptr).as_str() };

        let sheet = StyleSheet::parse(str_ref);
        StyleSheetBuf { source_ptr, sheet }
    }

    // Self explanatory.
    pub fn sheet(&self) -> &StyleSheet<'static> {
        &self.sheet
    }
}

// "Safely" (?) drop it.
impl Drop for StyleSheetBuf {
    fn drop(&mut self) {
        // SAFETY: It is safe because there is no way to drop such pointer before (i.e. a double
        // free can't happen) and the pointer wasn't modified since its creation by `into_raw`.
        unsafe {
            drop(Box::from_raw(self.source_ptr));
        }
    }
}

impl std::ops::Deref for StyleSheetBuf {
    type Target = StyleSheet<'static>;

    fn deref(&self) -> &Self::Target {
        self.sheet()
    }
}

struct StyleSheet<'s> {
    // Just keep some reference.
    parsed: &'s str,
}

impl<'s> StyleSheet<'s> {
    pub fn parse(source: &'s str) -> Self {
        StyleSheet {
            parsed: source.trim(),
        }
    }
}

fn main() {
    let foo: StyleSheetBuf = parse_file("foo.txt");
    print_parsed(&foo);
}

fn print_parsed(sheet: &StyleSheet) {
    println!("parsed: {:?}", sheet.parsed);
}

fn parse_file(path: &str) -> StyleSheetBuf {
    let source = std::fs::read_to_string(path).unwrap();
    StyleSheetBuf::new(source)
}
