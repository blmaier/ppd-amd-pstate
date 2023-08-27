use std::collections::hash_set::HashSet;
use std::error::Error;
use std::fs;
use std::io;
use std::str::FromStr;
use strum::EnumCount;
use strum_macros::{Display, EnumCount, EnumIter, EnumString, IntoStaticStr};

trait_set::trait_set! {
    trait SysfsEnum = std::fmt::Debug + Clone + Copy + PartialEq + Eq + FromStr + std::hash::Hash + std::fmt::Display + EnumCount + strum::IntoEnumIterator;
}

macro_rules! sysfs_enum {
    ($i:item) => {
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            Hash,
            Display,
            EnumString,
            IntoStaticStr,
            EnumCount,
            EnumIter,
        )]
        $i
    };
}

fn sysfs_read(path: &str) -> io::Result<String> {
    Ok(String::from(fs::read_to_string(path)?.trim()))
}

fn sysfs_parse<T: FromStr>(path: &str) -> Result<T, Box<dyn Error>>
where
    <T as FromStr>::Err: Error + 'static,
{
    Ok(sysfs_read(path)?.parse::<T>()?)
}

fn sysfs_parse_hashset<T: SysfsEnum>(path: &str) -> Result<HashSet<T>, Box<dyn Error>>
where
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

macro_rules! sysfs_parse_hashset {
    ($type:ident, $($tts:tt)*) => {
        sysfs_parse_hashset::<$type>(format!($($tts)*).as_str())
    }
}

#[cfg(test)]
fn test_sysfs_enum_parse<T: SysfsEnum>(tests: Vec<(&str, T)>) -> Result<(), Box<dyn Error>>
where
    <T as FromStr>::Err: Error + 'static,
{
    let mut variants = HashSet::<T>::new();
    // Check each string matches its variant
    for (string, value) in tests.iter() {
        assert_eq!(string.parse::<T>()?, *value);
        variants.insert(*value);
    }
    // Check all enum variants were tested
    assert_eq!(variants, HashSet::<T>::from_iter(T::iter()));
    Ok(())
}

pub mod cpu {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Cpu {
        Index(usize),
    }

    impl Cpu {
        fn to_index(&self) -> usize {
            match self {
                Cpu::Index(x) => *x,
            }
        }

        pub fn to_path(&self) -> String {
            format!("/sys/devices/system/cpu/cpu{}", self.to_index())
        }
    }

