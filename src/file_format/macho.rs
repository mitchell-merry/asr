//! Support for parsing Mach-O format

#[cfg(feature = "alloc")]
use core::iter::FusedIterator;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
use alloc::format;
#[cfg(feature = "alloc")]
use crate::{string::ArrayCString, Error};
use crate::{ print_message, Address, PointerSize, Process};

// Magic mach-o header constants from:
// https://opensource.apple.com/source/xnu/xnu-4570.71.2/EXTERNAL_HEADERS/mach-o/loader.h.auto.html
const MH_MAGIC_32: u32 = 0xfeedface;
const MH_CIGAM_32: u32 = 0xcefaedfe;
const MH_MAGIC_64: u32 = 0xfeedfacf;
const MH_CIGAM_64: u32 = 0xcffaedfe;

/// Checks if a given Mach-O module is 64-bit or 32-bit
pub fn pointer_size(process: &Process, range: (Address, u64)) -> Option<PointerSize> {
    match process.read::<u32>(scan_macho_page(process, range)?).ok()? {
        MH_MAGIC_64 | MH_CIGAM_64 => Some(PointerSize::Bit64),
        MH_MAGIC_32 | MH_CIGAM_32 => Some(PointerSize::Bit32),
        _ => None,
    }
}

/// Scans the range for a page that begins with Mach-O Magic
fn scan_macho_page(process: &Process, range: (Address, u64)) -> Option<Address> {
    const PAGE_SIZE: u64 = 0x1000;
    let (addr, len) = range;
    // negation mod PAGE_SIZE
    let distance_to_page = (PAGE_SIZE - (addr.value() % PAGE_SIZE)) % PAGE_SIZE;
    // round up to the next multiple of PAGE_SIZE
    let first_page = addr + distance_to_page;
    for i in 0..((len - distance_to_page) / PAGE_SIZE) {
        let a = first_page + (i * PAGE_SIZE);
        match process.read::<u32>(a) {
            Ok(MH_MAGIC_64 | MH_CIGAM_64 | MH_MAGIC_32 | MH_CIGAM_32) => {
                return Some(a);
            }
            _ => (),
        }
    }
    None
}

// Constants for the cmd field of load commands, the type
// https://opensource.apple.com/source/xnu/xnu-4570.71.2/EXTERNAL_HEADERS/mach-o/loader.h.auto.html
/// link-edit stab symbol table info
#[cfg(feature = "alloc")]
const LC_SYMTAB: u32 = 0x2;
/// 64-bit segment of this file to be mapped
#[cfg(feature = "alloc")]
const LC_SEGMENT_64: u32 = 0x19;

#[cfg(feature = "alloc")]
struct MachOFormatOffsets {
    number_of_commands: u32,
    load_commands: u32,
    command_size: u32,
    symtab_offset: u32,
    number_of_symbols: u32,
    strtab_offset: u32,
    nlist_value: u32,
    size_of_nlist_item: u32,
    segcmd64_vmaddr: u32,
    segcmd64_fileoff: u32,
}

#[cfg(feature = "alloc")]
impl MachOFormatOffsets {
    const fn new() -> Self {
        // offsets taken from:
        //  - https://github.com/hackf5/unityspy/blob/master/src/HackF5.UnitySpy/Offsets/MachOFormatOffsets.cs
        //  - https://opensource.apple.com/source/xnu/xnu-4570.71.2/EXTERNAL_HEADERS/mach-o/loader.h.auto.html
        MachOFormatOffsets {
            number_of_commands: 0x10,
            load_commands: 0x20,
            command_size: 0x04,
            symtab_offset: 0x08,
            number_of_symbols: 0x0c,
            strtab_offset: 0x10,
            nlist_value: 0x08,
            size_of_nlist_item: 0x10,
            segcmd64_vmaddr: 0x18,
            segcmd64_fileoff: 0x28,
        }
    }
}

/// A symbol exported into the current module.
#[cfg(feature = "alloc")]
pub struct Symbol {
    /// The address associated with the current function
    pub address: Address,
    /// The address storing the name of the current function
    name_addr: Address,
}

#[cfg(feature = "alloc")]
impl Symbol {
    /// Tries to retrieve the name of the current function
    pub fn get_name<const CAP: usize>(
        &self,
        process: &Process,
    ) -> Result<ArrayCString<CAP>, Error> {
        process.read(self.name_addr)
    }
}

