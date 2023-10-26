//! Process management syscalls
use crate::{
    config::MAX_SYSCALL_NUM,
    task::{
        change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, TaskStatus, get_current_mem_set, fetch_curr_task_control_block,
    }, 
    mm::{VirtAddr, MapPermission, VirtPageNum, write_byte_buffer}, timer::{get_time_us, get_time_ms},
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");

    let us = get_time_us();
    let ans = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    write_byte_buffer::<TimeVal>(ans, _ts);
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");

    let curr_task_cb = fetch_curr_task_control_block();
    let ans = TaskInfo {
        status: curr_task_cb.task_status,
        syscall_times: curr_task_cb.task_syscall_times,
        time: get_time_ms(),
    };
    write_byte_buffer::<TaskInfo>(ans, _ti);
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap");

    let va_start = VirtAddr::from(_start);
    let va_end = VirtAddr::from(_start + _len);

    if _len == 0 {
        return 0;
    }
    if !va_start.aligned(){
        trace!("mmap failed: _start 0x{:08x} not aligned", _start);
        return -1;
    }
    if _port & !0x7 != 0 || _port * 0x7 == 0 {
        return -1;
    }

    let start_vpn: VirtPageNum = va_start.floor();
    let end_vpn: VirtPageNum = va_end.ceil();
    let vpn_range = usize::from(start_vpn) ..usize::from(end_vpn);
    
    let mem_set = get_current_mem_set();

    for vpn in vpn_range.clone() {
        if let Some(pte) = mem_set.translate(VirtPageNum::from(vpn)) {
            if pte.is_valid() {
                return -1;
            }
        }
    }


    let permission = (_port as u8) << 1 | (1 << 4);
    let permission = MapPermission::from_bits_truncate(permission);

    mem_set.insert_framed_area(va_start, va_end, permission);
    
    for vpn in vpn_range.clone() {
        match mem_set.translate(VirtPageNum::from(vpn)) {
            None => {
                trace!("mmap failed at last");
                return -1;
            },
            Some(pte) => {
                if !pte.is_valid() {
                    trace!("mmap failed at last");
                    return -1;
                }
            }
        }
    }
    
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap");

    let va_start = VirtAddr::from(_start);
    let va_end = VirtAddr::from(_start + _len);

    if _len == 0 {
        return 0;
    }
    if !va_start.aligned(){
        return -1;
    }

    let start_vpn: VirtPageNum = va_start.floor();
    let end_vpn: VirtPageNum = va_end.ceil();
    let vpn_range = usize::from(start_vpn) ..usize::from(end_vpn);
    
    let mem_set = get_current_mem_set();

    for vpn in vpn_range.clone(){
        match mem_set.translate(VirtPageNum::from(vpn)) {
            None => return -1,
            Some(pte) => {
                if !pte.is_valid() {
                    return -1;
                }
            }
        }
    }

    for vpn in vpn_range.clone() {
        mem_set.unmap_vpn(VirtPageNum::from(vpn));
    }

    for vpn in vpn_range.clone() {
        if let Some(pte) = mem_set.translate(VirtPageNum::from(vpn)) {
            if pte.is_valid() {
                trace!("munmap failed at last");
                return -1;
            }
        }
    }

    0
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}