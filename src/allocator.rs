use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicBool, Ordering};

use esp_alloc::{HEAP, MemoryCapability};

static USE_PSRAM: AtomicBool = AtomicBool::new(false);

pub struct DualHeapAllocator;

#[global_allocator]
pub static ALLOCATOR: DualHeapAllocator = DualHeapAllocator;

unsafe impl GlobalAlloc for DualHeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let caps = if USE_PSRAM.load(Ordering::Acquire) {
            MemoryCapability::External.into()
        } else {
            MemoryCapability::Internal.into()
        };
        // SAFETY: delegated to esp_alloc which upholds all invariants.
        unsafe { HEAP.alloc_caps(caps, layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: delegated to esp_alloc.
        unsafe { GlobalAlloc::dealloc(&HEAP, ptr, layout) }
    }
}

#[inline]
pub fn use_psram_heap() {
    USE_PSRAM.store(true, Ordering::Release);
}

#[inline]
pub fn use_sram_heap() {
    USE_PSRAM.store(false, Ordering::Release);
}

#[inline]
pub fn set_psram_alloc(enabled: bool) {
    USE_PSRAM.store(enabled, Ordering::Release);
}
