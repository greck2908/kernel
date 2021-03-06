//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//! `x86` and `x86_64` descriptor tables (IDT, GDT, or LDT)
//!
//! For more information, refer to the _Intel® 64 and IA-32 Architectures
//! Software Developer’s Manual_, Vol. 3A, section 3.2, "Using Segments", and
//! section 6.10, "Interrupt Descriptor Table (IDT)".

#![deny(missing_docs)]
// use memory::PAddr;
use core::mem::size_of;

/// A pointer to a descriptor table.
/// This is a format suitable
#[repr(C, packed)]
pub struct Pointer<T: DTable> { /// the length of the descriptor table
                     pub limit: u16
                   , /// pointer to the region in memory
                     /// containing the descriptor table.
                     pub base: *const T
                   }
unsafe impl<T: DTable> Sync for Pointer<T> { }
/// A descriptor table (IDT or GDT).
///
/// The IA32 architecture uses two descriptor table structures, the GDT
/// (Global Descriptor Table), which is used for configuring segmentation,
/// and the IDT (Interrupt Descriptor Table), which tells the CPU where
/// interrupt service routines are located.
///
/// As SOS relies on paging rather than segmentation for memory protection on
/// both 32-bit and 64-bit systems, we use the GDT only minimally. However, the
/// CPU still requires a correctly configured GDT to run in protected mode, even
/// if it is not actually used.
///
/// This trait specifies base functionality common to both types of descriptor
/// table.
pub trait DTable: Sized {
    /// The type of an entry in this descriptor table.
    ///
    /// For an IDT, these are
    /// interrupt [`Gate`](../interrupts/idt/gate/struct.Gate.html)s,
    /// while for a GDT or LDT, they are segment
    /// [`Descriptor`](../segment/struct.Descriptor.html)s.
    //  TODO: can there be a trait for DTable entries?
    //      - eliza, 10/06/2016
    type Entry: Sized;

    /// Get the IDT pointer struct to pass to `lidt` or `lgdt`
    ///
    /// This expects that the object implementing `DTable` not contain
    /// additional data before or after the actual `DTable`, if you wish
    /// to attach information to a descriptor table besides the array of
    /// entries that it consists of, it will be necessary to encose the
    /// descriptor table in another `struct` or `enum` type.
    //  TODO: can we have an associated `Entry` type + a function to get the
    //        number of entries in the DTable, instead? that way, we could
    //        calculate the limit using that information, allowing Rust code
    //        to place more variables after the array in the DTable structure.
    //
    //        If we wanted to be really clever, we could probably also have a
    //        method to get a pointer to a first entry (or enforce that the
    //        DTable supports indexing?) and then we could get a pointer only
    //        to the array segment of the DTable, while still allowing variables
    //        to be placed before/after the array.
    //
    //        I'm not sure if we actually want to support this – is there really
    //        a use-case for it? I suppose it would also make our size calc.
    //        more correct in case Rust ever puts additional data around a
    //        DTable rray, but I imagine it will probably never do that...
    //              – eliza, 06/03/2016
    //
    #[inline]
    fn get_ptr(&self) -> Pointer<Self> {
        Pointer {
            limit: (size_of::<Self::Entry>() * self.entry_count()) as u16
          , base: self as *const _
        }
    }

    /// Returns the number of Entries in the `DTable`.
    ///
    /// This is used for calculating the limit.
    fn entry_count(&self) -> usize;

    /// Load the descriptor table with the appropriate load instruction
    fn load(&'static self);
}
