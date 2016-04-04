use memory::PAddr;

use core::mem;
use core::fmt;

// Distinguished section indices.
pub const SHN_UNDEF: u16        = 0;
pub const SHN_LORESERVE: u16    = 0xff00;
pub const SHN_LOPROC: u16       = 0xff00;
pub const SHN_HIPROC: u16       = 0xff1f;
pub const SHN_LOOS: u16         = 0xff20;
pub const SHN_HIOS: u16         = 0xff3f;
pub const SHN_ABS: u16          = 0xfff1;
pub const SHN_COMMON: u16       = 0xfff2;
pub const SHN_XINDEX: u16       = 0xffff;
pub const SHN_HIRESERVE: u16    = 0xffff;

pub const SHT_LOOS: u32   = 0x60000000;
pub const SHT_HIOS: u32   = 0x6fffffff;
pub const SHT_LOPROC: u32 = 0x70000000;
pub const SHT_HIPROC: u32 = 0x7fffffff;
pub const SHT_LOUSER: u32 = 0x80000000;
pub const SHT_HIUSER: u32 = 0xffffffff;

/// Represents an ELF section header
///
/// Refer to the [ELF standard](http://www.sco.com/developers/gabi/latest/ch4.sheader.html)
/// for more information.
#[derive(Debug)]
#[repr(C)]
pub struct Header {
    /// This member specifies the name of the section.
    ///
    /// Its value is an index into the section header string table section,
    /// giving the location of a null-terminated string.
    name_offset: u32
  , /// This member categorizes the section's contents and semantics.
    ty: TypeRepr
  , flags: Flags
  , pub address: PAddr
  , offset: PAddr
  , pub length: PAddr
  , link: u32
  , info: u32
  , address_align: u32
  , entry_length: PAddr
}

bitflags! {
    flags Flags: usize {
        // Flags (SectionHeader::flags)
        const SHF_WRITE            =        0x1
      , const SHF_ALLOC            =        0x2
      , const SHF_EXECINSTR        =        0x4
      , const SHF_MERGE            =       0x10
      , const SHF_STRINGS          =       0x20
      , const SHF_INFO_LINK        =       0x40
      , const SHF_LINK_ORDER       =       0x80
      , const SHF_OS_NONCONFORMING =      0x100
      , const SHF_GROUP            =      0x200
      , const SHF_TLS              =      0x400
      , const SHF_COMPRESSED       =      0x800
      , const SHF_MASKOS           = 0x0ff00000
      , const SHF_MASKPROC         = 0xf0000000
    }
}

impl fmt::LowerHex for Flags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.bits.fmt(f)
    }
}

bitflags! {
    flags GroupFlags: u32 {
        const GRP_COMDAT	=        0x1
      , const GRP_MASKOS	= 0x0ff00000
      , const GRP_MASKPROC	= 0xf0000000
    }
}

impl Header {

    /// Returns true if this section is writable.
    #[inline] pub fn is_writable(&self) -> bool {
        self.flags.contains(SHF_WRITE)
    }

    /// Returns true if this section occupies memory during program execution.
    #[inline] pub fn is_allocated(&self) -> bool {
        self.flags.contains(SHF_ALLOC)
    }

    /// Returns true if this section contains executable instructions.
    #[inline] pub fn is_executable(&self) -> bool {
        self.flags.contains(SHF_EXECINSTR)
    }

    /// Returns true if this section can be merged.
    #[inline] pub fn is_mergeable(&self) -> bool {
        self.flags.contains(SHF_MERGE)
    }

    /// Returns true if this section contains data that is of a uniform size.
    #[inline] pub fn is_uniform(&self) -> bool {
        self.flags.contains(SHF_MERGE) && !self.flags.contains(SHF_STRINGS)
    }
}

pub enum Contents<'a> {
    Empty
  , Undefined(&'a [u8])
  , Group { flags: &'a u32, indicies: &'a[u32] }
}

#[derive(Debug, Copy, Clone)]
struct TypeRepr(u32);

