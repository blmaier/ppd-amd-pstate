
use std::fs;
use zbus::{Connection, dbus_proxy, Result};

fn amd_pstate_is_active() -> bool {
    match fs::read_to_string("/sys/devices/system/cpu/amd_pstate/status") {
        Ok(s) => s.trim() == "active",
        Err(_) => false,
    }
}

fn cpux_scaling_driver(cpu : u32) -> String {
    let path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_driver", cpu);
    String::from(fs::read_to_string(path.as_str())
                 .expect("Failed to read active scaling driver")
                 .trim())
}

fn cpux_scaling_driver_is_epp(cpu : u32) -> bool {
    cpux_scaling_driver(cpu) == "amd-pstate-epp"
}

fn cpu_possible() -> Vec<u32> {
    let present = String::from(fs::read_to_string("/sys/devices/system/cpu/possible")
                               .expect("Failed to read CPU present")
                               .trim());

    let groups = present.split(",");

    groups.map(|group| {
        let mut range = group.split("-");

        let left = range.next()
            .expect("CPU possible contains invalid range")
            .parse::<u32>().expect("CPU possible contains invalid left range");
        let right = match range.next() {
            Some(x) => x.parse::<u32>().expect("CPU possible contains invalid right range"),
            None => left,
        };
        left..=right
    }).flatten().collect()
}

fn cpux_scaling_governor_active(cpu : u32) -> String {
    let path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor", cpu);
    String::from(fs::read_to_string(path.as_str())
                 .expect("Failed to read active scaling governor")
                 .trim())
}

fn cpux_scaling_governor_avail(cpu : u32) -> Vec<String> {
    let path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_available_governors", cpu);
    let avail_str = String::from(fs::read_to_string(path.as_str())
                                 .expect("Failed to read available scaling governors")
                                 .trim());
    let avail = avail_str.split_whitespace();
    avail.map(|x| String::from(x)).collect()
}

fn cpux_epp_active(cpu : u32) -> String {
    let path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/energy_performance_preference", cpu);
    String::from(fs::read_to_string(path.as_str())
                 .expect("Failed to read EPP active")
                 .trim())
}

fn cpux_epp_avail(cpu : u32) -> Vec<String> {
    let path = format!( "/sys/devices/system/cpu/cpu{}/cpufreq/energy_performance_available_preferences", cpu);
    let avail_str = String::from(fs::read_to_string(path.as_str())
                                 .expect("Failed to read EPP available")
                                 .trim());
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

async fn power_profile_active() -> String {
    let connection = Connection::system().await.expect("A");
    let proxy = PowerProfilesProxy::new(&connection).await.expect("A");
    proxy.active_profile().await.expect("A")
}

async fn print_info() {
    println!("amd pstate status: {}", amd_pstate_is_active());
    println!("Power Profile: {}", power_profile_active().await);
    for cpu in cpu_possible() {
        println!("cpu{}", cpu);
        println!("  scaling driver: {}", cpux_scaling_driver(cpu));
        println!("  scaling driver is epp: {}", cpux_scaling_driver_is_epp(cpu));
        println!("  epp active: {}", cpux_epp_active(cpu));
        println!("  epp avail: {}", cpux_epp_avail(cpu).join(", "));
        println!("  scaling governor active: {}", cpux_scaling_governor_active(cpu));
        println!("  scaling governor avail: {}", cpux_scaling_governor_avail(cpu).join(", "));
        break;
    }
}

fn assert_amd_pstate() {
    if ! amd_pstate_is_active() {
        panic!("System is not using AMD pstate");
    };
    for cpu in cpu_possible() {
        if cpux_scaling_driver(cpu) != "amd-pstate-epp" {
            panic!("cpu{} not using amd-pstate-epp scaling driver", cpu);
        };
    };
}

#[async_std::main]
async fn main() {
    print_info().await;
    assert_amd_pstate();
}
