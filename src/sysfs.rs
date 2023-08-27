use std::collections::hash_set::HashSet;
use std::error::Error;
use std::fs;
use std::io;
use std::str::FromStr;
use strum::EnumCount;
use strum_macros::{Display, EnumCount, EnumString, IntoStaticStr};

fn sysfs_read(path: &str) -> io::Result<String> {
    Ok(String::from(fs::read_to_string(path)?.trim()))
}

fn sysfs_parse<T: FromStr>(path: &str) -> Result<T, Box<dyn Error>>
where
    <T as FromStr>::Err: Error + 'static,
{
    Ok(sysfs_read(path)?.parse::<T>()?)
}

fn sysfs_parse_set<T>(path: &str) -> Result<HashSet<T>, Box<dyn Error>>
where
    T: FromStr,
    T: Eq,
    T: std::hash::Hash,
    T: EnumCount,
    <T as FromStr>::Err: Error + 'static,
{
    let raws = sysfs_read(path)?;
    let mut set = HashSet::<T>::with_capacity(T::COUNT);
    for raw in raws.split_whitespace() {
        set.insert(raw.parse::<T>()?);
    }
    Ok(set)
}

#[allow(unused_macros)]
macro_rules! sysfs_read {
    ($($tts:tt)*) => {
        sysfs_read(format!($($tts)*).as_str())
    }
}

macro_rules! sysfs_parse {
    ($type:ident, $($tts:tt)*) => {
        sysfs_parse::<$type>(format!($($tts)*).as_str())
    }
}

macro_rules! sysfs_parse_set {
    ($type:ident, $($tts:tt)*) => {
        sysfs_parse_set::<$type>(format!($($tts)*).as_str())
    }
}

pub fn amd_pstate_is_active() -> bool {
    match sysfs_read("/sys/devices/system/cpu/amd_pstate/status") {
        Ok(s) => s == "active",
        Err(_) => false,
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Display, EnumString, IntoStaticStr, EnumCount,
)]
pub enum ScalingDriver {
    #[strum(serialize = "amd-pstate-epp")]
    AmdPstateEpp,
}

pub fn cpux_scaling_driver(cpu: usize) -> Result<ScalingDriver, Box<dyn Error>> {
    sysfs_parse!(
        ScalingDriver,
        "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_driver",
        cpu
    )
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

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Display, EnumString, IntoStaticStr, EnumCount,
)]
pub enum ScalingGovernor {
    #[strum(serialize = "powersave")]
    Powersave,
    #[strum(serialize = "performance")]
    Performance,
}

pub fn cpux_scaling_governor_active(cpu: usize) -> Result<ScalingGovernor, Box<dyn Error>> {
    sysfs_parse!(
        ScalingGovernor,
        "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor",
        cpu
    )
}

pub fn cpux_scaling_governor_avail(cpu: usize) -> Result<HashSet<ScalingGovernor>, Box<dyn Error>> {
    sysfs_parse_set!(
        ScalingGovernor,
        "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_available_governors",
        cpu
    )
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Display, EnumString, IntoStaticStr, EnumCount,
)]
pub enum EnergyPerformancePreference {
    #[strum(serialize = "default")]
    Default,
    #[strum(serialize = "performance")]
    Performance,
    #[strum(serialize = "balance_performance")]
    BalancePerformance,
    #[strum(serialize = "balance_power")]
    BalancePower,
    #[strum(serialize = "power")]
    Power,
}

pub fn cpux_epp_active(cpu: usize) -> Result<EnergyPerformancePreference, Box<dyn Error>> {
    sysfs_parse!(
        EnergyPerformancePreference,
        "/sys/devices/system/cpu/cpu{}/cpufreq/energy_performance_preference",
        cpu
    )
}

//TODO make macro to merge with scaling governors
pub fn cpux_epp_avail(cpu: usize) -> Result<HashSet<EnergyPerformancePreference>, Box<dyn Error>> {
    sysfs_parse_set!(
        EnergyPerformancePreference,
        "/sys/devices/system/cpu/cpu{}/cpufreq/energy_performance_available_preferences",
        cpu
    )
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
