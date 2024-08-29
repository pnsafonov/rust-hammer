use std::env;
use std::process;
use std::process::exit;
use log::{info, error};
use libc::{getutxent, USER_PROCESS};

fn main() {
    do_main();
}

fn do_main() {
    env_logger::init();

    let args: Vec<_> = env::args().collect();
    let l0 = args.len();
    if l0 > 1 {
        info!("The first argument is {}", args[1]);
    }

    let mut days0 = DEF_DAYS.to_string();
    let mut mins0 = String::from("0");
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
    };
    ctx.init();

    do_job(&ctx);

    info!("all is fine");
}

struct Context {
    days: i32,
    mins: i32,
}

impl Context {
    fn init(&self) {
        info!("days = {}", self.days);
        info!("mins = {}", self.mins);
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

        loop {
            let utp = getutxent();
            if utp.is_null() {
                break;
            }
            let utp0 = *utp;

            if utp0.ut_type != USER_PROCESS {
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