# About

This tool is for use with the Linux amd\_pstate\_epp driver. It works with
power-profiles-daemon to automatically manage the amd-pstate Energy Performance
Preference (EPP) settings.

The power-profiles-daemon already supports amd\_pstate\_epp, but it is
[disabled if the system also supports platform\_profile](https://gitlab.freedesktop.org/hadess/power-profiles-daemon/-/issues/124).
ppd-amd-pstate fills this gap by managing amd\_pstate\_epp while
power-profiles-daemon manages platform\_profile.

You can check if your system has platform\_profile by running

```
cat /sys/firmware/acpi/platform_profile
```

For info about CPU power management see
[Arch Linux CPU Frequency Scaling](https://wiki.archlinux.org/title/CPU_frequency_scaling).

# Installation

[Enable amd\_pstate\_epp](https://wiki.archlinux.org/title/CPU_frequency_scaling#amd_pstate)
by setting the kernel parameter "amd\_pstate=active".

[Install power-profiles-daemon](https://wiki.archlinux.org/title/CPU_frequency_scaling#power-profiles-daemon)
and enable the systemd service.

Git clone this repo and run the install.sh script with sudo

```
git clone https://github.com/blmaier/ppd-amd-pstate.git && cd ppd-amd-pstate && sudo ./install.sh
```
