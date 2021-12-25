use std::{
    fs::read_to_string, 
    process::Command, 
    str
};
use sysinfo::{
    ComponentExt, 
    ProcessorExt, 
    System, 
    SystemExt
};

/*
 * returns the current shell used by the user by parsing /etc/passwd
 */
fn get_shell() -> String {
    let passwd: String = read_to_string("/etc/passwd").expect("could not determine shell");

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
    (i as f32 / 1045.0).round() as u32
}

/*
 * This website was used to generate the logo: https://cloudapps.herokuapp.com/imagetoascii/
 * The hex color code in the front of the string is the same as you would put in your bashrc file
 * for example, just in hexadecimal format instead of octal, this is because rust does not support
 * printing octal correctly
 */
fn print_logo(sys: System, uptime: String, actual_wm: &str, cpu: String, swap: String) {
    // logo color using bash color codes
    let lcolor = "\x1b[0;36m ".to_string();

    // text color, also using bash color codes
    let tcolor = "\x1b[0;32m".to_string();

    // print out the logo, this is very janky but it's a simple way to do it
    let all = format!(
        "\x1b[0;36m
{}      Q9kwojju]t]O                    {}Host:\t{}
{}    e}}X#########Nh(le                 {}Distro:\t{}
{}  etg######NZ]2q####Oi\\q              {}Kernel:\t{}
{} ce#######N,     ?8###Nu;]            {}{}
{} :SN######N:,     v######o_t          {}Shell:\t{}
{} #s;^]O#####Otlvv6########Nr,Q        {}WM:\t{}
{}   Q8u=;;cN################{{ o        {}CPU:\t{}
{}      Q]\\a###############w:`jQ        {}temp:\t{:?}°C
{}    Q]lO##############Nf,`/gQ         {}RAM:\t{}MiB / {}MiB
{}  #i{{g##############9?`,}}NQ           {}{}
{} //N#############Ki,'\\qQ
{}f_###########ge|',\\kBQ
{}$`zg####gqu|:,ruRQQ
{} O>,~:::;=zwgBQQ
{}  QQQ##BQQQ
             ",
        lcolor, tcolor, sys.host_name().unwrap(),
        lcolor, tcolor, sys.name().unwrap(),
        lcolor, tcolor, sys.kernel_version().unwrap(),
        lcolor, tcolor, uptime,
        lcolor, tcolor, get_shell(),
        lcolor, tcolor, actual_wm,
        lcolor, tcolor, cpu,
        lcolor, tcolor, sys.components()[0].temperature(),
        lcolor, tcolor, make_pretty(sys.used_memory()), make_pretty(sys.total_memory()),
        lcolor, tcolor, swap,
        lcolor,
        lcolor,
        lcolor,
        lcolor,
        lcolor
    );

    println!("{}", all);
}

fn main() {
    // gather system information
    let mut sys = System::new_all();
    sys.refresh_all();

    // if the uptime is less than an hour, display in minutes
    //
    // else print it out in hours
    let mut uptime: String = String::new();
    if sys.uptime() < 60 {
        uptime = format!("Uptime:\t{} min", sys.uptime() / 60);
    } else {
        uptime = format!("Uptime:\t{} hrs", sys.uptime() / 3600);
    }

    // use xprop to find the wm
    let win_command = Command::new("xprop")
        .args(["-root", "-notype"])
        .output()
        .expect("could not find wm");

    let win_command = str::from_utf8(&win_command.stdout).unwrap();

    let win_id = &win_command[(win_command.find("_NET_SUPPORTING_WM_CHECK: window id #").unwrap() + 38)..(win_command.find("_XROOTPMAP_ID:").unwrap() - 1)];

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

    // if the name is long, simply cut off the end, this may not look pretty/work well for your cpu
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
            "Swap:\t{:?}MiB / {:?}MiB",
            make_pretty(sys.used_swap()),
            make_pretty(sys.total_swap())
        );
    }

    // print ascii logo
    print_logo(sys, uptime, actual_wm, cpu, swap);
}
