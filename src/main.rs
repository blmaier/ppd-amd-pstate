use std::fs;
use zbus::blocking::Connection;
use zbus::{dbus_proxy, Result};

fn amd_pstate_is_active() -> bool {
    match fs::read_to_string("/sys/devices/system/cpu/amd_pstate/status") {
        Ok(s) => s.trim() == "active",
        Err(_) => false,
    }
}

fn cpux_scaling_driver(cpu: usize) -> String {
    let path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_driver", cpu);
    String::from(
        fs::read_to_string(path.as_str())
            .expect("Failed to read active scaling driver")
            .trim(),
    )
}

fn cpux_scaling_driver_is_epp(cpu: usize) -> bool {
    cpux_scaling_driver(cpu) == "amd-pstate-epp"
}

fn cpu_parse_range(cpu_string: &str) -> Vec<usize> {
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
        assert_eq!(cpu_parse_range("1-1000"), (1..=1000).collect::<Vec<usize>>());
        assert_ne!(cpu_parse_range("0-3"), (0..=4).collect::<Vec<usize>>());
    }
}

fn cpu_possible() -> Vec<usize> {
    let present =
        fs::read_to_string("/sys/devices/system/cpu/possible").expect("Failed to read CPU present");
    cpu_parse_range(present.trim())
}

fn cpux_scaling_governor_active(cpu: usize) -> String {
    let path = format!(
        "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor",
        cpu
    );
    String::from(
        fs::read_to_string(path.as_str())
            .expect("Failed to read active scaling governor")
            .trim(),
    )
}

fn cpux_scaling_governor_avail(cpu: usize) -> Vec<String> {
    let path = format!(
        "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_available_governors",
        cpu
    );
    let avail_str = String::from(
        fs::read_to_string(path.as_str())
            .expect("Failed to read available scaling governors")
            .trim(),
    );
    let avail = avail_str.split_whitespace();
    avail.map(|x| String::from(x)).collect()
}

fn cpux_epp_active(cpu: usize) -> String {
    let path = format!(
        "/sys/devices/system/cpu/cpu{}/cpufreq/energy_performance_preference",
        cpu
    );
    String::from(
        fs::read_to_string(path.as_str())
            .expect("Failed to read EPP active")
            .trim(),
    )
}

fn cpux_epp_avail(cpu: usize) -> Vec<String> {
    let path = format!(
        "/sys/devices/system/cpu/cpu{}/cpufreq/energy_performance_available_preferences",
        cpu
    );
    let avail_str = String::from(
        fs::read_to_string(path.as_str())
            .expect("Failed to read EPP available")
            .trim(),
    );
    let avail = avail_str.split_whitespace();
    avail.map(|x| String::from(x)).collect()
}

#[dbus_proxy(
    interface = "net.hadess.PowerProfiles",
    default_path = "/net/hadess/PowerProfiles"
)]
trait PowerProfiles {
    #[dbus_proxy(property)]
    fn active_profile(&self) -> Result<String>;
}

fn power_profile_active() -> String {
    let connection = Connection::system().expect("A");
    let proxy = PowerProfilesProxyBlocking::new(&connection).expect("A");
    proxy.active_profile().expect("A")
}

fn monitor_power_profile() {
    let connection = Connection::system().expect("A");
}

fn print_info() {
    println!("amd pstate status: {}", amd_pstate_is_active());
    println!("Power Profile: {}", power_profile_active());
    for cpu in cpu_possible() {
        println!("cpu{}", cpu);
        println!("  scaling driver: {}", cpux_scaling_driver(cpu));
        println!(
            "  scaling driver is epp: {}",
            cpux_scaling_driver_is_epp(cpu)
        );
        println!("  epp active: {}", cpux_epp_active(cpu));
        println!("  epp avail: {}", cpux_epp_avail(cpu).join(", "));
        println!(
            "  scaling governor active: {}",
            cpux_scaling_governor_active(cpu)
        );
        println!(
            "  scaling governor avail: {}",
            cpux_scaling_governor_avail(cpu).join(", ")
        );
        break;
    }
}

fn assert_amd_pstate() {
    if !amd_pstate_is_active() {
        panic!("System is not using AMD pstate");
    };
    for cpu in cpu_possible() {
        if cpux_scaling_driver(cpu) != "amd-pstate-epp" {
            panic!("cpu{} not using amd-pstate-epp scaling driver", cpu);
        };
    }
}

fn main() {
    print_info();
    assert_amd_pstate();


}
