use std::fs;
use std::io;
use std::error::Error;

fn sysfs_read(path: &str) -> io::Result<String> {
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

pub fn cpux_scaling_driver(cpu: usize) -> io::Result<String> {
    sysfs_read!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_driver", cpu)
}

pub fn cpux_scaling_driver_is_epp(cpu: usize) -> bool {
    match cpux_scaling_driver(cpu) {
        Ok(s) => s == "amd-pstate-epp",
        Err(_) => false,
    }
}

pub fn cpu_parse_range(cpu_string: &str) -> Result<Vec<usize>, Box<dyn Error>> {
    use std::ops::RangeInclusive;

    let groups = cpu_string.split(",");

    let range_maps = groups.map(|group| {
        let mut range = group.split("-");

        let left = range.next().ok_or("?")?.parse::<usize>()?;
        let right = match range.next() {
            Some(x) => x.parse::<usize>()?,
            None => left,
        };
        Ok(left..=right)
    });
    let ranges: Result<Vec<RangeInclusive<usize>>, Box<dyn Error>> = range_maps.collect();
    Ok(ranges?.into_iter().flatten().collect())
}

pub fn cpu_possible() -> Result<Vec<usize>, Box<dyn Error>> {
    let possible = sysfs_read("/sys/devices/system/cpu/possible")?;
    cpu_parse_range(possible.as_str())
}

pub fn cpux_scaling_governor_active(cpu: usize) -> io::Result<String> {
    sysfs_read!(
        "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor",
        cpu
    )
}

pub fn cpux_scaling_governor_avail(cpu: usize) -> io::Result<Vec<String>> {
    let avail = sysfs_read!(
        "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_available_governors",
        cpu
    )?;
    let avail = avail.split_whitespace();
    Ok(avail.map(|x| String::from(x)).collect())
}

pub fn cpux_epp_active(cpu: usize) -> io::Result<String> {
    sysfs_read!(
        "/sys/devices/system/cpu/cpu{}/cpufreq/energy_performance_preference",
        cpu
    )
}

pub fn cpux_epp_avail(cpu: usize) -> io::Result<Vec<String>> {
    let avail = sysfs_read!(
        "/sys/devices/system/cpu/cpu{}/cpufreq/energy_performance_available_preferences",
        cpu
    )?;
    let avail = avail.split_whitespace();
    Ok(avail.map(|x| String::from(x)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_parse_range() -> Result<(), Box<dyn Error>> {
        assert_eq!(cpu_parse_range("0-4")?, (0..=4).collect::<Vec<usize>>());
        assert_eq!(
            cpu_parse_range("0-4,6")?,
            (0..=4).chain(6..=6).collect::<Vec<usize>>()
        );
        assert_eq!(
            cpu_parse_range("0-4,6-8")?,
            (0..=4).chain(6..=8).collect::<Vec<usize>>()
        );
        assert_eq!(cpu_parse_range("1-4")?, (1..=4).collect::<Vec<usize>>());
        assert_eq!(
            cpu_parse_range("1-1000")?,
            (1..=1000).collect::<Vec<usize>>()
        );
        assert_ne!(cpu_parse_range("0-3")?, (0..=4).collect::<Vec<usize>>());
        Ok(())
    }
}
