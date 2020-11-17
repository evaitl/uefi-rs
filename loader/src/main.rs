#![no_std]
#![no_main]
#![feature(asm)]
#![feature(abi_efiapi)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate alloc;

// Keep this line to ensure the `mem*` functions are linked in.
extern crate rlibc;

use core::mem;
use uefi::prelude::*;
use uefi::table::boot::MemoryDescriptor;
use uefi::proto::media::file::FileHandle;
use uefi::Result;

fn load_kernel(image: Handle, bt: &BootServices) -> Result<FileHandle>{
    use uefi::proto::loaded_image::LoadedImage;
    use uefi::proto::media::file::FileMode;
    use uefi::proto::media::file::FileAttribute;
    use uefi::proto::media::fs::SimpleFileSystem;
    use uefi::proto::media::file::File;
    use uefi::table::boot::MemoryType;
    use uefi::table::boot::AllocateType;
    let li=bt.handle_protocol::<LoadedImage>(image)?.unwrap().get();
    let fs=bt.handle_protocol::<SimpleFileSystem>(li.as_ref()
                                                  .ok_or(Status::WARN_FILE_SYSTEM)?
                                                  .device())?.unwrap().get();
    let kernel_file_handle=
        fs.as_ref().ok_or(Status::WARN_FILE_SYSTEM)?.open_volume()?.unwrap()
        .open("kernel",FileMode::Read,FileAttribute::READ_ONLY);
    // Reserve location for final kernel so it doesn't get used by loaded file.
    if cfg!(target_arch="x86_64") {
        // On x86 save 4M (1024 pages) at 0x100000. 
        bt.allocate_pages(AllocateType::Address(0x100000),MemoryType::LOADER_DATA,1024);
    } else if cfg!(target_arch="aarch64") {
        // XXX
    }
    // Get the file size

    // Get space for the file
//    let file_data_ptr=bt.allocate_pool(AnyPages,XXX);
    // Read the file

    // Check the signature

    // Move Loadable segments
    // x86_64: Make sure they are in the 4M region at 0x100000?

    // Clear BSS

    // Get memory map

    // Get acpi tables

    // Jump to start address


    // Shouldn't get here.
    Status::LOAD_ERROR.into_with_err("deadbeaf")
}

#[entry]
fn efi_main(image: Handle, st: SystemTable<Boot>) -> Status {
    // Initialize utilities (logging, memory allocation...)
    uefi_services::init(&st).expect_success("Failed to initialize utilities");
    let bt=st.boot_services();
    // Reset the console before running all the other tests.
    st.stdout()
        .reset(false)
        .expect_success("Failed to reset stdout");

    // Ensure the tests are run on a version of UEFI we support.
    check_revision(st.uefi_revision());
    st.boot_services().stall(3_000_000);
    
    load_kernel(image,bt);
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
