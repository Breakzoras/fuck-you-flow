//! Windows job object: every sidecar process is added to one job with
//! "kill on close", so if Lalia dies for any reason the engine dies with it and
//! never lingers holding the GPU.

#[cfg(windows)]
mod imp {
    use once_cell::sync::Lazy;
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation, SetInformationJobObject,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_SET_QUOTA, PROCESS_TERMINATE};

    struct Job(HANDLE);
    unsafe impl Send for Job {}
    unsafe impl Sync for Job {}

    static JOB: Lazy<Option<Job>> = Lazy::new(|| unsafe {
        let job = CreateJobObjectW(None, None).ok()?;
        let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let ok = SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &info as *const _ as *const core::ffi::c_void,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        );
        if ok.is_err() {
            let _ = CloseHandle(job);
            return None;
        }
        Some(Job(job))
    });

    pub fn assign(pid: u32) {
        unsafe {
            let Some(job) = JOB.as_ref() else { return };
            if let Ok(proc_) = OpenProcess(PROCESS_SET_QUOTA | PROCESS_TERMINATE, false, pid) {
                if let Err(e) = AssignProcessToJobObject(job.0, proc_) {
                    tracing::warn!("AssignProcessToJobObject failed: {e}");
                }
                let _ = CloseHandle(proc_);
            }
        }
    }
}

pub fn assign_to_app_job(pid: u32) {
    #[cfg(windows)]
    imp::assign(pid);
    #[cfg(not(windows))]
    let _ = pid;
}
