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

fn print_info() {
    println!("amd pstate status: {}", sysfs::amd_pstate_is_active());
    println!("Power Profile: {}", power_profile_active());
    for cpu in sysfs::cpu_possible() {
        println!("cpu{}", cpu);
        println!("  scaling driver: {}", sysfs::cpux_scaling_driver(cpu));
        println!(
            "  scaling driver is epp: {}",
            sysfs::cpux_scaling_driver_is_epp(cpu)
        );
        println!("  epp active: {}", sysfs::cpux_epp_active(cpu));
        println!("  epp avail: {}", sysfs::cpux_epp_avail(cpu).join(", "));
        println!(
            "  scaling governor active: {}",
            sysfs::cpux_scaling_governor_active(cpu)
        );
        println!(
            "  scaling governor avail: {}",
            sysfs::cpux_scaling_governor_avail(cpu).join(", ")
        );
        break;
    }
}

fn assert_amd_pstate() {
    if !sysfs::amd_pstate_is_active() {
        panic!("System is not using AMD pstate");
    };
    for cpu in sysfs::cpu_possible() {
        if sysfs::cpux_scaling_driver(cpu) != "amd-pstate-epp" {
            panic!("cpu{} not using amd-pstate-epp scaling driver", cpu);
        };
    }
}

fn main() {
    print_info();
    assert_amd_pstate();
}
