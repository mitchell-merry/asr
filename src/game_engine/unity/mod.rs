//! Support for games using the Unity engine.
//!
//! # Example
//!
//! ```no_run
//! # async fn example(process: asr::Process) {
//! use asr::{
//!     future::retry,
//!     game_engine::unity::il2cpp::{Module, Version},
//!     Address, Address64,
//! };
//!
//! // We first attach to the Mono module. Here we know that the game is using IL2CPP 2020.
//! let module = Module::wait_attach(&process, Version::V2020).await;
//! // We access the .NET DLL that the game code is in.
//! let image = module.wait_get_default_image(&process).await;
//!
//! // We access a class called "Timer" in that DLL.
//! let timer_class = image.wait_get_class(&process, &module, "Timer").await;
//! // We access a static field called "_instance" representing the singleton
//! // instance of the class.
//! let instance = timer_class.wait_get_static_instance(&process, &module, "_instance").await;
//!
//! // Once we have the address of the instance, we want to access one of its
//! // fields, so we get the offset of the "currentTime" field.
//! let current_time_offset = timer_class.wait_get_field_offset(&process, &module, "currentTime").await;
//!
//! // Now we can add it to the address of the instance and read the current time.
//! if let Ok(current_time) = process.read::<f32>(instance + current_time_offset) {
//!    // Use the current time.
//! }
//! # }
//! ```
//! Alternatively you can use the `Class` derive macro to generate the bindings
//! for you. This allows reading the contents of an instance of the class
//! described by the struct from a process. Each field must match the name of
//! the field in the class exactly (or alternatively renamed with the `#[rename
//! = "..."]` attribute) and needs to be of a type that can be read from a
//! process. Fields can be marked as static with the `#[static_field]`
//! attribute.
//!
//! ```ignore
//! #[derive(Class)]
//! struct Timer {
//!     #[rename = "currentLevelTime"]
//!     level_time: f32,
//!     #[static_field]
//!     foo: bool,
//! }
//! ```
//!
//! This will bind to a .NET class of the following shape:
//!
//! ```csharp
//! class Timer
//! {
//!     float currentLevelTime;
//!     static bool foo;
//!     // ...
//! }
//! ```
//!
//! The class can then be bound to the process like so:
//!
//! ```ignore
//! let timer_class = Timer::bind(&process, &module, &image).await;
//! ```
//!
//! Once you have an instance, you can read the instance from the process like
//! so:
//!
//! ```ignore
//! if let Ok(timer) = timer_class.read(&process, timer_instance) {
//!     // Do something with the instance.
//! }
//! ```
//!
//! If only static fields are present, the `read` method does not take an
//! instance argument.

// References:
// https://github.com/just-ero/asl-help/tree/4c87822df0125b027d1af75e8e348c485817592d/src/Unity
// https://github.com/Unity-Technologies/mono
// https://github.com/CryZe/lunistice-auto-splitter/blob/b8c01031991783f7b41044099ee69edd54514dba/asr-dotnet/src/lib.rs

use crate::file_format::pe;
use crate::signature::Signature;
use crate::{Error, Process};

pub mod il2cpp;
pub mod mono;
pub mod scene_manager;

const CSTR: usize = 128;

#[derive(Copy, Clone, PartialEq, Hash, Debug)]
#[non_exhaustive]
enum BinaryFormat {
    PE,
    ELF,
    MachO,
}

/// If the field name is an auto-property, extract the backing field name.
fn get_backing_name(name: &str) -> Option<&str> {
    let start = name.find('<')?;
    let end = name[start + 1..].find('>')?;
    Some(&name[start + 1..start + 1 + end])
}

const ZERO: u8 = b'0';
const NINE: u8 = b'9';

pub fn get_unity_version(process: &Process) -> Result<(u32, u32), Error> {
    let (unity_module, binary_format) = [
        ("UnityPlayer.dll", BinaryFormat::PE),
        ("UnityPlayer.so", BinaryFormat::ELF),
        ("UnityPlayer.dylib", BinaryFormat::MachO),
        ("mono.dll", BinaryFormat::PE),
        ("libmono.so", BinaryFormat::ELF),
        ("libmono.0.dylib", BinaryFormat::MachO),
    ]
    .into_iter()
    .find_map(|(name, format)| match format {
        BinaryFormat::PE => {
            let address = process.get_module_address(name).ok()?;
            let range = pe::read_size_of_image(process, address)? as u64;
            Some(((address, range), BinaryFormat::PE))
        }
        format => Some((process.get_module_range(name).ok()?, format)),
    })
    .ok_or(Error {})?;

    // For Windows (PE):
    //   We can read Unity’s FileVersionInfo directly from the PE header and get the version there.
    if binary_format == BinaryFormat::PE {
        let file_version = pe::FileVersion::read(process, unity_module.0).ok_or(Error {})?;
        return Ok((
            file_version.major_version as _,
            file_version.minor_version as _,
        ));
    }

    // For ELF/MachO (Linux/macOS):
    //   No FileVersionInfo is available, so we fall back to scanning memory.
    //   Look for the ASCII signature "202?." or "6000.", which appears in Unity’s version string.
    // TODO: find the unity version programmatically
    const SIG_202X: Signature<6> = Signature::new("00 32 30 32 ?? 2E");
    const SIG_6000: Signature<6> = Signature::new("00 36 30 30 30 2E");

    let addr = SIG_202X
        .scan_process_range(process, unity_module)
        .or_else(|| SIG_6000.scan_process_range(process, unity_module))
        .ok_or(Error {})?;

    let version_string = process.read::<[u8; 6]>(addr + 1)?;

    let (before, after) = version_string.split_at(
        version_string
            .iter()
            .position(|&x| x == b'.')
            .ok_or(Error {})?,
    );

    let mut unity: u32 = 0;
    for &val in before {
        match val {
            ZERO..=NINE => unity = unity * 10 + (val - ZERO) as u32,
            _ => break,
        }
    }

    let mut unity_minor: u32 = 0;
    for &val in &after[1..] {
        match val {
            ZERO..=NINE => unity_minor = unity_minor * 10 + (val - ZERO) as u32,
            _ => break,
        }
    }

    Ok((unity, unity_minor))
}
