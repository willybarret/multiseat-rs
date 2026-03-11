# Multiseat Manager

A blazingly fast, seamless multiseat manager for Linux, written in Rust using GTK4, libadwaita, logind, udev, and evdev.

Tested on Fedora Workstation 43 (Wayland GNOME 49.2) with `gdm` and `gnome-shell` packages patched.

![](images/gui.png)

## Installation

### Using Cargo

```bash
cargo install multiseat-rs
```

### From Source

```bash
git clone https://github.com/willybarret/multiseat-rs.git
cd multiseat-rs
cargo install --path .
```

## What is Multiseat?

Multiseat turns one computer into multiple independent workstations. Each "seat" gets its own GPU, monitor, keyboard, and mouse, allowing multiple users to work simultaneously on a single machine with their own login sessions and protected files.

 **Why bother?**
 - GPUs are cheaper than whole computers
 - One box takes less space and makes less noise
 - Only one system to update, which means you won't need to sell a kidney for RAM upgrades (well, maybe just one lol)

 **Requirements:**
 - One GPU per seat (discrete cards, integrated graphics, or USB video adapters all work)
 - USB keyboards, mice, and gamepads for each seat
 - A display manager with multiseat support (GDM, LightDM)

For a deeper dive, you can see [Debian Multiseat Wiki](https://wiki.debian.org/Multi_Seat_Debian_HOWTO) and [Systemd Multiseat Wiki](https://www.freedesktop.org/wiki/Software/systemd/multiseat/).

![](images/use-case.jpg)

## ⚠️ Important Notes

### After Changing Device Assignments

Device reassignments are persistent but require a **restart** or **re-plug** to take effect.

### GPU-First Rule

When creating a new seat, **always assign a GPU first**. A seat without a GPU cannot display anything, so input devices assigned to a GPU-less seat become orphaned and unusable. The app will warn you about this.

### Single GPU Setups

I've been doing some research and found out that it's possible to do "multiseating" on Wayland using DRM leases of a single KMS device. Unfortunately, it's only a workaround that works on wlroots-based Compositors using `drm-lease-manager` under the hood and it still requires patching some packages to make everything work properly.

Perhaps adding some kind of logical DRM implementation on the Kernel to split a physical DRM device should do it? (This is easier said than done tho). This would allow a more Compositor-agnostic solution, but you could also argue that the Compositor should be the one that allows using DRM Leases for different seats.

For example, in the case of Mutter (GNOME's Compositor), it owns the DRM Master, just like `drm-lease-manager` does (the workaround I've mentioned earlier). Could be the answer, or not.

Anyways, I'm going to drink some [Tereré](https://en.wikipedia.org/wiki/Terer%C3%A9) before continuing.

## Development

### Dependencies

- GTK4 and libadwaita development libraries
- libudev (part of systemd)

On Fedora:
```bash
sudo dnf install gtk4-devel libadwaita-devel systemd-devel
```

On Arch Linux:
```bash
sudo pacman -S gtk4 libadwaita
```

### Running the App


```bash
git clone https://github.com/willybarret/multiseat-rs.git
cd multiseat-rs
cargo run
```

## Environment Setup (Patching GNOME)

Up to the present time, to enable multiseat support in GNOME, you must patch `gdm` and `gnome-shell`.
The required files are located in the `patches/` directory of this repository.

When this step becomes unnecessary (follow [(gnome-shell) Multiseat enablement for Wayland](https://gitlab.gnome.org/GNOME/gnome-shell/-/merge_requests/2230) if interested), these steps will be updated.

### Arch Linux (AUR)

You can find the patched packages in the AUR:

```bash
paru -S gdm-multiseat gnome-shell-multiseat
# or: yay -S gdm-multiseat gnome-shell-multiseat
```

### Fedora (Manual Patching)

#### 1. Patching GDM

```bash
mkdir -p patching-directory/gdm && cd patching-directory/gdm
dnf download gdm --source

# Extract the source rpm
rpm2cpio *.src.rpm | cpio -idmv

# Install dev dependencies
sudo dnf builddep gdm.spec

# Add your patches and update the version
# 1. Change 'Release' to something like .77 in gdm.spec
# 2. Add Patch line to gdm.spec
vi gdm.spec

# Build for Fedora 43
fedpkg --release f43 local

# Packages will be in ./x86_64/<package-name>.rpm
# Install the patched package
sudo dnf install ./x86_64/<package-name>.rpm
```

#### 2. Patching gnome-shell

```bash
mkdir -p patching-directory/gnome-shell && cd patching-directory/gnome-shell
dnf download gnome-shell --source

# Extract the source rpm
rpm2cpio *.src.rpm | cpio -idmv

# Install dev dependencies
sudo dnf builddep gnome-shell.spec

# Add your patches and update the version
# 1. Change 'Release' to something like .77 in gnome-shell.spec
# 2. Add Patch line to gnome-shell.spec
vi gnome-shell.spec

# Build for Fedora 43
fedpkg --release f43 local

# Packages will be in ./noarch/<package-name>.rpm and ./x86_64/<package-name>.rpm
# Install them with
sudo dnf install ./noarch/<package-name>.rpm ./x86_64/<package-name>.rpm
```

*Note: A system reboot will be required after installing these patched packages.*

Now you can start assigning devices to the seats you want with Multiseat Manager.

## License

This project is licensed under the GPL-3.0 License - see the [LICENSE.md](LICENSE.md) file for details.

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests.

---

Made with 🧉 and Rust

