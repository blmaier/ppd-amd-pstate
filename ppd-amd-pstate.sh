#!/bin/bash
# This script monitors for PPD changes on DBus, and anytime a change is
# detected it reconfigures the EPP to match what PPD would normally set EPP to.

set -euo pipefail

# Set AMD PState EPP mode, controls how CPU's automatic scaler operates
set_epp_mode() {
	local mode=$1

	for epp in /sys/devices/system/cpu/cpu*/cpufreq/energy_performance_preference; do
		if [ "$(cat "$epp")" != "$mode" ]; then
			echo "$mode" >"$epp"
		fi
	done
}

set_gov() {
	local gov=$1

	if [ "$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor)" != "$gov" ]; then
		cpupower frequency-set -g "$gov"
		if [ "$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor)" != "$gov" ]; then
			echo "Error: Failed to switch CPU scaling governor to $gov" >&2
			exit 1
		fi
	fi
}

set_profile() {
	local profile=$1

	# Profile to EPP conversion based on power-profiles-daemon implementation
	# https://gitlab.gnome.org/Infrastructure/Mirrors/lorry-mirrors/gitlab_freedesktop_org/hadess/power-profiles-daemon/-/blob/main/src/ppd-driver-amd-pstate.c#L127

	gov=powersave

	case "$profile" in
	power-saver)
		epp_mode=power
		;;
	balanced)
		epp_mode=balance_performance
		;;
	performance)
		epp_mode=performance
		;;
	*)
		echo "Unkown profile '$profile'" >&2
		exit 1
	esac

	set_gov "$gov"
	set_epp_mode "$epp_mode"
}

update_profile() {
	local profile

	profile=$(powerprofilesctl get)

	echo "Updating amd-pstate-epp to $profile"

	set_profile "$profile"
}

is_pstate() {
	# Check amd_pstate_epp driver is in use
	if [ "$amd_pstate_status" != active ]; then
		echo "Error: AMD pstate not in 'active' mode" >&2
		echo "Error: check if cmdline contains 'amd_pstate=active'" >&2
		exit 1
	fi

	if [ "$scaling_driver" != amd-pstate-epp ]; then
		echo "Error: system not using CPU scaling driver amd-pstate-epp" >&2
		echo "Error: check if cmdline contains 'amd_pstate=active'" >&2
		exit 1
	fi
}

amd_pstate_status=$(cat /sys/devices/system/cpu/amd_pstate/status || true)
scaling_driver=$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_driver || true)
epp_available=$(cat /sys/devices/system/cpu/cpu0/cpufreq/energy_performance_available_preferences || true)
scaling_governor=$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor || true)
epp_used=$(cat /sys/devices/system/cpu/cpu0/cpufreq/energy_performance_preference || true)

op=${1:-info}

if ! command -v powerprofilesctl >/dev/null; then
	echo "This tool requires powerprofilesctl is installed" >&2
	exit 1
fi

if [ "$op" = info ]; then
	cat <<-EOF
	amd pstate status: $amd_pstate_status
	cpu scaling driver: $scaling_driver
	cpu scaling governor: $scaling_governor
	epp: $epp_used
	epp available: $epp_available
	platform profile: $(powerprofilesctl get)
	EOF
	exit 0
elif [ "$op" = monitor ]; then
	is_pstate

	update_profile

	# Monitor for the DBus event that triggers power-profiles-daemon to switch power mode
	while read -r _; do
		update_profile
	done < <(dbus-monitor --system --profile 'path=/net/hadess/PowerProfiles,member=Set')

	# If we get here dbus-monitor crashed
	exit 1
else
	echo "Unknown op '$op' (info, monitor)" >&2
	exit 1
fi
