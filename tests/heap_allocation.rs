#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use alloc::{boxed::Box, vec::Vec};
use rust_os::{
    allocator::{self, HEAP_SIZE},
    memory::{self, BootInfoFrameAllocator},
};
use x86_64::VirtAddr;

extern crate alloc;

use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    rust_os::init();

    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(physical_memory_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    test_main();
    loop {}
}

/// Test simple heap allocation
#[test_case]
fn simple_allocation() {
    let heap_value1 = Box::new(42);
    let heap_value2 = Box::new(8245);
    assert_eq!(*heap_value1, 42);
    assert_eq!(*heap_value2, 8245)
}

/// Test vec allocation (and, due to growing size, reallocation)
#[test_case]
fn large_vec() {
    let n = 1024;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), n * (n - 1) / 2);
}

/// Test allocation and dellocation
#[test_case]
fn many_boxes() {
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}

/// Stress test the bump allocator
#[test_case]
#[should_panic]
fn many_boxes_long_lived() {
    let long_lived = Box::new(1);
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
    assert_eq!(*long_lived, 1);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info)
}
