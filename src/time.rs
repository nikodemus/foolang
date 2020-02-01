use std::hash::{Hash, Hasher};
use std::ops::Add;
use std::ops::Sub;

static mut START_TIME: Option<std::time::Instant> = None;

#[derive(Debug, Clone, Copy)]
pub struct TimeInfo {
    pub user: f64,
    pub system: f64,
    pub real: f64,
}

impl PartialEq for TimeInfo {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for TimeInfo {}

impl Hash for TimeInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self, state);
    }
}

impl Add for TimeInfo {
    type Output = TimeInfo;
    fn add(self, other: TimeInfo) -> TimeInfo {
        TimeInfo {
            user: self.user + other.user,
            system: self.system + other.system,
            real: self.real + other.real,
        }
    }
}

impl Sub for TimeInfo {
    type Output = TimeInfo;
    fn sub(self, other: TimeInfo) -> TimeInfo {
        TimeInfo {
            user: self.user - other.user,
            system: self.system - other.system,
            real: self.real - other.real,
        }
    }
}

fn start_time() -> std::time::Instant {
    match unsafe { START_TIME } {
        Some(t) => t,
        None => panic!("TimeInfo not initialized!"),
    }
}

fn wallclock() -> f64 {
    let wall = std::time::Instant::now() - start_time();
    (wall.as_secs() as f64) + (wall.subsec_millis() as f64 / 1000.0)
}

impl TimeInfo {
    pub fn init() {
        unsafe { START_TIME = Some(std::time::Instant::now()) };
    }
    #[cfg(target_family = "windows")]
    pub fn now() -> TimeInfo {
        use winapi::shared::minwindef::FILETIME;
        use winapi::um::processthreadsapi::GetCurrentProcess;
        use winapi::um::processthreadsapi::GetProcessTimes;

        fn filetime_zero() -> FILETIME {
            FILETIME {
                dwHighDateTime: 0,
                dwLowDateTime: 0,
            }
        }

        let (kernel, user) = unsafe {
            let handle = GetCurrentProcess();
            let mut created = filetime_zero();
            let mut exited = filetime_zero();
            let mut kernel = filetime_zero();
            let mut user = filetime_zero();
            let res = GetProcessTimes(
                handle,
                &mut created as *mut FILETIME,
                &mut exited as *mut FILETIME,
                &mut kernel as *mut FILETIME,
                &mut user as *mut FILETIME,
            );
            if res == 0 {
                panic!("GetProcessTimes failed for current process.");
            }
            (kernel, user)
        };
        fn seconds(t: FILETIME) -> f64 {
            // A file time is a 64-bit value that represents the number
            // of 100-nanosecond intervals that have elapsed since
            // 12:00 A.M. January 1, 1601 Coordinated Universal Time (UTC).
            let high = (t.dwHighDateTime as u64) << 32;
            let low = t.dwLowDateTime as u64;
            let total = high | low;
            // Microsecond resolution works OK into floats.
            (total / 10) as f64 / 1000_000.0
        }
        TimeInfo {
            user: seconds(user),
            system: seconds(kernel),
            real: wallclock(),
        }
    }
    #[cfg(target_family = "unix")]
    pub fn now() -> TimeInfo {
        fn time0() -> libc::timeval {
            libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            }
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
        TimeInfo {
            user: seconds(&usage.ru_utime),
            system: seconds(&usage.ru_stime),
            real: wallclock(),
        }
    }
}
