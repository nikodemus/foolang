static mut START_TIME: Option<std::time::Instant> = None;

#[derive(Debug)]
pub struct TimeInfo {
  pub user: f64,
  pub system: f64,
  pub wall: f64,
}

impl TimeInfo {
     pub fn init() {
        unsafe { START_TIME = Some(std::time::Instant::now()) };
     }
     #[cfg(target_family = "windows")]
     pub fn now() -> TimeInfo {
        // https://docs.rs/winapi/0.3.7/x86_64-pc-windows-msvc/winapi/um/sysinfoapi/fn.GetSystemInfo.html
           use winapi::um::sysinfoapi::{SYSTEM_INFO, GetSystemInfo};
           let mut info = SYSTEM_INFO.default();
     }
     #[cfg(target_family = "unix")]
     fn now() -> TimeInfo {
        fn time0() -> libc::timeval {
              libc::timeval { tv_sec: 0, tv_usec: 0 }
        }
        fn seconds(t: &libc::timeval) -> f64 {
              t.tv_sec as f64 + (t.tv_usec as f64 / 1000_000.0)
        }
        let mut usage = libc::rusage {
            ru_utime: time0(),
            ru_stime: time0(),
            ru_maxrss: 0,
            ru_ixrss: 0,
            ru_idrss: 0,
            ru_isrss: 0,
            ru_minflt: 0,
            ru_majflt: 0,
            ru_nswap: 0,
            ru_inblock: 0,
            ru_oublock: 0,
            ru_msgsnd: 0,
            ru_msgrcv: 0,
            ru_nsignals: 0,
            ru_nvcsw: 0,
            ru_nivcsw: 0,
        };
        unsafe {
               libc::getrusage(libc::RUSAGE_SELF, &mut usage as *mut libc::rusage);
        };
        let wall = std::time::Instant::now() - unsafe { START_TIME.unwrap() };
        TimeInfo {
           user: seconds(&usage.ru_utime),
           system: seconds(&usage.ru_stime),
           wall: (wall.as_secs() as f64) + (wall.subsec_millis() as f64 / 1000.0),
        }
     }
}



