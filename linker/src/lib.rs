pub mod archive;

use anyhow::{Result, anyhow};
use indexmap::IndexMap;
use object::{
    ComdatKind, File, Object, ObjectComdat, ObjectSection, ObjectSymbol, Relocation,
    RelocationEncoding, RelocationKind, RelocationTarget, SectionIndex, SectionKind, SymbolIndex,
    SymbolKind, SymbolSection,
    pe::*,
    write::pe::{NtHeaders, Writer},
};
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug)]
pub struct Linker<'data> {
    //键代表文件名
    pub inputs: IndexMap<&'data str, File<'data>>,
    pub output: String,

    pub idata_section: Vec<u8>,
    pub sections: IndexMap<(&'data str, &'data str, SectionIndex), Section>,
    //合并后的段的地址和大小
    pub section_address: IndexMap<SectionKind, u64>,
    pub section_size: IndexMap<SectionKind, u64>,
    pub section_align: u64,

    pub global_symbols: IndexMap<&'data str, Symbol>,
    pub local_symbols: IndexMap<&'data str, IndexMap<(&'data str, SymbolIndex), Symbol>>,
    pub comdats: IndexMap<&'data str, Comdat>,

    pub image_base: u64,
    pub entry_address: u64,
}

#[derive(Debug)]
pub struct Section {
    //在同一段中的偏移
    pub offset: u64,
    pub kind: SectionKind,
    pub data: Vec<u8>,
    pub relocation: Vec<(u64, Relocation)>,
}

#[derive(Debug)]
pub struct Symbol {
    pub kind: SymbolKind,
    pub section_index: SectionIndex,
    pub address: u64,
    pub file_index: usize,
    pub is_weak: bool,
    pub size: u64,
}

#[derive(Debug)]
pub struct Comdat {
    pub kind: ComdatKind,
}

