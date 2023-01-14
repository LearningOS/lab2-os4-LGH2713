//! Process management syscalls

use crate::config::MAX_SYSCALL_NUM;
use crate::mm::{translate_by_token, VirtAddr};
use crate::task::{
    current_user_token, exit_current_and_run_next, get_current_run_time, get_current_task_status,
    get_syscall_times, suspend_current_and_run_next, task_mmap, task_munmap, TaskStatus,
};
use crate::timer::get_time_us;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

#[derive(Clone, Copy)]
pub struct TaskInfo {
    pub status: TaskStatus,
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    pub time: usize,
}

pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

// YOUR JOB: 引入虚地址后重写 sys_get_time
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    let us = get_time_us();
    let ts_pa = translate_by_token(current_user_token(), VirtAddr::from(_ts as usize))
        .unwrap()
        .0 as *mut TimeVal;
    unsafe {
        *ts_pa = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

// CLUE: 从 ch4 开始不再对调度算法进行测试~
pub fn sys_set_priority(_prio: isize) -> isize {
    -1
}

// YOUR JOB: 扩展内核以实现 sys_mmap 和 sys_munmap
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    task_mmap(_start, _len, _port)
}

pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    task_munmap(_start, _len)
}

// YOUR JOB: 引入虚地址后重写 sys_task_info
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    let ti_pa = translate_by_token(current_user_token(), VirtAddr::from(ti as usize))
        .unwrap()
        .0 as *mut TaskInfo;
    unsafe {
        *ti_pa = TaskInfo {
            status: get_current_task_status(),
            syscall_times: get_syscall_times(),
            time: get_current_run_time(),
        }
    }
    0
}
