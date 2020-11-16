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
fake kernel. All it does is say hello by scribbling on the vga memory.
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

Traditionally, Linux kernels load at 0x100000, which is the start of
[extended memory](https://tinyurl.com/y3rx26oq) on PCs. I'm not sure
what you do on arm systems yet.

That's done. Next step is change the loader to do a hello world and run it. 

I forget how to build this thing... Checking the targets

    $ rustc --print target-list | grep efi
    i686-unknown-uefi
    x86_64-unknown-uefi


In the loader directory do `cargo build -Z build-std --target=x86_64-unknown-uefi` to build uefi-rs/target/loader.efi

It looks like there is a json machine description file for
aarch64-unknown-uefi targets. We'll play with that later.

The qemu command from the original build script appears to be:

```sh
qemu-system-x86_64 -nodefaults -machine q35 -smp 3 -m 128M --enable-kvm -drive if=pflash,format=raw,file=/usr/share/ovmf/x64/OVMF_CODE.fd,readonly=on -drive if=pflash,format=raw,file=/usr/share/ovmf/x64/OVMF_VARS.fd,readonly=on -drive format=raw,file=fat:rw:/home/evaitl/git/UGA/CSCI8965/uefi-rs/target/x86_64-unknown-uefi/debug/esp -serial stdio -qmp pipe:qemu-monitor -device isa-debug-exit,iobase=0xf4,iosize=0x04 -vga std
```

Yeesh... That's long. 

One way to go woule be to make sure loader.efi is added under the esp
directory and add a startup.nsh in esp with:

    fs0
    loader.efi

as the contents?

Let's start trimming down loader to a "hello world", make sure it
runs, then add in the stuff we need...

Says hi and reboots OK, at least under qemu. Resetting the console
clears any previous output, so we need to add a stall if you want to
see things.  Tagging this as "hi 1" and pushing. 

Next, I'll look at loading the elf file into memory. 
