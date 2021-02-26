// tysm Thog for writing the excellent oss-rtld that I adapted for this
// https://github.com/Thog/oss-rtld

use nnsdk::root::Elf64_Sym;
use skyline::nn;
use skyline::libc;

macro_rules! c_str {
    ($l:tt) => {
        [$l.as_bytes(), "\u{0}".as_bytes()].concat().as_ptr();
    };
}

unsafe fn gen_rtld_elf_hash(mut name: *const libc::c_char) -> u32 {
    let mut ret: u32 = 0;
    let mut g: u32 = 0;

    loop {
        if *name == 0 {
            break;
        }
        ret = (ret << 4) + (*name as u32);
        name = name.offset(1);
        g = ret & 0xf0000000;
        if g != 0 {
            ret ^= g >> 24;
        }
        ret &= !g;
    }
    ret
}

unsafe fn rtld_strcmp(mut s1: *const libc::c_char, mut s2: *const libc::c_char) -> i32 {
    loop {
        if !(*s1 != 0 && (*s1 == *s2)) { break; }
        s1 = s1.offset(1);
        s2 = s2.offset(1);
    }
    (*(s1 as *const u8) - *(s2 as *const u8)) as i32
}

pub unsafe fn get_symbol_by_name(module_object: *const nnsdk::root::rtld::ModuleObject, name: String) -> *const Elf64_Sym {
    let _name = name.as_str();
    let hash = gen_rtld_elf_hash(c_str!(_name));
    let mut i = *(*module_object).hash_bucket.offset((hash % ((*module_object).hash_nbucket_value as u32)) as isize);
    loop {
        if i == 0 { break;}
        let sym = &mut *(*module_object).dynsym.offset(i as isize);
        let mut is_common: bool = true;
        if sym.st_shndx != 0 {
            is_common = sym.st_shndx == 0xFFF2;
        } 
        if !is_common && rtld_strcmp(c_str!(_name), (*module_object).dynstr.offset(sym.st_name as isize)) == 0 {
            return (*module_object).dynsym.offset(i as isize);
        }
        i = *(*module_object).hash_chain.offset(i as isize);
    }
    return 0 as *const Elf64_Sym;
}