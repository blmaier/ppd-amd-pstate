use dbus::blocking::Connection;
use std::time::Duration;

mod powerprofiles;
mod sysfs;

fn power_profile_active() -> String {
    let c = Connection::new_system().expect("connect error");
    let p = c.with_proxy(
        "net.hadess.PowerProfiles",
        "/net/hadess/PowerProfiles",
        Duration::from_millis(5000),
    );
    use powerprofiles::NetHadessPowerProfiles;
    p.active_profile().expect("get active profile error")
}

fn errstr<V: std::fmt::Display, E: std::fmt::Display>(res: Result<V, E>) -> String {
    res.map_or_else(|e| e.to_string(), |v| v.to_string())
}

fn print_info() {
    println!("amd pstate status: {}", sysfs::amd_pstate_is_active());
    println!("Power Profile: {}", power_profile_active());
    match sysfs::cpu_possible() {
        Ok(cpus) => {
            for cpu in cpus.into_iter() {
                println!("cpu{}", cpu);
                println!(
                    "  scaling driver: {}",
                    errstr(sysfs::cpux_scaling_driver(cpu))
                );
                println!(
                    "  scaling driver is epp: {}",
                    sysfs::cpux_scaling_driver_is_epp(cpu)
                );
                println!("  epp active: {}", errstr(sysfs::cpux_epp_active(cpu)));
                println!(
                    "  epp avail: {}",
                    errstr(sysfs::cpux_epp_avail(cpu).map(|e| e.join(", ")))
                );
                println!(
                    "  scaling governor active: {}",
                    errstr(sysfs::cpux_scaling_governor_active(cpu))
                );
                println!(
                    "  scaling governor avail: {}",
                    errstr(sysfs::cpux_scaling_governor_avail(cpu).map(|e| e.join(", ")))
                );
                break;
            }
        }
        Err(e) => println!("No CPUs found: {:#?}", e),
    }
}

fn assert_amd_pstate() {
    if !sysfs::amd_pstate_is_active() {
        panic!("System is not using AMD pstate");
    };
    for cpu in sysfs::cpu_possible().expect("No CPUs found") {
        if sysfs::cpux_scaling_driver(cpu).expect("No scaling driver") != "amd-pstate-epp" {
            panic!("cpu{} not using amd-pstate-epp scaling driver", cpu);
        };
    }
}

fn main() {
    print_info();
    assert_amd_pstate();
}
