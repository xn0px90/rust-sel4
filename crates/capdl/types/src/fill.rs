use core::fmt;
use core::ops::Range;

#[cfg(feature = "deflate")]
use core::iter;

#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FillEntryContentFile {
    pub file: String,
    pub file_offset: usize,
}

// // //

pub trait AvailableFillEntryContent {
    fn copy_out(&self, dst: &mut [u8]);
}

#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FillEntryContentBytes<'a> {
    pub bytes: &'a [u8],
}

#[cfg(feature = "alloc")]
impl<'a> FillEntryContentBytes<'a> {
    pub fn pack(content: &[u8]) -> Vec<u8> {
        content.to_vec()
    }
}

impl<'a> AvailableFillEntryContent for FillEntryContentBytes<'a> {
    fn copy_out(&self, dst: &mut [u8]) {
        dst.copy_from_slice(self.bytes)
    }
}

impl<'a> fmt::Debug for FillEntryContentBytes<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FillEntryContentBytes")
            .field("bytes", &"&[...]")
            .finish()
    }
}

#[cfg(feature = "deflate")]
#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FillEntryContentDeflatedBytes<'a> {
    pub deflated_bytes: &'a [u8],
}

#[cfg(all(feature = "alloc", feature = "deflate"))]
impl<'a> FillEntryContentDeflatedBytes<'a> {
    pub fn pack(content: &[u8]) -> Vec<u8> {
        miniz_oxide::deflate::compress_to_vec(content, 10)
    }
}

#[cfg(feature = "deflate")]
impl<'a> AvailableFillEntryContent for FillEntryContentDeflatedBytes<'a> {
    fn copy_out(&self, dst: &mut [u8]) {
        let n = miniz_oxide::inflate::decompress_slice_iter_to_slice(
            dst,
            iter::once(self.deflated_bytes),
            false, // zlib_header
            true,  // ignore_adler32
        )
        .unwrap();
        assert_eq!(n, dst.len())
    }
}

#[cfg(feature = "deflate")]
impl<'a> fmt::Debug for FillEntryContentDeflatedBytes<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FillEntryContentDeflatedBytes")
            .field("deflated_bytes", &"&[...]")
            .finish()
    }
}

// // //

pub trait AvailableFillEntryContentVia {
    type Via: ?Sized;

    fn copy_out_via(&self, means: &Self::Via, dst: &mut [u8]);
}

impl<T: AvailableFillEntryContent> AvailableFillEntryContentVia for T {
    type Via = ();

    fn copy_out_via(&self, _means: &Self::Via, dst: &mut [u8]) {
        self.copy_out(dst)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FillEntryContentBytesVia {
    pub bytes_range: Range<usize>,
}

impl AvailableFillEntryContentVia for FillEntryContentBytesVia {
    type Via = [u8];

    fn copy_out_via(&self, means: &Self::Via, dst: &mut [u8]) {
        FillEntryContentBytes {
            bytes: &means[self.bytes_range.clone()],
        }
        .copy_out(dst)
    }
}

#[cfg(feature = "deflate")]
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FillEntryContentDeflatedBytesVia {
    pub deflated_bytes_range: Range<usize>,
}

#[cfg(feature = "deflate")]
impl AvailableFillEntryContentVia for FillEntryContentDeflatedBytesVia {
    type Via = [u8];

    fn copy_out_via(&self, means: &Self::Via, dst: &mut [u8]) {
        FillEntryContentDeflatedBytes {
            deflated_bytes: &means[self.deflated_bytes_range.clone()],
        }
        .copy_out(dst)
    }
}