/// Iterates over the exported symbols for a given module.
/// Only 64-bit Mach-O format is supported
#[cfg(feature = "alloc")]
pub fn symbols(
    process: &Process,
    range: (Address, u64),
) -> Option<impl FusedIterator<Item = Symbol> + '_> {
    let page = scan_macho_page(process, range)?;
    let page = Address::from(0x113e2b000_u64);
    print_message(&format!("0x{} - 0x{}", range.0, range.0 + range.1));
    print_message(&format!("0x{}", page));
    let offsets = MachOFormatOffsets::new();
    let number_of_commands: u32 = process.read(page + offsets.number_of_commands).ok()?;

    let mut symtab_fileoff: u32 = 0;
    let mut number_of_symbols: u32 = 0;
    let mut strtab_fileoff: u32 = 0;
    let mut map_fileoff_to_vmaddr: BTreeMap<u64, u64> = BTreeMap::new();

    let mut next: u32 = offsets.load_commands;
    for _i in 0..number_of_commands {
        print_message(&format!("addr {}", page + next));
        let cmdtype: u32 = process.read(page + next).ok()?;
        print_message(&format!("cmdtype {}", cmdtype));

        if cmdtype == LC_SYMTAB {
            symtab_fileoff = process.read(page + next + offsets.symtab_offset).ok()?;
            // print_message(&format!("symtab_fileoff {}", symtab_fileoff));
            number_of_symbols = process.read(page + next + offsets.number_of_symbols).ok()?;
            // print_message(&format!("number_of_symbols {}", number_of_symbols));
            strtab_fileoff = process.read(page + next + offsets.strtab_offset).ok()?;
            // print_message(&format!("strtab_fileoff {}", strtab_fileoff));
        } else if cmdtype == LC_SEGMENT_64 {
            let name = process.read::<ArrayCString<26>>(page + next + 0x8).ok()?;
            // print_message(&format!("name {}", name.validate_utf8().unwrap()));
            let vmaddr: u64 = process.read(page + next + offsets.segcmd64_vmaddr).ok()?;
            // print_message(&format!("vmaddr {}", vmaddr));
            let fileoff: u64 = process.read(page + next + offsets.segcmd64_fileoff).ok()?;
            // print_message(&format!("fileoff {}", fileoff));
            map_fileoff_to_vmaddr.insert(fileoff, vmaddr);
        }
        let command_size: u32 = process.read(page + next + offsets.command_size).ok()?;
        next += command_size;
    }

    if symtab_fileoff == 0 || number_of_symbols == 0 || strtab_fileoff == 0 {
        return None;
    }

    print_message(&format!("map_fileoff_to_vmaddr {:?}",map_fileoff_to_vmaddr));
    let symtab_vmaddr = fileoff_to_vmaddr(&map_fileoff_to_vmaddr, symtab_fileoff as u64);
    // print_message(&format!("symtab_vmaddr {:X} {:X}", symtab_vmaddr, symtab_fileoff));

    let strtab_vmaddr = fileoff_to_vmaddr(&map_fileoff_to_vmaddr, strtab_fileoff as u64);
    // print_message(&format!("strtab_vmaddr {:X} {:X}", strtab_vmaddr, strtab_fileoff));

    Some(
        (0..number_of_symbols)
            .filter_map(move |j| {
                let nlist_item = page + symtab_vmaddr + (j * offsets.size_of_nlist_item);
                // print_message(&format!("nlist {}", nlist_item));
                let symname_offset: u32 = process.read(nlist_item).ok()?;
                // print_message(&format!("symname_offset {}", symname_offset));
                let string_address = page + strtab_vmaddr + symname_offset;
                // print_message(&format!("string_address {}", string_address));
                let symbol_fileoff = process.read(nlist_item + offsets.nlist_value).ok()?;
                // print_message(&format!("fileoff {}", symbol_fileoff));
                let symbol_vmaddr = fileoff_to_vmaddr(&map_fileoff_to_vmaddr, symbol_fileoff);
                // print_message(&format!("vmaddr {}", symbol_vmaddr));
                let symbol_address = page + symbol_vmaddr;
                Some(Symbol {
                    address: symbol_address,
                    name_addr: string_address,
                })
            })
            .fuse(),
    )
}

#[cfg(feature = "alloc")]
fn fileoff_to_vmaddr(map: &BTreeMap<u64, u64>, fileoff: u64) -> u64 {
    map.iter()
        .filter(|(&k, _)| k <= fileoff)
        .max_by_key(|(&k, _)| k)
        .map(|(&k, &v)| v + fileoff - k)
        .unwrap_or(fileoff)
}