impl<'data> Linker<'data> {
    pub fn new(inputs: IndexMap<&'data str, File<'data>>, output: String) -> Linker<'data> {
        Linker {
            inputs,
            output,

            idata_section: [0; 20].to_vec(),
            sections: IndexMap::new(),
            section_address: IndexMap::new(),
            section_size: IndexMap::new(),
            section_align: 0x1000,

            global_symbols: IndexMap::new(),
            local_symbols: IndexMap::new(),
            comdats: IndexMap::new(),

            //TODO 根据实际情况
            image_base: 0x140000000,
            entry_address: 0x1000,
        }
    }

    pub fn link(&mut self) -> Result<()> {
        for (i, (path, file)) in self.inputs.iter().enumerate() {
            for comdat in file.comdats() {
                self.comdats.insert(
                    comdat.name()?,
                    Comdat {
                        kind: comdat.kind(),
                    },
                );
            }
            for section in file.sections() {
                self.sections.insert(
                    (*path, section.name()?, section.index()),
                    Section {
                        offset: *self.section_size.get(&section.kind()).unwrap_or(&0),
                        kind: section.kind(),
                        data: section.data()?.to_vec(),
                        relocation: section.relocations().collect(),
                    },
                );

                if let Some(t) = self.section_size.get_mut(&section.kind()) {
                    *t += section.data()?.len() as u64;
                } else {
                    self.section_size
                        .insert(section.kind(), section.data()?.len() as u64);
                }
            }

            for symbol in file.symbols() {
                if symbol.is_undefined() {
                    continue;
                }

                let value = Symbol {
                    kind: symbol.kind(),
                    section_index: match symbol.section() {
                        SymbolSection::Section(index) => index,
                        _ => continue,
                    },
                    address: symbol.address(),
                    file_index: i,
                    is_weak: symbol.is_weak(),
                    size: symbol.size(),
                };

                if symbol.is_global() {
                    let symbols = &mut self.global_symbols;
                    if let Some(pre_symbol) = symbols.get_mut(symbol.name()?) {
                        let mut duplicate = !symbol.is_weak() && !pre_symbol.is_weak;
                        let mut keep = !symbol.is_weak();

                        if let Some(comdat) = self.comdats.get(symbol.name()?) {
                            match comdat.kind {
                                ComdatKind::NoDuplicates => duplicate = true,
                                ComdatKind::Any => duplicate = false,
                                ComdatKind::SameSize => {
                                    duplicate = symbol.size() != pre_symbol.size
                                }
                                ComdatKind::Largest => keep = symbol.size() > pre_symbol.size,
                                ComdatKind::Newest => keep = true,
                                _ => {}
                            }
                        }
                        if duplicate {
                            return Err(anyhow!(
                                "duplicate symbol: '{}' in '{path}'",
                                symbol.name()?
                            ));
                        } else if !keep {
                            continue;
                        }
                    }
                    symbols.insert(symbol.name()?, value);
                } else {
                    if let None = self.local_symbols.get(*path) {
                        self.local_symbols.insert(*path, IndexMap::new());
                    }
                    let symbols = &mut self.local_symbols.get_mut(*path).unwrap();
                    symbols.insert((symbol.name()?, symbol.index()), value);
                }
            }
        }

        let mut offset: u64 = 0;
        for (kind, size) in &self.section_size {
            offset = offset.div_ceil(self.section_align) * self.section_align;

            self.section_address
                .insert(*kind, self.image_base + self.entry_address + offset);

            offset += *size;
        }

        self.relocate()?;
        self.write()?;

        Ok(())
    }

    pub fn relocate(&mut self) -> Result<()> {
        //来自不同文件的段的地址
        let mut section_address = IndexMap::new();
        for (key, section) in &self.sections {
            section_address.insert(*key, self.section_address[&section.kind] + section.offset);
        }

        for ((path, name, index), section) in self.sections.iter_mut() {
            let file = &self.inputs[path];

            for (offset, relocation) in &section.relocation {
                let s = match &relocation.target() {
                    RelocationTarget::Absolute => 0,
                    RelocationTarget::Section(index) => {
                        section_address[&(*path, file.section_by_index(*index)?.name()?, *index)]
                    }
                    RelocationTarget::Symbol(index) => {
                        let name = file.symbol_by_index(*index)?.name()?;
                        let symbol = if let Some(t) = self.local_symbols.get(*path)
                            && let Some(t) = t.get(&(name, *index))
                        {
                            t
                        } else if let Some(t) = self.global_symbols.get(name) {
                            t
                        } else {
                            return Err(anyhow!("undefined symbol: {name}"));
                        };
                        let (path, file) = self.inputs.get_index(symbol.file_index).unwrap();
                        section_address[&(
                            *path,
                            file.section_by_index(symbol.section_index)?.name()?,
                            symbol.section_index,
                        )] + symbol.address
                    }
                    _ => todo!(),
                };
                let a = relocation.addend() as u64;
                let p = section_address.get(&(*path, *name, *index)).unwrap() + *offset;

                let value = match relocation.kind() {
                    RelocationKind::Absolute => s + a,
                    RelocationKind::Relative => s + a - p,
                    RelocationKind::ImageOffset => s + a - self.image_base,
                    _ => todo!(),
                };

                let size = relocation.size().div_ceil(8) as usize;
                let range = (*offset as usize)..(*offset as usize + size);
                match &relocation.encoding() {
                    RelocationEncoding::Generic => {
                        section.data[range].clone_from_slice(&value.to_le_bytes()[..size]);
                    }
                    _ => todo!(),
                }
            }
        }

        Ok(())
    }

    pub fn write(&self) -> Result<()> {
        //TODO 根据实际情况
        let nt_headers = NtHeaders {
            machine: 0x8664,
            time_date_stamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32,
            characteristics: IMAGE_FILE_EXECUTABLE_IMAGE,
            major_linker_version: env!("CARGO_PKG_VERSION_MAJOR").parse()?,
            minor_linker_version: env!("CARGO_PKG_VERSION_MINOR").parse()?,
            address_of_entry_point: self.entry_address as u32,
            image_base: self.image_base,
            major_operating_system_version: 4,
            minor_operating_system_version: 0,
            major_image_version: 0,
            minor_image_version: 0,
            major_subsystem_version: 5,
            minor_subsystem_version: 2,
            subsystem: IMAGE_SUBSYSTEM_WINDOWS_CUI,
            dll_characteristics: IMAGE_DLLCHARACTERISTICS_HIGH_ENTROPY_VA
                | IMAGE_DLLCHARACTERISTICS_DYNAMIC_BASE
                | IMAGE_DLLCHARACTERISTICS_NX_COMPAT,
            size_of_stack_reserve: 0x200000,
            size_of_stack_commit: 0x1000,
            size_of_heap_reserve: 0x100000,
            size_of_heap_commit: 0x1000,
        };

        let mut section_data: IndexMap<SectionKind, Vec<u8>> = IndexMap::new();
        for (_, section) in &self.sections {
            if section.data.len() == 0 {
                continue;
            }
            if let Some(t) = section_data.get_mut(&section.kind) {
                t.extend(section.data.clone());
            } else {
                section_data.insert(section.kind, section.data.clone());
            }
        }

        let mut buffer = vec![];
        //TODO 根据实际情况
        let mut writer = Writer::new(true, self.section_align as u32, 0x200, &mut buffer);

        writer.reserve_dos_header_and_stub();

        writer.reserve_nt_headers(IMAGE_NUMBEROF_DIRECTORY_ENTRIES);
        for i in 0..IMAGE_NUMBEROF_DIRECTORY_ENTRIES {
            writer.set_data_directory(i, 0, 0);
        }

        //还有一个idata段
        writer.reserve_section_headers(section_data.len() as u16 + 1);

        let mut section_offset = IndexMap::new();
        for (kind, data) in &section_data {
            let offset = match kind {
                SectionKind::Text => writer.reserve_text_section(data.len() as u32).file_offset,
                SectionKind::Data => {
                    writer
                        .reserve_data_section(data.len() as u32, data.len() as u32)
                        .file_offset
                }
                SectionKind::ReadOnlyData => {
                    writer.reserve_rdata_section(data.len() as u32).file_offset
                }
                SectionKind::UninitializedData => {
                    writer.reserve_bss_section(data.len() as u32).file_offset
                }
                _ => todo!(),
            };
            section_offset.insert(*kind, offset);
        }

        //TODO 根据实际情况
        let idata_offset = writer
            .reserve_idata_section(self.idata_section.len() as u32)
            .file_offset;

        writer.write_dos_header_and_stub()?;

        writer.write_nt_headers(nt_headers);

        writer.write_section_headers();

        for (kind, offset) in section_offset {
            writer.write_section(offset, &section_data[&kind]);
        }

        writer.write_section(idata_offset, &self.idata_section);

        fs::write(self.output.clone(), buffer)?;

        Ok(())
    }
}
