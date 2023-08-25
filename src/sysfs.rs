use std::error::Error;
use std::fs;

fn sysfs_read(path: &str) -> Result<String, Box<dyn Error>> {
    Ok(String::from(fs::read_to_string(path)?.trim()))
}

macro_rules! sysfs_read {
    ($($tts:tt)*) => {
        sysfs_read(format!($($tts)*).as_str())
    }
}

pub fn amd_pstate_is_active() -> bool {
    match sysfs_read("/sys/devices/system/cpu/amd_pstate/status") {
        Ok(s) => s == "active",
        Err(_) => false,
    }
}

pub fn cpux_scaling_driver(cpu: usize) -> String {
    sysfs_read!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_driver", cpu)
        .expect("Failed to read active scaling driver")
}

pub fn cpux_scaling_driver_is_epp(cpu: usize) -> bool {
    cpux_scaling_driver(cpu) == "amd-pstate-epp"
}

pub fn cpu_parse_range(cpu_string: &str) -> Vec<usize> {
    let groups = cpu_string.split(",");

    groups
        .map(|group| {
            let mut range = group.split("-");

            let left = range
                .next()
                .expect("CPU possible contains invalid range")
                .parse::<usize>()
                .expect("CPU possible contains invalid left range");
            let right = match range.next() {
                Some(x) => x
                    .parse::<usize>()
                    .expect("CPU possible contains invalid right range"),
                None => left,
            };
            left..=right
        })
        .flatten()
        .collect()
}

pub fn cpu_possible() -> Vec<usize> {
    let possible =
        sysfs_read("/sys/devices/system/cpu/possible").expect("Failed to read CPU present");
    cpu_parse_range(possible.as_str())
}

pub fn cpux_scaling_governor_active(cpu: usize) -> String {
    sysfs_read!(
        "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor",
        cpu
    )
    .expect("Failed to read active scaling governor")
}

pub fn cpux_scaling_governor_avail(cpu: usize) -> Vec<String> {
    let avail = sysfs_read!(
        "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_available_governors",
        cpu
    )
    .expect("Failed to read available scaling governors");
    let avail = avail.split_whitespace();
    avail.map(|x| String::from(x)).collect()
}

pub fn cpux_epp_active(cpu: usize) -> String {
    sysfs_read!(
        "/sys/devices/system/cpu/cpu{}/cpufreq/energy_performance_preference",
        cpu
    )
    .expect("Failed to read EPP active")
}

pub fn cpux_epp_avail(cpu: usize) -> Vec<String> {
    let avail = sysfs_read!(
        "/sys/devices/system/cpu/cpu{}/cpufreq/energy_performance_available_preferences",
        cpu
    )
    .expect("Failed to read EPP available");
    let avail = avail.split_whitespace();
    avail.map(|x| String::from(x)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_parse_range() {
        assert_eq!(cpu_parse_range("0-4"), (0..=4).collect::<Vec<usize>>());
        assert_eq!(
            cpu_parse_range("0-4,6"),
            (0..=4).chain(6..=6).collect::<Vec<usize>>()
        );
        assert_eq!(
            cpu_parse_range("0-4,6-8"),
            (0..=4).chain(6..=8).collect::<Vec<usize>>()
        );
        assert_eq!(cpu_parse_range("1-4"), (1..=4).collect::<Vec<usize>>());
        assert_eq!(
            cpu_parse_range("1-1000"),
            (1..=1000).collect::<Vec<usize>>()
        );
        assert_ne!(cpu_parse_range("0-3"), (0..=4).collect::<Vec<usize>>());
    }
}
