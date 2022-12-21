use super::address::PhysPageNum;
use crate::{config::MEMORY_END, mm::address::PhysAddr, sync::UPSafeCell};
use alloc::vec::Vec;
use core::fmt::{self, Debug, Formatter};
use lazy_static::*;

pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
        // 清空页
        let bytes_array = ppn.get_bytes_array();
        for i in bytes_array {
            *i = 0;
        }
        Self { ppn }
    }
}

impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FrameTracker:PPN={:#x}", self.ppn.0))
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}

// 物理页帧管理器
trait FrameAllocator {
    fn new() -> Self;
    // 以物理页号为单位进行物理页帧的分配和回收
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}

// 栈式物理页帧管理策略
pub struct StackFrameAllocator {
    current: usize,       // 空闲内存的起始物理页号
    end: usize,           // 空闲内存的结束物理页号
    recycled: Vec<usize>, // 保存被回收的物理页号
}

impl StackFrameAllocator {
    // 初始化可用的物理页号区间
    pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        self.current = l.0;
        self.end = r.0;
    }
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }
    // 页帧分配
    fn alloc(&mut self) -> Option<PhysPageNum> {
        // 若recycled里有回收页号则直接弹出使用
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else if self.current == self.end
        // 物理页号已经耗尽
        {
            None
        } else {
            self.current += 1; // +1 代表current已分配
            Some((self.current - 1).into()) // current-1代表已分配的物理页号，使用into将current转换回物理页号
        }
    }
    // 页帧回收
    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;
        // 检查回收页面的合法性
        if ppn >= self.current/* 页面之前一定被分配出去过 */ || self.recycled.iter().any(|v| *v == ppn)
        /*物理页号不能在recycled里找到*/
        {
            panic!("Frame ppn={:#X} has not been allocated!", ppn);
        }
        // 回收压栈
        self.recycled.push(ppn);
    }
}

// 创建全局实例
type FrameAllocatorImpl = StackFrameAllocator;
lazy_static! {
    pub static ref FRAME_ALLOCATOR: UPSafeCell<FrameAllocatorImpl> =
        unsafe { UPSafeCell::new(FrameAllocatorImpl::new()) };
}

// 物理页帧全局管理器初始化
pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR.exclusive_access().init(
        PhysAddr::from(ekernel as usize).ceil(), // 起始位置向上取整
        PhysAddr::from(MEMORY_END).floor(),      // 结束位置向下取整
    );
}

// 公开给其他模块使用的帧分配/回收接口
pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .exclusive_access()
        .alloc()
        .map(FrameTracker::new)
}

fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn);
}