    impl std::fmt::Display for Cpu {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "cpu{}", self.to_index())
        }
    }

    pub fn list_parse(cpu_string: &str) -> Result<Vec<Cpu>, Box<dyn Error>> {
        let groups = cpu_string.split(",");

        let mut cpu_iter: Box<dyn Iterator<Item = usize>> = Box::new(std::iter::empty::<usize>());
        for group in groups {
            let mut range = group.split("-");

            let left = range.next().ok_or("Missing left index")?.parse::<usize>()?;
            let right = match range.next() {
                Some(x) => x.parse::<usize>()?,
                None => left,
            };
            cpu_iter = Box::new(cpu_iter.chain(left..=right));
        }

        Ok(cpu_iter.map(|v| Cpu::Index(v)).collect())
    }

    pub fn possible() -> Result<Vec<Cpu>, Box<dyn Error>> {
        let possible = sysfs_read("/sys/devices/system/cpu/possible")?;
        list_parse(possible.as_str())
    }

    pub mod amd_pstate {
        use super::*;

        sysfs_enum! {
            #[strum(serialize_all = "kebab-case")]
            pub enum Status {
                Active,
                Guided,
                Passive,
            }
        }

        pub fn status() -> Result<Status, Box<dyn Error>> {
            sysfs_parse::<Status>("/sys/devices/system/cpu/amd_pstate/status")
        }

        #[cfg(test)]
        mod tests {
            use super::*;
            #[test]
            fn test_status_parse() -> Result<(), Box<dyn Error>> {
                test_sysfs_enum_parse(vec![
                    ("active", Status::Active),
                    ("guided", Status::Guided),
                    ("passive", Status::Passive),
                ])
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        macro_rules! to_cpu_chains {
            ($l:expr) => {
                {
                    ($l..=$l)
                }
            };
            ($l:expr, $r:expr) => {
                {
                    ($l..=$r)
                }
            };
            ($l:expr, $r:expr, $($es:expr),+) => {
                ($l..=$r).chain(to_cpu_chains! { $($es),+ } )
            };
        }

        macro_rules! to_cpus {
            ($($es:expr),+) => {
                {
                    to_cpu_chains! { $($es),+ }.map(|v| Cpu::Index(v)).collect::<Vec<Cpu>>()
                }
            };
        }

        #[test]
        fn test_list_parse() -> Result<(), Box<dyn Error>> {
            // Single element
            assert_eq!(list_parse("0")?, to_cpus!(0));
            assert_eq!(list_parse("1000")?, to_cpus!(1000));
            // Two elements
            assert_eq!(list_parse("0,1000")?, to_cpus!(0, 0, 1000));
            assert_eq!(list_parse("1,1000")?, to_cpus!(1, 1, 1000));
            // Single range
            assert_eq!(list_parse("0-1000")?, to_cpus!(0, 1000));
            assert_eq!(list_parse("1-1000")?, to_cpus!(1, 1000));
            // Range and element
            assert_eq!(list_parse("0-4,6")?, to_cpus!(0, 4, 6));
            // Two ranges
            assert_eq!(list_parse("0-4,6-8")?, to_cpus!(0, 4, 6, 8));
            // Inverse test
            assert_ne!(list_parse("0-3")?, to_cpus!(0, 4));
            Ok(())
        }
    }

    pub mod policy {
        use super::cpu::Cpu;
        use super::*;

        sysfs_enum! {
            #[strum(serialize_all = "kebab-case")]
            pub enum ScalingDriver {
                AcpiCpufreq,
                AmdPstate,
                AmdPstateEpp,
                CppcCpufreq,
                IntelCpufreq,
                IntelPstate,
                SpeedstepLib,
            }
        }

        pub fn cpux_scaling_driver(cpu: Cpu) -> Result<ScalingDriver, Box<dyn Error>> {
            sysfs_parse!(ScalingDriver, "{}/cpufreq/scaling_driver", cpu.to_path())
        }

        sysfs_enum! {
            #[strum(serialize_all = "kebab-case")]
            pub enum ScalingGovernor {
                Conservative,
                Ondemand,
                Performance,
                Powersave,
                Schedutil,
                Userspace,
            }
        }

        pub fn cpux_scaling_governor_active(cpu: Cpu) -> Result<ScalingGovernor, Box<dyn Error>> {
            sysfs_parse!(
                ScalingGovernor,
                "{}/cpufreq/scaling_governor",
                cpu.to_path()
            )
        }

        pub fn cpux_scaling_governor_avail(
            cpu: Cpu,
        ) -> Result<HashSet<ScalingGovernor>, Box<dyn Error>> {
            sysfs_parse_hashset!(
                ScalingGovernor,
                "{}/cpufreq/scaling_available_governors",
                cpu.to_path()
            )
        }

        sysfs_enum! {
            #[strum(serialize_all = "snake_case")]
            pub enum EnergyPerformancePreference {
                BalancePerformance,
                BalancePower,
                Default,
                Performance,
                Power,
            }
        }

        pub fn cpux_epp_active(cpu: Cpu) -> Result<EnergyPerformancePreference, Box<dyn Error>> {
            sysfs_parse!(
                EnergyPerformancePreference,
                "{}/cpufreq/energy_performance_preference",
                cpu.to_path()
            )
        }

        pub fn cpux_epp_avail(
            cpu: Cpu,
        ) -> Result<HashSet<EnergyPerformancePreference>, Box<dyn Error>> {
            sysfs_parse_hashset!(
                EnergyPerformancePreference,
                "{}/cpufreq/energy_performance_available_preferences",
                cpu.to_path()
            )
        }

        #[cfg(test)]
        mod tests {
            use super::*;

            #[test]
            fn test_scaling_driver_parse() -> Result<(), Box<dyn Error>> {
                test_sysfs_enum_parse(vec![
                    ("acpi-cpufreq", ScalingDriver::AcpiCpufreq),
                    ("amd-pstate", ScalingDriver::AmdPstate),
                    ("amd-pstate-epp", ScalingDriver::AmdPstateEpp),
                    ("cppc-cpufreq", ScalingDriver::CppcCpufreq),
                    ("intel-cpufreq", ScalingDriver::IntelCpufreq),
                    ("intel-pstate", ScalingDriver::IntelPstate),
                    ("speedstep-lib", ScalingDriver::SpeedstepLib),
                ])
            }

            #[test]
            fn test_scaling_governor_parse() -> Result<(), Box<dyn Error>> {
                test_sysfs_enum_parse(vec![
                    ("conservative", ScalingGovernor::Conservative),
                    ("ondemand", ScalingGovernor::Ondemand),
                    ("performance", ScalingGovernor::Performance),
                    ("powersave", ScalingGovernor::Powersave),
                    ("schedutil", ScalingGovernor::Schedutil),
                    ("userspace", ScalingGovernor::Userspace),
                ])
            }
        }
    }
}
