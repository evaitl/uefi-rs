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
use core::slice;
use uefi::prelude::*;
use uefi::proto::media::file::FileHandle;
use uefi::table::boot::MemoryDescriptor;

fn load_kernel(image: Handle, st: SystemTable<Boot>) -> ! {
    let bt = st.boot_services();
    use uefi::proto::loaded_image::LoadedImage;
    use uefi::proto::media::file::File;
    use uefi::proto::media::file::FileAttribute;
    use uefi::proto::media::file::FileMode;
    use uefi::proto::media::fs::SimpleFileSystem;
    use uefi::table::boot::AllocateType;
    use uefi::table::boot::MemoryType;

    let root_device = bt
        .handle_protocol::<LoadedImage>(image)
        .expect("No LoadedImage protocol")
        .expect("No LoadedImage Protocol 2")
        .get()
        .as_ref()
        .expect("No loadedimage 3")
        .device();
    let mut kernel_file = bt
        .handle_protocol::<SimpleFileSystem>(root_device)
        .expect("no sfs 1")
        .expect("no sfs 2")
        .get()
        .as_ref()
        .expect("no sfs 3")
        .open_volume()
        .expect("open volume failure")
        .expect("open volume failure 2");
    let fi_buf = unsafe {
        slice::from_raw_parts_mut(
            bt.allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, 1)
                .expect("Couldn't allocate info page")
                .expect("Couldn't allocate info page 2") as *mut u8,
            4096,
        )
    };
    use uefi::proto::media::file::FileInfo;

    kernel_file
        .get_info::<FileInfo>(fi_buf)
        .expect("Couldn't get file info");

    let kernel_file_handle = kernel_file.open("kernel", FileMode::Read, FileAttribute::READ_ONLY);
    // Reserve location for final kernel so it doesn't get used by loaded file.
    if cfg!(target_arch = "x86_64") {
        // On x86 save 4M (1024 pages) at 0x100000.
        bt.allocate_pages(
            AllocateType::Address(0x100000),
            MemoryType::LOADER_DATA,
            1024,
        );
    } else if cfg!(target_arch = "aarch64") {
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
    error!("How'd I get here?");
    shutdown(image, st);
}

#[entry]
fn efi_main(image: Handle, st: SystemTable<Boot>) -> Status {
    // Initialize utilities (logging, memory allocation...)
    uefi_services::init(&st).expect_success("Failed to initialize utilities");
    let bt = st.boot_services();
    // Reset the console before running all the other tests.
    st.stdout()
        .reset(false)
        .expect_success("Failed to reset stdout");

    // Ensure the tests are run on a version of UEFI we support.
    check_revision(st.uefi_revision());
    st.boot_services().stall(3_000_000);

    load_kernel(image, st);
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
