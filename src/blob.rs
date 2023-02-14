pub struct ReferenceTag
{

}

//  Object to hold dynamically allocated opaque binary data.
//  On modern compilers, it will be movable but not copyable. Copies
//  must be explicitly created by set_deep_copy.
//  On older compilers, it is copyable for syntactical reasons.
#[derive(Default,Debug,Clone,PartialOrd, PartialEq)]
struct Blob {
    data: Vec<u8>,
    // _size: usize,
    owned: bool
}


impl Blob {
    //  Creates an empty Blob.
    pub fn with_size(size_in: usize) -> Self {
        Self {
            data: Vec::with_capacity(size_in),
            // _size: size_in,
            owned: true
        }
    }

    //  Creates a Blob of a given size, an initializes content by copying
    // from another buffer.
    pub fn from_buffer(size_in: usize, data_in: &[u8]) -> Self {
        let mut out = Self {
           data: Vec::with_capacity(size_in),
            // _size: size_in,
            owned: true
        };
        out.data.clone_from_slice(data_in);
        out
    }

    //  Creates a Blob for temporary use that only references a
    //  pre-allocated block of data.
    //  Use with caution and ensure that the Blob will not outlive
    //  the referenced data.
    // TODO

    //  Returns the size of the Blob.
    pub fn size(&self) -> usize {
        self.data.len()
    }

    //  Returns a pointer to the data of the Blob.
    pub fn data(&self) -> *const u8 {
        self.data.as_ptr()
    }

    //  Returns a pointer to the data of the Blob.
    pub fn data_mut(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }

    //  Sets a Blob to a deep copy of another Blob.
    pub fn set_deep_copy(&mut self, other: &Self) {
        self.data.clone_from_slice(other.data.as_slice());
        self.owned = other.owned;
    }

    //  Sets a Blob to a copy of a given buffer.
    pub fn set(&mut self, data: &[u8], size: usize) {
        self.data.clone_from_slice(data)
    }

    //  Empties a Blob.
    pub fn clear(&mut self) {
        self.data.clear();
    }
}


// #endif
