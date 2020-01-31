use clap::{crate_name, crate_version, App, AppSettings, Arg, ArgMatches};
use d5_cli::D5;
use ssh_home::SshHome;
use std::{error::Error, net::Ipv4Addr, os::unix::process::CommandExt, process::Command};
use utils::Die;

fn main() {
    #[rustfmt::skip]
    let cli = App::new(crate_name!())
        .version(crate_version!())
        .about("Connect to visual programs on my home development server via VNC.")
        .setting(AppSettings::SubcommandRequired)
        .subcommand(App::new("start")
                .about("Start (and connect to) a new program")
                .arg("--ip [IP_ADDRESS] 'The IP address to use (instead of getting it via d5)'")
                .arg(Arg::with_name("PROGRAM")
                        .help("The program to launch")
                        .required(true)))
        .subcommand(App::new("attach")
                .about("Attach to an existing DESKTOP at the provided number")
                .arg("--ip [IP_ADDRESS] 'The IP address to use (instead of getting it via d5)'")
                .arg(Arg::with_name("DESKTOP")
                        .help("The desktop session to attach to")
                        .required(true)))
        .subcommand(App::new("kill")
                .about("Terminate the VNC server at a specified DESKTOP number")
                .arg("--ip [IP_ADDRESS] 'The IP address to use (instead of getting it via d5)'")
                .arg(Arg::with_name("DESKTOP")
                        .help("The desktop session to terminate")
                        .required(true)))
        .subcommand(App::new("list").about("List all active desktops (by number)"))
        .subcommand(App::new("src").about("--src 'Prints this program's source to stdout'"))
        .get_matches();
    run(cli).unwrap_or_die();
}

fn run(cli: ArgMatches) -> Result<(), Box<dyn Error>> {
    match cli.subcommand_name() {
        Some("src") => {
            print!("/// main.rs\n{}", include_str!("main.rs"));
        }
        Some("attach") => {
            let cli = cli.subcommand_matches("attach").expect("Required by match");
            let vnc_addr = &format!(
                "{computer_addr}:{desktop_number}",
                computer_addr = "192.168.0.222",
                desktop_number = cli.value_of("DESKTOP").expect("required by Clap"),
            );
            Command::new("vncviewer").arg(vnc_addr).exec();
        }
        Some("start") => {
            let cli = cli.subcommand_matches("start").expect("Required by match");
            let vnc_cmd = &format!(
                "vncserver -xstartup /home/dsock/.vnc/init.arg -name {program}",
                program = cli.value_of("PROGRAM").expect("required by Clap")
            );

            let mut ssh_home = SshHome::new(get_ip(cli.value_of("ip"), cli.value_of("pass"))?);
            ssh_home.command = Some(vnc_cmd);
            let (_out, result_msg) = ssh_home.run()?; // Successful execution => status message to stderr
            print!("{}", result_msg);

            let vnc_addr = &format!(
                "{computer_addr}:{desktop_number}",
                computer_addr = "192.168.0.222",
                desktop_number = result_msg.rmatches(char::is_numeric).nth(0).unwrap(),
            );
            Command::new("vncviewer").arg(vnc_addr).exec();
        }
        Some("kill") => {
            let cli = cli.subcommand_matches("kill").expect("Required by match");

            let mut ssh_home = SshHome::new(get_ip(cli.value_of("ip"), cli.value_of("pass"))?);
            let cmd = format!(
                "vncserver -kill :{target_desktop}",
                target_desktop = cli.value_of("DESKTOP").expect("required by Clap")
            );
            ssh_home.command = Some(&cmd);
            let (_out, result_msg) = ssh_home.run()?; // Successful execution => status message to stderr
            print!("{}", result_msg);
        }
        Some("list") => {
            let mut ssh_home = SshHome::new(get_ip(cli.value_of("ip"), cli.value_of("pass"))?);
            ssh_home.command = Some("vncserver -list");
            let (out, err) = ssh_home.run()?;
            print!("{}{}", out, err);
        }
        None | Some(_) => unreachable!(), // Clap requires a subcommand and all commands covered
    };
    Ok(())
}

fn get_ip(ip: Option<&str>, pass: Option<&str>) -> Result<Ipv4Addr, Box<dyn Error>> {
    let ip: std::net::Ipv4Addr = match ip {
        Some(ip) => ip
            .parse()
            .map_err(|_| format!("{} is not a valid IP address", ip))?,
        None => {
            let mut d5 = D5::new();
            d5.password = pass;
            d5.try_ip()?
        }
    };
    Ok(ip)
}
