use std::env;
use std::fs::File;
use std::io::{Error, Write};
use std::process::Command;

const RESOLVCONF_HEAD_ENV_VAR: &str = "RESOLVCONF_HEAD_PATH";
const RESOLVCONF_HEAD_DEFAULT_PATH: &str = "/etc/resolvconf/resolv.conf.d/head";

const DEFAULT_TEMPLATE: &str = "
# Dynamic resolv.conf(5) file for glibc resolver(3) generated by resolvconf(8)
#     DO NOT EDIT THIS FILE BY HAND -- YOUR CHANGES WILL BE OVERWRITTEN
# 127.0.0.53 is the systemd-resolved stub resolver.
# run \"systemd-resolve --status\" to see details about the actual nameservers.
";

const DNS_SERVER_1_ADDR: &str = "94.140.14.14";
const DNS_SERVER_2_ADDR: &str = "94.149.15.15";

const ADGUARD_DNS_SERVER_CONFIG: &str = "
# AdGuard DNS 
# https://adguard-dns.com/en/public-dns.html
nameserver 94.140.14.14
nameserver 94.149.15.15
";

const HELP_MESSAGE: &str = "
Usage: sudo cfg-adguard-dns [options...]

        --activate      Activate AdGuard DNS server 
        --deactivate    Deactivate AdGuard DNS server 
        --status        Shows wether AdGuard DNS server is activated or not
        --help          Display the current help message

Disclaimer: Using this tool will restore the /etc/resolvconf/resolv.conf.d/head file to its default state.
";

fn main() -> Result<(), Error> {
    let mut file = File::create(get_path())?;
    let args: Vec<_> = env::args().collect();

    match args.len() {
        1 => println!("{}", HELP_MESSAGE),
        _ => match &args[1][..] {
            "--help" => println!("{}", HELP_MESSAGE),
            "--activate" | "activate" => activate_adguard_dns(&mut file),
            "--deactivate" | "deactivate" => deactivate_adguard_dns(&mut file),
            "--status" | "status" => show_status(),
            _ => eprintln!("Unknown argument. Try `cfg-adguard-dns --help` for more information"),
        },
    }

    Ok(())
}

fn get_path() -> String {
    match env::var(RESOLVCONF_HEAD_ENV_VAR) {
        Ok(value) => value,
        Err(_) => String::from(RESOLVCONF_HEAD_DEFAULT_PATH),
    }
}

fn activate_adguard_dns(file: &mut File) {
    write_default_template_with_adguard_dns(file);
    update_resolvconf();
}

fn deactivate_adguard_dns(file: &mut File) {
    write_default_template(file);
    update_resolvconf();
}

fn show_status() {
    let output = std::process::Command::new("nslookup")
        .arg("wikipedia.org")
        .output()
        .expect("failed to execute nslookup");

    if let Ok(output) = String::from_utf8(output.stdout) {
        if contains_server_1_or_2_config(output) {
            println!("ADGUARD DNS is activated")
        } else {
            println!("ADGUARD DNS is deactivated")
        }
    } else {
        eprintln!("nslookup is not installed or could not lookup wikipedia.org")
    };
}

fn contains_server_1_or_2_config(output: String) -> bool {
    output.contains(&DNS_SERVER_1_ADDR.to_string())
        || output.contains(&DNS_SERVER_2_ADDR.to_string())
}

fn write_default_template_with_adguard_dns(file: &mut File) {
    let content = format!("{} {}", DEFAULT_TEMPLATE, ADGUARD_DNS_SERVER_CONFIG);
    write!(file, "{}", content).expect("failed to write default template");
}

fn write_default_template(file: &mut File) {
    write!(file, "{}", DEFAULT_TEMPLATE).expect("failed to write default template");
}

fn update_resolvconf() {
    Command::new("resolvconf")
        .arg("-u")
        .output()
        .expect("failed to update resolvconf");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const RESOLVCONF_HEAD_ENV_VAR: &str = "RESOLVCONF_HEAD_PATH";
    const RESOLVCONF_HEAD_DEFAULT_PATH: &str = "/etc/resolvconf/resolv.conf.d/head";

    #[test]
    fn get_path_function_returns_the_resolvconf_head_env_var_value_if_it_is_set() {
        if let Ok(value) = env::var(RESOLVCONF_HEAD_ENV_VAR) {
            assert_eq!(get_path(), value)
        } else {
            assert_eq!(get_path(), RESOLVCONF_HEAD_DEFAULT_PATH)
        }
    }

    #[test]
    fn activate_adguard_dns_test() -> std::io::Result<()> {
        let mut file = File::create("test_file_with_adguard_dns").unwrap();

        activate_adguard_dns(&mut file);

        match fs::read_to_string("test_file_with_adguard_dns") {
            Ok(content) => assert!(content.contains(ADGUARD_DNS_SERVER_CONFIG)),
            Err(_) => panic!("test failed"),
        };

        fs::remove_file("test_file_with_adguard_dns")?;
        Ok(())
    }

    #[test]
    fn deactivate_adguard_dns_test() -> std::io::Result<()> {
        let mut file = File::create("test_file_without_adguard_dns").unwrap();

        deactivate_adguard_dns(&mut file);

        match fs::read_to_string("test_file_without_adguard_dns") {
            Ok(content) => assert!(!content.contains(ADGUARD_DNS_SERVER_CONFIG)),
            Err(_) => panic!("test failed"),
        };

        fs::remove_file("test_file_without_adguard_dns")?;
        Ok(())
    }
}
