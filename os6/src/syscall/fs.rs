//! File and filesystem-related syscalls

use crate::mm::translated_byte_buffer;
use crate::mm::translated_str;
use crate::mm::translated_refmut;
use crate::task::current_user_token;
use crate::task::current_task;
use crate::fs::{open_file, linkat, unlinkat};
use crate::fs::OpenFlags;
use crate::fs::Stat;
use crate::mm::UserBuffer;
use alloc::sync::Arc;
use crate::mm::{PageTable, VirtAddr, PhysAddr};
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        ) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.read(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        ) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(
        path.as_str(),
        OpenFlags::from_bits(flags).unwrap()
    ) {
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

// YOUR JOB: 扩展 easy-fs 和内核以实现以下三个 syscall
pub fn sys_fstat(_fd: usize, _st: *mut Stat) -> isize {
    let current_task = current_task().unwrap();
    let token = current_user_token();
    let page_table = PageTable::from_token(token);
    let va = VirtAddr::from(_st as usize);
    let vpn = va.floor();
    let ppn = page_table.translate(vpn).unwrap().ppn();
    let offset = va.page_offset();
    let pa: PhysAddr = ppn.into();
    let task_inner = current_task.inner_exclusive_access();
    if let Some(file) = &task_inner.fd_table[_fd]{
        unsafe {
            file.stat(&mut *((pa.0 + offset) as *mut Stat))
        }
    }else {
        -1
    }
}

pub fn sys_linkat(_old_name: *const u8, _new_name: *const u8) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let old_name = translated_str(token, _old_name);
    let new_name = translated_str(token, _new_name);
    linkat(&old_name, &new_name)
}

pub fn sys_unlinkat(_name: *const u8) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let name = translated_str(token, _name);
    unlinkat(&name)
}
