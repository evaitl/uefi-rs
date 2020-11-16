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


