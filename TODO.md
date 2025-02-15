# Highest Priority
- Use `GRUB` over the `bootloader` crate, or update the bootloader crate. *(note: worked on in this branch)*

# Medium Priority
- Fix bug where clearing the screen too much breaks it
    - Note: It seems to only be when doing it *fast*. If you do it *slower* then you probably won't cause the crash.
- Fix the dots
- Other (better) filesystems

# Low Priority
- Fix AHCI Reading and Writing
- Add SATA & USB Mass Storage support
- Add a GUI
    - Libs for external programs to interface with
    - Mouse support
- Networking (not looking forward to this...)
- Audio
