use crate::system::debug_write;
use crate::{sys, throw_new};
use core::alloc::Layout;
use core::mem::size_of;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicBool, Ordering};

extern crate alloc;

const PAGE_SIZE: usize = 4096;
// Blok başlıkları ve veri adresleri bu bayt sınırına hizalanır.
const GRAIN: usize = 8;
const SECTOR_HEADER: usize = align_up(size_of::<SectorHeader>(), GRAIN);
const BLOCK_HEADER: usize = align_up(size_of::<BlockHeader>(), GRAIN);

// Sektör listesinin başı. 0 ise hiç sektör yok.
// Durum .data içinde kalmalı: link.x .bss'e yalnız *(.bss) topluyor, .bss.* bölümleri FILE_END'in arkasına
// düşüyor; FILE_END ise uygulama yolu tamponunun başlangıcı.
#[link_section = ".data"]
static mut FIRST_SECTOR: usize = 0;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
enum Sign {
    Active = 1,
    Free = 2,
}

// raw: sys::alloc'un döndürdüğü ham adres (sys::free için).
// next: sonraki sektör. total: başlık dahil toplam boyut. live: kullanımdaki blok sayısı.
#[repr(C)]
#[derive(Clone, Copy)]
struct SectorHeader {
    raw: usize,
    next: usize,
    total: usize,
    live: usize,
}

// Sektör, başlık ve veriden oluşan bloklara tam olarak bölünmüştür: ilk blok sektör
// başlığından hemen sonra başlar, son blok sektörün sonunda biter.
#[repr(C)]
#[derive(Clone, Copy)]
struct BlockHeader {
    sign: Sign,
    size: usize,
}

const fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

#[link_section = ".data"]
static HEAP_INIT: AtomicBool = AtomicBool::new(false);

pub fn init() {
    if !HEAP_INIT.swap(true, Ordering::Relaxed) {
        unsafe {
            sys::init_heap();
        }
    }
}

pub fn malloc(layout: Layout) -> *mut u8 {
    let size = align_up(layout.size().max(GRAIN), GRAIN);
    let align = layout.align().max(GRAIN);
    if align > PAGE_SIZE {
        return null_mut();
    }
    init();

    unsafe {
        // Sığan en küçük serbest blok seçilir. Tarama sırasında ardışık serbest bloklar birleştirilir.
        let mut best_sector = 0;
        let mut best_block = 0;
        let mut best_data = 0;
        let mut best_size = usize::MAX;

        let mut sector = FIRST_SECTOR;
        while sector != 0 {
            let hdr = *(sector as *const SectorHeader);
            let sector_end = sector + hdr.total;
            let mut block = sector + SECTOR_HEADER;

            // İmza baytı ham olarak okunur: kullanıcı taşması bir başlığın imzasını bozarsa geçersiz bir enum üretilmez.
            while block + BLOCK_HEADER <= sector_end {
                let mut block_size = (*(block as *const BlockHeader)).size;
                if *(block as *const u8) == Sign::Free as u8 {
                    loop {
                        let next = block + BLOCK_HEADER + block_size;
                        if next + BLOCK_HEADER > sector_end || *(next as *const u8) != Sign::Free as u8 {
                            break;
                        }
                        block_size += BLOCK_HEADER + (*(next as *const BlockHeader)).size;
                    }
                    put_block(block, Sign::Free, block_size);

                    let end = block + BLOCK_HEADER + block_size;
                    let data = data_at(block, align);
                    if fits(data, size, end) && block_size < best_size {
                        best_sector = sector;
                        best_block = block;
                        best_data = data;
                        best_size = block_size;
                    }
                }
                block += BLOCK_HEADER + block_size;
            }
            sector = hdr.next;
        }

        if best_block != 0 {
            let end = best_block + BLOCK_HEADER + best_size;
            return carve(best_sector, best_block, end, best_data, size);
        }

        // Sığan serbest blok yok: yeni sektör aç.
        let sector = new_sector(size, align);
        if sector == 0 {
            return null_mut();
        }
        let block = sector + SECTOR_HEADER;
        let end = sector + (*(sector as *const SectorHeader)).total;
        let data = data_at(block, align);
        if !fits(data, size, end) {
            release(sector);
            return null_mut();
        }
        carve(sector, block, end, data, size)
    }
}

// `data` adresinden başlayan `size` baytlık veri `end`'i aşmıyorsa doğru. Toplam yerine fark kullanılır; böylece 32 bitte taşmaz.
fn fits(data: usize, size: usize, end: usize) -> bool {
    data <= end && size <= end - data
}

