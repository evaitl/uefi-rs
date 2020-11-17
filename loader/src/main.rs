#![no_std]
#![no_main]
#![feature(asm)]
#![feature(abi_efiapi)]

#![allow(unused_imports)] // XXX remove later

#[macro_use]
extern crate log;
#[macro_use]
extern crate alloc;

// Keep this line to ensure the `mem*` functions are linked in.
extern crate rlibc;

use core::mem;
use core::slice;
use uefi::prelude::*;
use uefi::table::boot::MemoryDescriptor;

fn locate_elf(bt: &BootServices, buf: &[u8]) -> Option<u64>{
    use elf_rs::*;
    if let Elf::Elf64(e) = Elf::from_bytes(buf).unwrap() {
        for p in e.program_header_iter() {
            let ph=p.ph;
            if ph.ph_type()==ProgramType::LOAD {
                let offset=ph.offset() as usize;
                let paddr=ph.paddr() as usize;
                let fsize=ph.filesz() as usize;
                unsafe{bt.memmove(paddr as *mut u8, &buf[offset], fsize)};
            }
        }
        return Some(e.header().entry_point());
    }
    None
}

fn load_kernel(image: Handle, st: & SystemTable<Boot>) {
    let bt = st.boot_services();
    use uefi::proto::loaded_image::LoadedImage;
    use uefi::proto::media::file::File;
    use uefi::proto::media::file::FileAttribute;
    use uefi::proto::media::file::FileMode;
    use uefi::proto::media::fs::SimpleFileSystem;
    use uefi::table::boot::AllocateType;
    use uefi::table::boot::MemoryType;
    use uefi::proto::media::file::RegularFile;
    use uefi::proto::media::file::FileInfo;
    let sfs=bt.locate_protocol::<SimpleFileSystem>().expect("sfs failure").unwrap();
    let sfs=unsafe {&mut *sfs.get() };
    let mut directory=sfs.open_volume().unwrap().unwrap();
    let kernel_file=directory.open("kernel",
                                   FileMode::Read,FileAttribute::empty()).expect("Open failure").unwrap();
    let mut kernel_file=unsafe{RegularFile::new(kernel_file)};
    
    let mut info_buffer=vec![0;256];
    let file_size=
        kernel_file.get_info::<FileInfo>(&mut info_buffer).expect("File info problem").unwrap().file_size();
    
    drop(info_buffer);
    // Reserve location for final kernel so it doesn't get used by loaded file.
    if cfg!(target_arch = "x86_64") {
        // On x86 save 4M (1024 pages) at 0x100000.
        bt.allocate_pages(
            AllocateType::Address(0x100000),
            MemoryType::LOADER_DATA,
            1024,
        );
        unsafe{bt.memset(0x100000 as *mut u8, 4*1024*1024,0)};
    } else if cfg!(target_arch = "aarch64") {
        // XXX
    }

    let mut image_buf=vec![0u8;file_size as usize];
    // Read the file

    let read_size=kernel_file.read(image_buf.get_mut(..).unwrap()).expect("Read error").unwrap() as u64;
    assert!(read_size == file_size);

    // Check the signature

    // XXX
    
    // Move Loadable segments
    // x86_64: Make sure they are in the 4M region at 0x100000?

    locate_elf(&bt,&image_buf);
    
    // Get memory map

    // Get acpi tables

    // Jump to start address

    // Shouldn't get here.
    error!("How'd I get here?");
}

#[entry]
fn efi_main(image: Handle, st: SystemTable<Boot>) -> Status {
    // Initialize utilities (logging, memory allocation...)
    uefi_services::init(&st).expect_success("Failed to initialize utilities");

    // Reset the console before running all the other tests.
    st.stdout()
        .reset(false)
        .expect_success("Failed to reset stdout");

    // Ensure the tests are run on a version of UEFI we support.
    check_revision(st.uefi_revision());
    st.boot_services().stall(3_000_000);

    load_kernel(image, &st);
    
    st.boot_services().stall(3_000_000);

    shutdown(image, st);
}

fn check_revision(rev: uefi::table::Revision) {
    let (major, minor) = (rev.major(), rev.minor());

    info!("UEFI {}.{}", major, minor / 10);

    assert!(major >= 2, "Running on an old, unsupported version of UEFI");
    assert!(
        minor >= 30,
        "Old version of UEFI 2, some features might not be available."
    );
}

fn shutdown(image: uefi::Handle, st: SystemTable<Boot>) -> ! {
    use uefi::table::runtime::ResetType;

    // Get our text output back.
    st.stdout().reset(false).unwrap_success();

    info!("Testing complete, shutting down in 3 seconds...");
    st.boot_services().stall(3_000_000);

    // Exit boot services as a proof that it works :)
    let max_mmap_size =
        st.boot_services().memory_map_size() + 8 * mem::size_of::<MemoryDescriptor>();
    let mut mmap_storage = vec![0; max_mmap_size].into_boxed_slice();
    let (st, _iter) = st
        .exit_boot_services(image, &mut mmap_storage[..])
        .expect_success("Failed to exit boot services");

    // Shut down the system
    let rt = unsafe { st.runtime_services() };
    rt.reset(ResetType::Shutdown, Status::SUCCESS, None);
}

/*
Local Variables:
compile-command: "cargo build -Zbuild-std --target x86_64-unknown-uefi"
End:
*/
