struct StyleSheetBuf {
    source_ptr: *mut String,

    // Why use an option here? See:
    // https://doc.rust-lang.org/nomicon/destructors.html
    sheet: Option<StyleSheet<'static>>,
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
        StyleSheetBuf {
            source_ptr,
            sheet: Some(sheet),
        }
    }

    // Self explanatory.
    pub fn sheet(&self) -> &StyleSheet<'static> {
        // SAFETY: `self.sheet` only is `None` within `StyleSheetBuf`'s destructor.
        unsafe { self.sheet.as_ref().unwrap_unchecked() }
    }
}

// "Safely" (?) drop it.
impl Drop for StyleSheetBuf {
    fn drop(&mut self) {
        // Ensures the underlying `StyleSheet` is dropped before the string is dropped. This is
        // needed to avoid an use after free in the case of `StyleSheet` defining its own destructor
        // which could then use the string we need to drop next.
        drop(self.sheet.take());
        // The `self.sheet` field is now `None`, which means that `StyleSheet`s destructor won't be
        // executed again. It is now safe to clean up the string.

        println!("  Will drop StyleSheetBuf...");

        // SAFETY: It is safe because there is no way to drop such pointer before (i.e. a double
        // free can't happen) and the pointer wasn't modified since its creation by `into_raw`.
        unsafe {
            drop(Box::from_raw(self.source_ptr));
        }

        println!("    Done.");
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

impl Drop for StyleSheet<'_> {
    fn drop(&mut self) {
        println!("  Will drop StyleSheet:");
        println!("    Just making a final read... -> {:?}", self.parsed);
        println!("    Done.");
    }
}

fn main() {
    let string = String::from(" olá. ");
    let owned_sheet = StyleSheetBuf::new(string);

    print_parsed(&owned_sheet);
    println!("Will drop...");
    drop(owned_sheet);
    println!("Finished drop.");
}

fn print_parsed(sheet: &StyleSheet) {
    println!("parsed: {:?}", sheet.parsed);
}

#[allow(dead_code)]
fn parse_file(path: &str) -> StyleSheetBuf {
    let source = std::fs::read_to_string(path).unwrap();
    StyleSheetBuf::new(source)
}
