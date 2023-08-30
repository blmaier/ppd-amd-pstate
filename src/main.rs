use dbus::blocking::Connection;
use dbus::message::MatchRule;
use dbus::Message;
use std::error::Error;
use std::time::Duration;
use strum_macros::{Display, EnumCount, EnumIter, EnumString, IntoStaticStr};

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

fn handle_message(msg: &Message) {
    println!("Message: {:?}", msg);
    update_profile();
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

    conn.start_receive(
        rule,
        Box::new(|msg, _| {
            handle_message(&msg);
            true
        }),
    );

    loop {
        conn.process(Duration::from_millis(1000)).unwrap();
    }
}

fn update_profile() {
    let _active = power_profile_active();
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
    update_profile();
    power_profile_monitor();
}
