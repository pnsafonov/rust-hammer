use std::env;
use std::process;
use std::process::exit;
use log::{info, error};
use libc::{getutxent, USER_PROCESS, DEAD_PROCESS, printf};

fn main() {
    do_main();
}

fn do_main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"))
        .init();

    let args: Vec<_> = env::args().collect();
    let l0 = args.len();
    let mut days0 = DEF_DAYS.to_string();
    let mut mins0 = String::from("0");
    let mut verbose0 = false;
    for (i, arg) in args.iter().enumerate() {
        match arg.as_str() {
            "-d"|"--days" => {
                let i0 = i + 1;
                if i0 < l0 {
                    days0 = args[i0].clone();
                }
            }
            "-m"|"--mins" => {
                let i0 = i + 1;
                if i0 < l0 {
                    mins0 = args[i0].clone();
                }
            }
            "-h"|"--help" => {
                print_help();
                exit(0);
            }
            "--verbose" => {
                verbose0 = true;
            }
            "-v"|"--version" => {
                print_version();
                exit(0);
            }
            _ => (),
        }
    }

    let days1= match days0.parse::<i32>() {
        Ok(x) => x,
        Err(err) => {
            error!("do_main, days0.parse err = {}", err);
            exit(1);
        },
    };

    let mins1= match mins0.parse::<i32>() {
        Ok(x) => x,
        Err(err) => {
            error!("do_main, days0.parse err = {}", err);
            exit(1);
        },
    };

    let ctx = Context {
        days: days1,
        mins: mins1,
        verbose: verbose0,
    };
    ctx.init();

    do_job(&ctx);

    info!("all is fine");
}

fn print_help() {
    print!("Usage: inact [OPTIONS]
inact checks last logins and do shutdown if no recent logins

    -d, --days <days>     days count before current day to check for logins, default 7
    -m, --mins <mins>     mins count before current time to check for logins
        --verbose         print information about last logins

    -h, --help            display this help and exit
    -v, --version         output version information and exit
");
}

fn print_version() {
    let sha1 = env!("VERGEN_GIT_SHA");
    print!("inact  1.0.2  {}\n", sha1);
}

struct Context {
    days: i32,
    mins: i32,
    verbose: bool,
}

impl Context {
    fn init(&self) {
        info!("days = {}", self.days);
        info!("mins = {}", self.mins);
        info!("verbose = {}", self.verbose);
    }
}

const DEF_DAYS: i32 = 7;

fn do_job(ctx: &Context) {
    let mut count = 0;

    unsafe {
        let now = libc::time(0 as *mut libc::time_t) as i32;
        let mut before= 60 * 60 * 24 * DEF_DAYS;

        if ctx.mins > 0 {
            before = now - (60 * ctx.mins);
        }
        if ctx.mins == 0 && ctx.days > 0 {
            before = now - (60 * 60 * 24 * ctx.days);
        }

        let mut i = 0;
        loop {
            let utp0 = match getutxent().as_ref() {
                Some(x) => x,
                None => break,  // NULL is end of cycle
            };

            if ctx.verbose {
                print_utmpx(&utp0, i);
            }
            i += 1;

            if utp0.ut_type != USER_PROCESS && utp0.ut_type != DEAD_PROCESS {
                continue
            }
            let utp_time = utp0.ut_tv.tv_sec as i32; // for freebsd
            if utp_time < before {
                continue
            }

            count += 1;
        }
    }


    info!("do_job, recent logins count = {}", count);
    if count != 0 {
        info!("do_job, where is recent logins, no shutdown");
        exit(0);
    }

    let args0 = if cfg!(target_os = "linux") {
        ["-h", "now"]
    } else if cfg!(target_os = "freebsd") {
        ["-p", "now"]
    } else {
        ["-h", "now"]
    };

    info!("do_job, do shutdown");
    set_env_path();
    _ = match process::Command::new("shutdown")
        .args(args0).spawn() {
        Ok(mut x) => x.wait(),
        Err(err) => {
            error!("do_job, command err = {}", err);
            exit(1);
        },
    };

    exit(0);
}

unsafe fn print_utmpx(utp0: &libc::utmpx, i: i32) {
    // let s0= std::ffi::CStr::from_ptr(utp0.ut_id.as_ptr());
    // let s1 = std::ffi::CStr::from_ptr(utp0.ut_user.as_ptr());
    // let s2 = std::ffi::CStr::from_ptr(utp0.ut_line.as_ptr());
    // info!("i = {}, ut_type = {}, tv_sec = {}, ut_id = {:?}, ut_pid = {}, ut_user = {:?}, ut_line = {:?}",
    //     i,
    //     utp0.ut_type,
    //     utp0.ut_tv.tv_sec,
    //     s0,
    //     utp0.ut_pid,
    //     s1,
    //     s2,
    // );

    let c_str = std::ffi::CString::new("i = %d, ut_type = %d, tv_sec = %d, ut_id = %s, ut_pid = %d, ut_user = %s, ut_line = %s, ut_host = %s\n").unwrap();
    printf(c_str.as_ptr(),
        i,
        utp0.ut_type as i32,
        utp0.ut_tv.tv_sec,
        utp0.ut_id.as_ptr(),
        utp0.ut_pid,
        utp0.ut_user.as_ptr(),
        utp0.ut_line.as_ptr(),
        utp0.ut_host.as_ptr(),
    );
}

fn set_env_path() {
    let mut val0: String = String::from("");
    let key = "PATH";
    match env::var(key) {
        Ok(val) => val0 = val,
        Err(_) => (),
    };

    if val0 != "" && !val0.ends_with(":") {
        val0 += ":";
    }

    val0 += "/sbin:/bin:/usr/sbin:/usr/bin:/usr/local/sbin:/usr/local/bin:/root/bin";
    env::set_var(key, val0);
}