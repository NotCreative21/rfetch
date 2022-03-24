use std::{
    fs::{self, read_to_string},
    process::Command,
    str,
};
use sysinfo::{ComponentExt, ProcessorExt, System, SystemExt};

const LOGO_COLOR: &'static str = "\x1b[0;34m";
const TEXT_COLOR: &'static str = "\x1b[0;32m";

/*
 * returns the current shell used by the user by parsing /etc/passwd
 */
fn get_shell() -> String {
    let passwd: String = match read_to_string("/etc/passwd") {
        Ok(v) => v,
        Err(_) => " ".to_string(),
    };

    let shell: &str = &passwd[22..passwd.find("\n").unwrap()];

    shell.to_string()
}

/*
 * tiny function to change the output from sysinfo into MiB
 *
 * This number is used because it matches with htop lmao, not sure why, but 1024 is just inaccurate
 * to both free/avaible memory
 */
fn make_pretty(i: u64) -> u32 {
    (i as f32 / 1073.7).round() as u32
}

/*
 * This website was used to generate the logo: https://cloudapps.herokuapp.com/imagetoascii/
 * The hex color code in the front of the string is the same as you would put in your bashrc file
 * for example, just in hexadecimal format instead of octal, this is because rust does not support
 * printing octal correctly
 */
fn print_logo(sys: System, uptime: String, actual_wm: &str, cpu: String, swap: String) {
    let mut logo: Vec<String> = Vec::new();
    match fs::read_to_string(
        dirs::home_dir().unwrap().to_string_lossy().to_string() + "/.config/rfetch.txt",
    ) {
        Ok(v) => v
            .replace("{", "{{")
            .replace("}", "}}")
            .replace("\\", "\\\\")
            .split("\n")
            .for_each(|x| logo.push(x.to_string())),
        Err(e) => {
            //todo
            println!("{e}");
            std::process::exit(0);
        }
    };

    let info = vec![
        format!("host:\t{}", sys.host_name().unwrap()),
        format!("distro:\t{}", sys.name().unwrap()),
        format!("kernel:\t{}", sys.kernel_version().unwrap()),
        uptime,
        format!("shell:\t{}", get_shell()),
        format!("wm:\t{}", actual_wm),
        format!("cpu:\t{}", cpu),
        format!("temp:\t{}°C", sys.components()[0].temperature()),
        format!(
            "ram:\t{}MiB / {}MiB",
            make_pretty(sys.used_memory()),
            make_pretty(sys.total_memory())
        ),
        swap,
    ];

    let mut output = "".to_string();
    let mut index = 0;
    for i in logo {
        if index + 1 > info.len() {
            //println!("{}{}\t\t", logo_color, i);
            output.push_str(&format!("{LOGO_COLOR}{i}\t\t\n"));
            continue;
        }
        //println!("{}{}\t\t{}{}", logo_color, i, text_color, info[index]);
        output.push_str(&format!("{LOGO_COLOR}{i}\t\t{TEXT_COLOR}{}\n", info[index]));
        index += 1;
    }
    println!("{output}");
}

fn main() {
    // gather system information
    let mut sys = System::new_all();
    sys.refresh_all();

    // if the uptime is less than an hour, display in minutes
    //
    // else print it out in hours
    let uptime = match sys.uptime() {
        3600.. => format!("Uptime:\t{} hrs", sys.uptime() / 3600),
        _ => format!("uptime:\t{} min", sys.uptime() / 60),
    };

    // use xprop to find the wm
    let win_command = Command::new("xprop")
        .args(["-root", "-notype"])
        .output()
        .expect("could not find wm");

    let win_command = str::from_utf8(&win_command.stdout).unwrap();

    let win_id = &win_command[(win_command
        .find("_NET_SUPPORTING_WM_CHECK: window id #")
        .unwrap()
        + 38)..(win_command.find("_XROOTPMAP_ID:").unwrap() - 1)];

    let wm = Command::new("xprop")
        .args(["-id", &win_id])
        .output()
        .expect("could not find wm");

    let mut actual_wm: &str = "";

    // if the error is long, then just skip it
    if wm.stderr.len() < 1 {
        let wm_str = str::from_utf8(&wm.stdout).unwrap();

        actual_wm = &wm_str[29..(wm_str.find("_NET_SUPPORTING").unwrap() - 2) as usize];
    }

    // check each cpu core and display the fastest
    let mut cpu_name = String::new();

    let mut cpu_freq = 0;

    for processor in sys.processors() {
        if processor.frequency() > cpu_freq {
            cpu_freq = processor.frequency();

            // this is only inside the if statement so the amount of times the string gets
            // reassigned is lower
            cpu_name = processor.brand().to_string();
        }
    }

    // if the name is long, simply cut off the end, this may not look pretty/work well for
    // your cpu
    // but it can easily be tweaked
    if cpu_name.len() > 17 {
        cpu_name = cpu_name[..16].to_string();
    }
    let cpu: String = format!(
        "{} ({}) @ {} MHz",
        cpu_name,
        sys.processors().len(),
        cpu_freq
    );
    // this is done so the last line is blank if there is no swap
    let mut swap: String = String::new();

    // if there is no swap then skip, otherwise print
    if sys.total_swap() > 0 {
        swap = format!(
            "swap:\t{:?}MiB / {:?}MiB",
            make_pretty(sys.used_swap()),
            make_pretty(sys.total_swap())
        );
    }

    // print ascii logo
    print_logo(sys, uptime, actual_wm, cpu, swap);
}
