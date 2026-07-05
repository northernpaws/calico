use object::{Object, ObjectSection, ObjectSymbol, Section, Symbol, SymbolKind};
use std::io::Read;
use std::str::Utf8Error;
use std::{fs::File, path::Path};

use calico_rpc::runner::{Test, TestOutcome};

/// Defines the structure of a test defined in the Calico ELF table.
#[derive(serde::Serialize, serde::Deserialize)] 
pub struct TestDefinition {
    disambiguator: u64,
    name: String,
    ignored: bool,
    should_panic: bool,
    timeout: Option<u32>,
}

impl From<TestDefinition> for Test {
    fn from(val: TestDefinition) -> Self {
        Test{
            name: val.name,
            expected_outcome: TestOutcome{},
            ignored: val.ignored,
            timeout: val.timeout,
            address: None
        }
    }
}

/// Struct holding test info extrated from
/// the INFO section of an ELF binary.
#[derive(Debug, PartialEq)]
pub struct TestElfInfo {
    /// Version of Calico that the ELF was compiled against.
    pub version: u32,
    /// Tests found in the elf.
    pub tests: Vec<Test>,
}

#[derive(Debug)]
pub enum ElfError {
    FileOpenError(std::io::Error),
    FileReadError(std::io::Error),
    ParseError(object::Error),
    ElfReadError(ElfReadError)
}

impl From<std::io::Error> for ElfError {
    fn from(value: std::io::Error) -> Self {
        ElfError::FileReadError(value)
    }
}

impl TestElfInfo {
    /// Accepts a path to an ELF file to parse.
    pub fn from_elf(path: &Path) -> Result<Self, ElfError> {
        // Attempt to open the file on disk.
        let mut file = File::open(path).map_err(|e| ElfError::FileOpenError(e))?;

        // Load the binary into an in-memory buffer.
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        let buffer = buffer.as_slice();

        let elf = match object::File::parse(buffer) {
            Ok(elf) => elf,
            Err(e) => {
                return Err(ElfError::ParseError(e));
            }
        };

        ElfReader { buffer, elf }
            .decode().map_err(|e|ElfError::ElfReadError(e))
    }
}

/// Parses a buffer containing an ELF binary into the required info.
struct ElfReader<'a> {
    buffer: &'a [u8],
    elf: object::File<'a>,
}

#[derive(Debug)]
pub enum ElfReadError {
    NoSymbols,
    NoCalicoSection,
    /// No CALICO_VERSION symbol in ELF
    NoVersionFound,
    /// Failed to read the Calico version symbol
    MalformedVersion,
    UnknownVersion(u32),
    /// Module path does not contain '::'
    ModulePathMissingColons,
    /// Section not found for mod path str {mod_path_ptr:x}
    ModSectionNotFound(u32),
    /// Indicates there where issues reading from the ELF.
    InvalidElf(object::Error),
    /// Indicates that we weren't able to parse the JSON data for a test case.
    InvalidTestcaseJSON(serde_json::Error),
    InvalidUTF8String(Utf8Error)
}

impl From<object::Error> for ElfReadError {
    fn from(value: object::Error) -> Self {
        ElfReadError::InvalidElf(value)
    }
}

impl From<serde_json::Error> for ElfReadError {
    fn from(value: serde_json::Error) -> Self {
        ElfReadError::InvalidTestcaseJSON(value)
    }
}

impl From<Utf8Error> for ElfReadError {
    fn from(value: Utf8Error) -> Self {
        ElfReadError::InvalidUTF8String(value)
    }
}