fn free(block: *const u8) {
    unsafe {
        let data = block as usize;
        // Ayırıcının verdiği her veri adresi GRAIN'e hizalıdır; hizasız adresin başlığı okunmadan reddedilir.
        if !data.is_multiple_of(GRAIN) {
            throw_new!("free: adres hizasız");
            return;
        }
        let sector = sector_of(data);
        if sector == 0 {
            throw_new!("free: adres ayırıcıya ait değil");
            return;
        }
        let header = data - BLOCK_HEADER;
        // İmza bir enum'dur; geçersiz bit deseni okunmasın diye yalnız ham bayt olarak okunur.
        if *(header as *const u8) != Sign::Active as u8 {
            throw_new!("free: blok zaten serbest");
            return;
        }
        let size = (*(header as *const BlockHeader)).size;
        put_block(header, Sign::Free, size);

        let sector_hdr = sector as *mut SectorHeader;
        (*sector_hdr).live -= 1;
        if (*sector_hdr).live == 0 {
            release(sector);
        }
    }
}

// `block` serbest bloğunu (sonu `end`) kullanarak `data` adresinde `size` baytlık blok ayırır.
// Önde ve arkada kalan yer bir blok başlığı alacak kadar büyükse serbest blok olarak yazılır;
// değilse ayrılan bloğa eklenir.
unsafe fn carve(sector: usize, block: usize, end: usize, data: usize, size: usize) -> *mut u8 {
    let header = data - BLOCK_HEADER;
    if header > block {
        put_block(block, Sign::Free, header - block - BLOCK_HEADER);
    }
    let rest = end - (data + size);
    let used = if rest >= BLOCK_HEADER {
        put_block(data + size, Sign::Free, rest - BLOCK_HEADER);
        size
    } else {
        end - data
    };
    put_block(header, Sign::Active, used);
    (*(sector as *mut SectorHeader)).live += 1;
    data as *mut u8
}

// Veri adresini `align`'a hizalar. Blok başları ve sektör 8'e hizalı olduğundan önde kalan boşluk ya sıfırdır
// ya da en az bir blok başlığı kadardır; carve bu boşluğu serbest blok olarak yazar.
fn data_at(block: usize, align: usize) -> usize {
    align_up(block + BLOCK_HEADER, align)
}

// Yeni sektör ayırır ve tüm alanını tek bir serbest blok olarak yazar.
unsafe fn new_sector(size: usize, align: usize) -> usize {
    // En kötü durumda veri, sektör başlığından sonra iki blok başlığı ve iki hizalama adımı uzaktadır.
    let need = SECTOR_HEADER + 2 * BLOCK_HEADER + 2 * align + size;
    // Sistem sayfa katı ayırır. sys::alloc sonucu 8'e hizalı olmayabilir; bu yüzden GRAIN bayt pay sayfa yuvarlamasına dahil edilir.
    let alloc_size = align_up((need + GRAIN).max(PAGE_SIZE), PAGE_SIZE);
    let raw = sys::alloc(alloc_size) as usize;
    if raw == 0 {
        return 0;
    }
    let sector = align_up(raw, GRAIN);
    // Sektör raw'dan ileri başlıyorsa toplam GRAIN bayt eksik sayılır; sınır ayrılan alanı aşmaz, boyut 8'in katı kalır.
    let total = if sector == raw { alloc_size } else { alloc_size - GRAIN };
    *(sector as *mut SectorHeader) = SectorHeader {
        raw,
        next: FIRST_SECTOR,
        total,
        live: 0,
    };
    FIRST_SECTOR = sector;
    put_block(sector + SECTOR_HEADER, Sign::Free, total - SECTOR_HEADER - BLOCK_HEADER);
    sector
}

// `data` adresini içeren sektörün başlık adresini döndürür; yoksa 0.
unsafe fn sector_of(data: usize) -> usize {
    let mut sector = FIRST_SECTOR;
    while sector != 0 {
        let hdr = *(sector as *const SectorHeader);
        if sector + SECTOR_HEADER + BLOCK_HEADER <= data && data < sector + hdr.total {
            return sector;
        }
        sector = hdr.next;
    }
    0
}

// Boş sektörü listeden çıkarır ve belleğini sisteme geri verir.
unsafe fn release(sector: usize) {
    let hdr = *(sector as *const SectorHeader);
    if FIRST_SECTOR == sector {
        FIRST_SECTOR = hdr.next;
    } else {
        let mut prev = FIRST_SECTOR;
        while (*(prev as *const SectorHeader)).next != sector {
            prev = (*(prev as *const SectorHeader)).next;
        }
        (*(prev as *mut SectorHeader)).next = hdr.next;
    }
    sys::free(hdr.raw as *const u8);
}

unsafe fn put_block(block: usize, sign: Sign, size: usize) {
    *(block as *mut BlockHeader) = BlockHeader { sign, size };
}

struct GlobalAlloc;

unsafe impl alloc::alloc::GlobalAlloc for GlobalAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.align() > PAGE_SIZE {
            throw_new!("Align is too big");
            return null_mut();
        }

        malloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _: Layout) {
        free(ptr)
    }
}

#[global_allocator]
static ALLOC: GlobalAlloc = GlobalAlloc;
