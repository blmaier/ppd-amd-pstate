use dbus::blocking::Connection;
use std::error::Error;
use std::time::Duration;

mod sysfs;
#[rustfmt::skip]
mod powerprofiles;

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
    fn str_or_unknown<V: std::string::ToString, E: std::fmt::Debug>(res: Result<V, E>) -> String {
        res.map_or_else(|e| format!("Unknown ({:#?})", e), |v| v.to_string())
    }

    println!("amd pstate status: {}", str_or_unknown(sysfs::cpu::amd_pstate::status()));
    println!("Power Profile: {}", power_profile_active());
    match sysfs::cpu::possible() {
        Ok(cpus) => {
            for cpu in cpus.into_iter() {
                println!("{}", cpu);
                println!(
                    "  scaling driver: {}",
                    str_or_unknown(sysfs::cpu::policy::cpux_scaling_driver(cpu))
                );
                println!(
                    "  epp active: {}",
                    str_or_unknown(sysfs::cpu::policy::cpux_epp_active(cpu))
                );
                println!(
                    "  epp avail: {}",
                    str_or_unknown(sysfs::cpu::policy::cpux_epp_avail(cpu).map(|e| {
                        e.iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<_>>()
                            .join(" ")
                    }))
                );
                println!(
                    "  scaling governor active: {}",
                    str_or_unknown(sysfs::cpu::policy::cpux_scaling_governor_active(cpu))
                );
                println!(
                    "  scaling governor avail: {}",
                    str_or_unknown(
                        sysfs::cpu::policy::cpux_scaling_governor_avail(cpu).map(|e| {
                            e.iter()
                                .map(|v| v.to_string())
                                .collect::<Vec<_>>()
                                .join(" ")
                        })
                    )
                );
                break;
            }
        }
        Err(e) => println!("No CPUs found: {:#?}", e),
    }
}

fn is_amd_pstate() -> Result<(), Box<dyn Error>> {
    use sysfs::cpu::amd_pstate::{status, Status};

    if status()? != Status::Active {
        return Err("AMD PState not active".into());
    }

    use sysfs::cpu::{policy::cpux_scaling_driver, policy::ScalingDriver, possible};

    for cpu in possible()? {
        if cpux_scaling_driver(cpu)? != ScalingDriver::AmdPstateEpp {
            return Err("Not all CPUs in amd-pstate-epp mode".into());
        };
    }

    Ok(())
}

fn main() {
    print_info();
    is_amd_pstate().expect("AMD-pstate not active");
}