impl TypeRepr {
    #[inline] fn as_type(&self) -> Type {
        match self.0 {
            0 => Type::Null
          , 1 => Type::ProgramBits
          , 2 => Type::SymbolTable
          , 3 => Type::StringTable
          , 4 => Type::Rela
          , 5 => Type::HashTable
          , 6 => Type::Dynamic
          , 7 => Type::Notes
          , 8 => Type::NoBits
          , 9 => Type::Rel
          , 10 => Type::Shlib
          , 11 => Type::DynSymTable
          , 14 => Type::InitArray
          , 15 => Type::FiniArray
          , 16 => Type::PreInitArray
          , x @ SHT_LOOS ... SHT_HIOS => Type::OSSpecific(x)
          , x @ SHT_LOPROC ... SHT_HIPROC => Type::ProcessorSpecific(x)
          , x @ SHT_LOUSER ... SHT_HIUSER => Type::User(x)
          , _ => panic!("Invalid section type!")
        }
    }
}

/// Enum representing an ELF file section type.
///
/// Refer to Figure 1-10: "Section Types, sh_type" in Section 1 of the
/// [ELF standard](http://www.sco.com/developers/gabi/latest/ch4.sheader.html)
/// for more information.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Type {
    /// Section type 0: `SHT_NULL`
    ///
    /// This value marks the section header as inactive; it does not have an
    /// associated section. Other members of the section header have
    /// undefined values.
    Null
  , /// Section type 1: `SHT_PROGBITS`
    ///
    /// The section holds information defined by the program, whose format and
    /// meaning are determined solely by the program.
    ProgramBits
  , /// Section type 2: `SHT_SYMTAB`
    ///
    /// Typically, `SHT_SYMTAB` provides symbols for link editing, though it
    /// may also be used for dynamic linking. As a complete symbol table, it
    /// may contain many symbols unneces- sary for dynamic linking.
    ///
    /// Consequently, an object file may also contain a `SHT_DYNSYM` section,
    /// which holds a minimal set of dynamic linking symbols, to save space.
    SymbolTable
  , /// Section type 3: `SHT_STRTAB`
    ///
    /// The section holds a string table. An object file may have multiple
    /// string table sections.
    StringTable
  , /// Section type 4: `SHT_RELA`
    ///
    /// The section holds relocation entries with explicit addends, such as
    /// type `Elf32_Rela` for the 32-bit class of object files. An object file
    /// may have multiple relocation sections.
    Rela
  , /// Section type 5: `SHT_HASH`
    ///
    /// The section holds a symbol hash table. All objects participating in
    /// dynamic linking must contain a symbol hash table. Currently, an object
    /// file may have only one hash table, but this restriction may be relaxed
    /// in the future.
    HashTable
  , /// Section type 6: `SHT_DYNAMIC`
    ///
    /// The section holds information for dynamic linking. Currently, an object
    /// file may have only one dynamic section, but this restriction may be
    ///  relaxed in the future.
    Dynamic
  , /// Section type 7: `SHT_NOTE`
    ///
    /// The section holds information that marks the file in some way.
    Notes
  , /// Section type 8: `SHT_NOBITS`
    ///
    /// A section of this type occupies no space in the file but otherwise
    /// resembles `SHT_PROGBITS`. Although this section contains no bytes, the
    /// `sh_offset` member contains the conceptual file offset.
    NoBits
  , /// Section type 9: `SHT_REL`
    ///
    /// The section holds relocation entries without explicit addends, such as
    /// type `Elf32_Rel` for the 32-bit class of object files. An object file
    /// may have multiple reloca- tion sections.
    Rel
  , /// Section type 10: `SHT_SHLIB`
    ///
    /// This section type is reserved but has unspecified semantics. Programs
    /// that contain a section of this type do not conform to the ABI.
    Shlib
  , /// Section type 11: `SHT_DYNSYM`
    ///
    /// Typically, `SHT_SYMTAB` provides symbols for link editing, though it
    /// may also be used for dynamic linking. As a complete symbol table, it
    /// may contain many symbols unneces- sary for dynamic linking.
    ///
    /// Consequently, an object file may also contain a `SHT_DYNSYM` section,
    /// which holds a minimal set of dynamic linking symbols, to save space.
    DynSymTable
  , InitArray
  , FiniArray
  , PreInitArray
  , Group
  , SymbolTableShIndex
  , OSSpecific(u32)
  , ProcessorSpecific(u32)
  , User(u32)
}

//
// #[derive(Debug)]
// #[repr(u32)]
// pub enum Flags { Writable    = 0x1
//                , Allocated   = 0x2
//                , Executable  = 0x4
//                }
