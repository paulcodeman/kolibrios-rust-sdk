use crate::system::debug_write;
use crate::{sys, throw_new};
use core::alloc::Layout;
use core::mem::size_of;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicBool, Ordering};

extern crate alloc;

const PAGE_SIZE: usize = 4096;
static mut MAIN_SECTOR: usize = 0;

#[derive(Clone, Copy, PartialEq)] // Добавлено PartialEq для сравнения в дальнейшем
enum Sign {
    Dead = 0,
    Active = 1,
    Free = 2,
}

#[derive(Clone, Copy)]
struct SectorHeader {
    pub size: usize,
    pub size_left: usize,
}

#[derive(Clone, Copy)]
struct BlockHeader {
    pub sign: Sign,
    pub size: usize,
}

static HEAP_INIT: AtomicBool = AtomicBool::new(false);

pub fn init() {
    if !HEAP_INIT.swap(true, Ordering::Relaxed) {
        unsafe {
            sys::init_heap();
            MAIN_SECTOR = sys::alloc(PAGE_SIZE) as usize;
            // Инициализация первого сектора
            let sec_hdr = MAIN_SECTOR as *mut SectorHeader;
            *sec_hdr = SectorHeader {
                size: PAGE_SIZE,
                size_left: PAGE_SIZE - size_of::<SectorHeader>(),
            };
        }
    }
}

pub fn malloc(layout: Layout) -> *mut u8 {
    let size = layout.size();
    let align = layout.align(); // Использование align из layout

    unsafe {
        let mut found_block: *mut BlockHeader = null_mut();
        let mut found_block_size = 0;

        for i in 0..PAGE_SIZE / 4 {
            let addr = *((MAIN_SECTOR + i * 4) as *const u32) as *const u8;

            if (addr as usize) != 0 {
                let sec = addr;
                let hdr = *(addr as *const SectorHeader); // Убрано лишнее mut
                let sec_start_blocks = (sec as usize) + size_of::<SectorHeader>();

                if hdr.size_left >= size {
                    let mut j = sec_start_blocks;

                    while j <= sec_start_blocks + hdr.size - size_of::<BlockHeader>() {
                        let block = *(j as *const BlockHeader);

                        match block.sign {
                            Sign::Active => {
                                j += size_of::<BlockHeader>() + block.size;
                            }

                            Sign::Free => {
                                // Проверяем выравнивание блока
                                let aligned_addr = (j + size_of::<BlockHeader>()) as *mut u8;
                                let aligned_addr_value = aligned_addr as usize;
                                if aligned_addr_value % align == 0 && block.size >= size && (found_block_size == 0 || block.size < found_block_size) {
                                    found_block = j as *mut BlockHeader;
                                    found_block_size = block.size;
                                }
                                j += size_of::<BlockHeader>() + block.size;
                            }

                            Sign::Dead => {
                                if j + size + size_of::<BlockHeader>()
                                    <= sec_start_blocks + hdr.size
                                    && (found_block_size == 0 || (sec_start_blocks + hdr.size - j) < found_block_size)
                                {
                                    found_block = j as *mut BlockHeader;
                                    found_block_size = sec_start_blocks + hdr.size - j;
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }

        if found_block_size > 0 {
            if found_block_size >= size + size_of::<BlockHeader>() {
                let new_block = (found_block as *mut u8).add(size + size_of::<BlockHeader>()) as *mut BlockHeader;
                *new_block = BlockHeader {
                    sign: Sign::Free,
                    size: found_block_size - size - size_of::<BlockHeader>(),
                };
                (*found_block).size = size;
            }

            (*found_block).sign = Sign::Active;
            let addr = (found_block as *mut u8).add(size_of::<BlockHeader>());

            for i in 0..PAGE_SIZE / 4 {
                let sec_addr = *((MAIN_SECTOR + i * 4) as *const u32) as *const u8;
                if (sec_addr as usize) != 0 {
                    let sec = sec_addr;
                    let mut hdr = *(sec as *const SectorHeader);
                    if (sec as usize) < (addr as usize) && (addr as usize) < (sec as usize) + hdr.size {
                        hdr.size_left -= size + size_of::<BlockHeader>();
                        break;
                    }
                }
            }

            return addr;
        } else {
            // Выделение новой страницы
            let sec_size = size + PAGE_SIZE - size % PAGE_SIZE;
            let new_sec = sys::alloc(sec_size) as *mut u8;
            if new_sec == null_mut() {
                return null_mut();
            }
            let sec_hdr = new_sec as *mut SectorHeader;
            *sec_hdr = SectorHeader {
                size: sec_size,
                size_left: sec_size - size_of::<SectorHeader>(),
            };

            // Добавление адреса новой страницы в таблицу страниц
            for i in 0..PAGE_SIZE / 4 {
                let addr = (MAIN_SECTOR + i * 4) as *mut u32;
                if *addr == 0 {
                    *addr = new_sec as u32;
                    break;
                }
            }

            let new_block = new_sec.add(size_of::<SectorHeader>()) as *mut BlockHeader;
            *new_block = BlockHeader {
                sign: Sign::Active,
                size: size,
            };
            (*sec_hdr).size_left -= size + size_of::<BlockHeader>();
            return (new_block as *mut u8).add(size_of::<BlockHeader>()); // Возвращаем адрес данных, а не заголовка
        }
    }
}

fn free(block: *const u8) {
    unsafe {
        let block_hdr = (block.sub(size_of::<BlockHeader>())) as *mut BlockHeader;
        (*block_hdr).sign = Sign::Free;

        // Поиск соседних свободных блоков и объединение
        let current_block = block_hdr; // Убрано лишнее объявление переменной
        loop {
            let next_block = (current_block as *mut u8)
                .add(size_of::<BlockHeader>() + (*current_block).size)
                as *mut BlockHeader;
            if (*next_block).sign == Sign::Free {
                (*current_block).size += size_of::<BlockHeader>() + (*next_block).size;
                (*next_block).sign = Sign::Dead; // Помечаем следующий блок как неиспользуемый
            } else {
                break;
            }
        }

        // Обновление size_left в заголовке сектора
        for i in 0..PAGE_SIZE / 4 {
            let sec_addr = *((MAIN_SECTOR + i * 4) as *const u32) as *const u8;
            if (sec_addr as usize) != 0 {
                let sec = sec_addr;
                let mut hdr = *(sec as *const SectorHeader);
                if (sec as usize) < (block as usize) && (block as usize) < (sec as usize) + hdr.size {
                    hdr.size_left += (*block_hdr).size + size_of::<BlockHeader>();
                    break;
                }
            }
        }

        // Освобождение страницы, если она полностью свободна
        for i in 0..PAGE_SIZE / 4 {
            let sec_addr = *((MAIN_SECTOR + i * 4) as *const u32) as *const u8;
            if (sec_addr as usize) != 0 {
                let sec = sec_addr;
                let hdr = *(sec as *const SectorHeader);
                if hdr.size_left == hdr.size - size_of::<SectorHeader>() {
                    sys::free(sec as *mut u8);
                    *((MAIN_SECTOR + i * 4) as *mut u32) = 0; // Удаляем адрес сектора из таблицы
                    break;
                }
            }
        }
    }
}

struct GlobalAlloc;

unsafe impl alloc::alloc::GlobalAlloc for GlobalAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.align() > PAGE_SIZE {
            throw_new!("Align is too big");
            return null_mut();
        }

        init();
        malloc(layout) // Передаем layout в malloc
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _: Layout) {
        free(ptr)
    }
}

#[global_allocator]
static ALLOC: GlobalAlloc = GlobalAlloc;