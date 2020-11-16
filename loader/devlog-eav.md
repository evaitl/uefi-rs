Mon Nov 16:

Looking for an arm64 ovmf firmware. I don't want to hassle with a
cross build of edk2. Found it
[here](https://launchpad.net/ubuntu/xenial/arm64/ovmf/0~20160408.ffea0a2c-2ubuntu0.1). Grab
the deb file, and pull OVMF.fd for my firmware.

May not need that. [Apparently](https://packages.debian.org/sid/qemu-efi-aarch64), uefi is already built into qemu-efi-aarch64. 

Some [instructions](https://www.linaro.org/blog/enabling-uefi-secure-boot-on-u-boot/) for setting up the keys. 

[Instructions](https://git.kernel.org/pub/scm/linux/kernel/git/jejb/efitools.git/about/)
for the package needed to create keys and sign efi binaries. We need
three packages: 

- openssl to create keys

- sbsigntools to sign our loader

- efitools to modify uefi variables (which are the signature databases)

[Instructions](https://wiki.archlinux.org/index.php/Unified_Extensible_Firmware_Interface/Secure_Boot)
for creating keys and such.


The loader should be the same for both aarch64 and x86_64
systems. Because it is easier, I'll just start with x86_64.  I'll
start at the end which is a fake kernel file that just says "Hello
world".  The fake kernel will be a stand-alone x86_64 elf file that
loads at 0x100000.  I'll have to do another one for the aarch64 board,
but offhand I don't know how to do IO on those yet.

Created a new [repo](https://github.com/evaitl/x86_min_kernel) for the
fake kernel. 
If/when I get to it, I'll do a separate repo for the aarch64 kernel. 

The "kernel" is mainly copied from post2 of Oppermann's
[blog](https://os.phil-opp.com/minimal-rust-kernel/). I don't want to
pull in his bootimage tool though. I'll just build a 64 bit binary and
have the loader be a 64-bit uefi loader.

Doing a simple linker script. Merging rodata and text because all
pages are executable (I think). Createing several loadable segments so
we test that out.

```Linker Script
ENTRY(_start);
SECTIONS {
         . = 0x100000;
         .text : { *(.text) *(.text.*)
                   *(.rodata) *(.rodata.*) }
         .data : { *(.data) }
         .bss : { *(.bss) }
         
}
```






