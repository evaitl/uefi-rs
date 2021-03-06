## Wed Nov 18:

Today, I need to create a signing tool, sign the kernel image, then
add code to the loader to check the signature.

Created new git [repo](https://github.com/evaitl/ksigner) for the
signer.

Signer works. Ran into a problem adding the signature verifier to the loader:

    Compiling sha2 v0.9.2
    Compiling uefi-services v0.3.0 (/home/evaitl/git/UGA/CSCI8965/project/uefi-rs/uefi-services)
    LLVM ERROR: Do not know how to split the result of this operator!

    error: could not compile `sha2`

Looking into it....

Looks like the C code is broken for sha2. Needed to turn on the "asm" feature:


    [dependencies.sha2]
    default-features=false
    features=["asm"]


When you "optimize" your code so hard it breaks the compiler, you are
probably doing it wrong.

Shit. I can compile "asm", but I can't link. I think it pulls in libc
for some reason, which we don't have.

Found something that works: Turn off the default features for a couple
packages so they don't pull in std and turn on a "force-soft" feature
for sha2. I have a build again. In the loader Cargo file we need:

    [dependencies.ed25519-dalek]
    version="^1.0.1"
    default-features=false
    features=["u64_backend"]
    [dependencies.digest]
    version="^0.9.0"
    default-features=false
    features=[]
    [dependencies.sha2]
    version="0.9.2"
    default-features=false
    features=["force-soft"]



Build/test instructions:

- Checkout the [kernel](https://github.com/evaitl/x86_min_kernel) and
  build it. You should get a file called `min_kernel`, which is the
  target binary.
  
- Checkout the [signing](https://github.com/evaitl/ksigner)
  tool. Build it, which gets you a desktop binary called
  `ksigner`. The keypair for the signing tool is in the ksigner repo
  and is called `keypair.json`
  
- Sign the kernel. Copy it somewhere as `kernel` and sign it with
  `./ksigner -l keypair.json -S kernel`. This will tack a signature
  onto the end of the file and increse its size by 64 bytes. 
  
- Checkout our [uefi-rs](https://github.com/evaitl/uefi-rs.git) tree.
  Go to the loader subdirectory and type `./build.py build` to create
  the loader. 
  
- Copy your signed kernel to the
  `uefi-rs/target/x86_64-unknown/uefi/debug/esp` directory.
  
- Now type `./build.py run` in the loader directory. 

The loader will say a few things on the qemu screen, which are also
echoed to the console. The loader then jumps to the kernel, which
spews "Hello world from the kernel" to the seriall port. QEMU echos
this to the console.




## Tue Nov 17:


Sadly the uefi Result type is private, so I can't create a function
that returns a result and deal with arbitrary code that mixes uefi
Results with Option and other results.

I'll just hack through a `load_kernel()` function for now. It should
be factored and the error handling cleaned up.


OK. Loader is written, but not tested. I'm using
[elf_rs](https://crates.io/crates/elf_rs) for the elf parsing.


I'll take a break, then test the loader. 

I think I'll grab the
[ed25519_dalek](https://docs.rs/ed25519-dalek/1.0.1/ed25519_dalek/)
package and create a signing tool, then add the code to the loader to
check the signature.


None of this really matters though. We just have to do some slides and
we have more than enough to say for slides.


Tested the loader, it didn't appear to work. Chased it down. It turns
out that the old "scribble on vga buffer" doesn't work any more.  I
changed the kernel to scribble on the serial port instead. Everything works. 

I'll add signature checking tomorrow. Gotta go play AC Vahalla....


## Mon Nov 16:

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

[Here](https://github.com/rpjohnst/kernel/blob/5e95b48d6e12b4cb03aa3c770160652a221ff085/src/boot.c)
is a C version that is kind of what we need. It loads an elf binary
into memory and runs it. It doesn't however do signature checking.

I think I'll copy/convert this code for now and add the signature
checking later.

I deleted the uefi-loader repo because I think it will confuse people.
All work is in this repo (and the min-kernel repos).

Looking at the email I sent the other day, the rust libraries I saw for signatures are

- [signature](https://crates.io/crates/signature)
- [digest](https://lib.rs/crates/digest)
- [minisign](https://lib.rs/crates/minisign)

And for elf headers:

- [elf_rs](https://lib.rs/crates/elf_rs)
- [xmas-elf](https://crates.io/crates/xmas-elf)
- [goblin](https://crates.io/crates/xmas-elf)

Building docs command: `cargo doc -Zbuild-std --target=x86_64-unknown-uefi --open`

End of the day checkin. Fussing with stuff. 