impl<'a> ElfReader<'a> {
    fn decode(&self) -> Result<TestElfInfo, ElfReadError> {
        if self.elf.symbols().next().is_none() {
            return Err(ElfReadError::NoSymbols);
        }

        // Find the .calico containing test case info
        let Some(calico_section) = self.elf.section_by_name(".calico") else {
            return Err(ElfReadError::NoCalicoSection);
        };

        // Try to read the Calic version the ELF was compiled against.
        //
        // This defines the protocol format used to communicate between the
        // test runner and the version of Calico compiled into the binary.
        let Some(version_sym) = self.elf.symbol_by_name("CALICO_VERSION") else {
            return Err(ElfReadError::NoVersionFound);
        };

        // Decode the version bytes into a u32.
        let Some(version) = calico_section
            .data_range(version_sym.address(), version_sym.size())
            .map_err(|_| ElfReadError::MalformedVersion)? else {
            return Err(ElfReadError::MalformedVersion);
        };
        
        let version = u32::from_le_bytes(version.try_into().map_err(|_| ElfReadError::MalformedVersion)?);

        // Perform version-specific decoding.
        match version {
            0 => {
                // Read testcases from symbols
                let mut tests = vec![];

                // Loop over the symbols defined in the ELF,
                // and attempt to decode them as test cases.
                for sym in self.elf.symbols() {
                    println!("{:#?}", sym.name());
                    if let Some(sym) = self.try_decode_testcase_sym(&sym, &calico_section)? {
                        tests.push(sym);
                    }
                }

                Ok(TestElfInfo { version, tests })
            },

            // Return an error if the decoded version
            // isn't known to this version of the CLI.
            _ => Err(ElfReadError::UnknownVersion(version)),
        }        
    }

    /// Attempts to decode a symbol read from the ELF as a test case.
    /// 
    /// A testcase is stored as tuple of testfunc + module_path
    /// and has type `(fn()->!, &'static str)` which is 12 bytes.
    /// The symbol name is a escaped json object containing info about the test
    fn try_decode_testcase_sym(
        &self,
        sym: &Symbol<'_, '_>,
        calico_section: &Section<'_, '_>,
    ) -> Result<Option<Test>, ElfReadError> {
        const TESTCASE_SYM_SIZE: u64 = 12;

        // Check if the symbol meets the first-pass
        // criteria for a test case symbol.
        if !sym.is_global()
            // needs to be data
            || sym.kind() != SymbolKind::Data 
            // needs to be in the calico section
            || sym.section_index() != Some(calico_section.index())
            || sym.size() != TESTCASE_SYM_SIZE // sizeof( (fn()->!, &'static str) )
        {
            return Ok(None);
        }

        // Extract the symbol data from the ELF file.
        let sym_data = calico_section
            .data_range(sym.address(), sym.size())?
            .unwrap();

        // Unwrap is okay, this function is only called when the symbol size is known to be 12 bytes.
        let test_fn_ptr = u32::from_le_bytes(sym_data[0..4].try_into().unwrap());
        let mod_path_ptr = u32::from_le_bytes(sym_data[4..8].try_into().unwrap());
        let mod_path_len = u32::from_le_bytes(sym_data[8..12].try_into().unwrap());

        let mod_path = self.read_mod_path(mod_path_ptr, mod_path_len)?;
        let sym_name = sym.name()?;
        let def: TestDefinition = serde_json::from_str(sym_name)?;

        let mut test: Test = def.into();
        test.name = format!("{mod_path}::{}", test.name); // prepend mod path to test name
        test.address = Some(test_fn_ptr);
        Ok(Some(test))
    }

     #[inline]
    fn file_offset_for(&self, addr: u64, section: &Section<'_, '_>) -> usize {
        let (start, _end) = section.file_range().unwrap();
        let offset = addr - section.address();
        (start + offset) as usize
    }

    fn read_mod_path(&self, mod_path_ptr: u32, mod_path_len: u32) -> Result<&'a str, ElfReadError> {
        let section = self
            .elf
            .sections()
            .find(|section| {
                mod_path_ptr as u64 >= section.address()
                    && mod_path_ptr as u64 + mod_path_len as u64
                        <= (section.address() + section.size())
            })
            .ok_or(ElfReadError::ModSectionNotFound(mod_path_ptr))?;

        let file_offset = self.file_offset_for(mod_path_ptr as u64, &section);
        let full_path = &self.buffer[file_offset..file_offset + mod_path_len as usize];
        let full_path = str::from_utf8(full_path)?;
        let first_col = full_path
            .find("::")
            .ok_or(ElfReadError::ModulePathMissingColons)?;
        Ok(&full_path[first_col + 2..]) // strip the crate name from the module path
    }
}
