use dbus::blocking::Connection;
use dbus::message::MatchRule;
use std::error::Error;
use std::time::Duration;
use strum_macros::{Display, EnumCount, EnumIter, EnumString, IntoStaticStr};

#[doc(inline)]
pub use std;

mod sysfs;
#[rustfmt::skip]
mod powerprofiles;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Display, EnumString, IntoStaticStr, EnumCount, EnumIter,
)]
#[strum(serialize_all = "kebab-case")]
pub enum Profile {
    PowerSaver,
    Balanced,
    Performance,
}

fn power_profile_active() -> Profile {
    let conn = Connection::new_system().expect("connect error");
    let proxy = conn.with_proxy(
        "net.hadess.PowerProfiles",
        "/net/hadess/PowerProfiles",
        Duration::from_millis(5000),
    );
    use powerprofiles::NetHadessPowerProfiles;
    let profile = proxy.active_profile().expect("get active profile error");
    profile.parse::<Profile>().expect("Failed to parse profile")
}

fn power_profile_monitor() {
    use dbus::channel::MatchingReceiver;

    let conn = Connection::new_system().expect("connect error");

    let path = dbus::strings::Path::new("/net/hadess/PowerProfiles").expect("Invalid dbus path");
    let member = dbus::strings::Member::new("Set").expect("Invalid dbus member");
    let rule = MatchRule::new().with_path(path).with_member(member);

    let proxy = conn.with_proxy(
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        Duration::from_millis(5000),
    );
    let result: Result<(), dbus::Error> = proxy.method_call(
        "org.freedesktop.DBus.Monitoring",
        "BecomeMonitor",
        (vec![rule.match_str()], 0u32),
    );

    result.expect("Failed to open monitor");

    let mut profile = power_profile_active();
    power_profile_active_set(profile).expect("Failed to set profile");

    conn.start_receive(
        rule,
        Box::new(move |_msg, _| {
            let profile_new = power_profile_active();
            if profile != profile_new {
                power_profile_active_set(profile_new).expect("Failed to set profile");
                profile = profile_new;
            }
            true
        }),
    );

    loop {
        conn.process(Duration::from_millis(1000)).unwrap();
    }
}

fn power_profile_active_set(profile: Profile) -> Result<(), Box<dyn Error + 'static>> {
    let gov = sysfs::cpu::policy::ScalingGovernor::Powersave;
    let epp_mode = match profile {
        Profile::PowerSaver => sysfs::cpu::policy::EnergyPerformancePreference::Power,
        Profile::Balanced => sysfs::cpu::policy::EnergyPerformancePreference::BalancePerformance,
        Profile::Performance => sysfs::cpu::policy::EnergyPerformancePreference::Performance,
    };

    for cpu in sysfs::cpu::possible()? {
        let gov_now = sysfs::cpu::policy::cpux_scaling_governor_active(cpu)?;
        if gov != gov_now {
            println!("Would reconfigure gov {} for {}", gov, cpu);
            //TODO Set scaling governor
        }
        let epp_mode_now = sysfs::cpu::policy::cpux_epp_active(cpu)?;
        if epp_mode != epp_mode_now {
            println!("Would reconfigure epp {} for {}", epp_mode_now, cpu);
            //TODO Set scaling governor
        }
    }
    Ok(())
}

fn print_info() {
    fn str_or_unknown<V: std::string::ToString, E: std::fmt::Debug>(res: Result<V, E>) -> String {
        res.map_or_else(|e| format!("Unknown ({:#?})", e), |v| v.to_string())
    }

    println!(
        "amd pstate status: {}",
        str_or_unknown(sysfs::cpu::amd_pstate::status())
    );
    println!("Power Profile: {}", power_profile_active());
    match sysfs::cpu::possible() {
        Ok(cpus) => {
            if let Some(cpu) = cpus.into_iter().next() {
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
    power_profile_monitor();
}
