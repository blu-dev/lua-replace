use skyline::{hook, hooks::A64HookFunction, install_hooks};
use skyline::hooks::InlineCtx;
use skyline::nn; // for LookupSymbol
use skyline::libc;
use std::vec::Vec;
use parking_lot::Mutex;

const UNW_STEP_END: u64 = 0;
const UNW_STEP_SUCCESS: u64 = 1;

const _UA_SEARCH_PHASE: u64 = 1;

const _URC_FATAL_PHASE2_ERROR: u64 = 2;
const _URC_FATAL_PHASE1_ERROR: u64 = 3;
const _URC_HANDLER_FOUND: u64 = 6;
const _URC_INSTALL_CONTEXT: u64 = 7;

const IP_REGISTER: i32 = -1;

const LOG_LEVEL: usize = 1;

macro_rules! c_str {
    ($l:tt) => {
        [$l.as_bytes(), "\u{0}".as_bytes()].concat().as_ptr();
    };
}
#[derive(PartialEq)]
pub struct Nro {
    pub start: u64,
    pub end: u64
}

impl Nro {
    pub fn new(start_adr: u64, end_adr: u64) -> Self {
        Self {
            start: start_adr,
            end: end_adr
        }
    }

    pub fn contains(&self, addr: u64) -> bool {
        self.start < addr && addr < self.end
    }
}

pub static CURRENT_NROS: Mutex<Vec<Nro>> = Mutex::new(Vec::new());

unsafe fn ip_is_in_module(ip: u64) -> bool {
    let mut ret = false;
    let nros = CURRENT_NROS.lock();
    for nro in nros.iter() {
        ret = nro.contains(ip);
        if ret {
            break;
        }
    }
    ret
}

unsafe fn byte_search(start: *const u32, want: u32, distance: usize) -> *const u32 {
    let mut ret = 0 as *const u32;
    for x in 0..distance {
        let cur = start.offset(x as isize);
        if *cur == want {
            ret = cur;
            break;
        }
    }
    ret
}

unsafe extern "C" fn custom_personality(version: i32, actions: u64, exception_class: u64, unwind_exception: *mut u64, context: *mut u64) -> u64 {
    let mut ret = _URC_FATAL_PHASE1_ERROR;
    if version != 1 {
        println!("[lua-replace] Personality routine called in the wrong context.");
        ret = _URC_FATAL_PHASE1_ERROR;
    }
    if actions & _UA_SEARCH_PHASE != 0 {
        ret = _URC_HANDLER_FOUND;
    }
    else {
        let ip = _Unwind_GetIP(context);
        if !ip_is_in_module(ip) {
            println!("[lua-replace] Personality routine said it found an exception handler but there was no handler found.");
            ret = _URC_FATAL_PHASE2_ERROR;
        }
        else {
            let unwind_info_ptr = context.offset(0x44);
            let landing_pad = *unwind_info_ptr.offset(1) + 4;
            if !ip_is_in_module(landing_pad) {
                println!("[lua-replace] Landing pad found outside of loaded plugins.");
                ret = _URC_FATAL_PHASE2_ERROR;
            }
            else {
                _Unwind_SetIP(context, landing_pad);
                ret = _URC_INSTALL_CONTEXT;
            }
            // let new_ip = byte_search(ip as *const u32, 0xB000B1E5, 0x2000).offset(1); // 0x2000 is a lot but it should be fine for like 99% of impls
            // if new_ip == 0 as *const u32 {
            //     println!("lua-replace personality routine failed to find landing pad magic number 0xB000B1E5.");
            //     ret = _URC_FATAL_PHASE2_ERROR;
            // }
            // else {
            //     _Unwind_SetIP(context, new_ip as u64);
            //     ret = _URC_INSTALL_CONTEXT;
            // }
        }
    }
    ret
}

extern "C" {
    // Logging functions
    #[link_name = "\u{1}abort"]
    pub fn abort() -> !;
    #[link_name = "\u{1}fwrite"]
    pub fn fwrite(c_str: *const libc::c_char, size: libc::size_t, count: libc::size_t, file: *mut libc::c_void) -> libc::size_t;

    // Unwind helpers
    #[link_name = "\u{1}_Unwind_GetIP"]
    pub fn _Unwind_GetIP(context: *const u64) -> u64;
    #[link_name = "\u{1}_Unwind_SetIP"]
    pub fn _Unwind_SetIP(context: *const u64, ip: u64);
}
#[hook(replace = abort)]
pub fn abort_hook() -> ! {
        println!("[Fatal Error] abort() has been called. Flushing logger.");
        std::thread::sleep(std::time::Duration::from_millis(200));

        original!()();
    }
#[hook(replace = fwrite)]
pub fn fwrite_hook(c_str: *const libc::c_char, size: libc::size_t, count: libc::size_t, file: *mut libc::c_void) -> libc::size_t {
        unsafe {
            print!("{}", skyline::from_c_str(c_str));
        }
        original!()(c_str, size, count, file)
    }

// Functions we don't know the address of until we have the address of nnsdk
static mut STEP_WITH_DWARF: *const extern "C" fn(*mut u64, u64, *mut u64, *mut u64) -> u64= 0 as _;
static mut SET_INFO_BASED_ON_IP_REGISTER: *const extern "C" fn(*mut u64, bool) = 0 as _;

// UnwindCursor<A, R>::getReg, but we only use it for the IP register so I've simplified it
// Technically, we could call _Unwind_GetIP but that inroduces unnecessary jumping around.
// This will be fine.
unsafe fn get_ip_register(this: *const u64) -> u64 {
    let vtable = *this as *const u64;
    let get_reg: *const extern "C" fn(*const u64, i32) -> u64 = *vtable.offset(3) as _;
    let callable: extern "C" fn(*const u64, i32) -> u64 = std::mem::transmute(get_reg);
    callable(this, IP_REGISTER)
}

// We replace UnwindCursor<A, R>::step to make sure that we don't check our module image for DWARF exception information
static mut UNWIND_CURSOR_STEP: usize = 0;
#[hook(replace = UNWIND_CURSOR_STEP)]
unsafe fn step_replace(this: *mut u64) -> u64 {
    let step_with_dwarf: extern "C" fn(*mut u64, u64, *mut u64, *mut u64) -> u64 = std::mem::transmute(STEP_WITH_DWARF);
    let set_info_based_on_ip_register: extern "C" fn (*mut u64, bool) = std::mem::transmute(SET_INFO_BASED_ON_IP_REGISTER);

    // TODO: Implement the actual structures, although that is not *really* necessary
    if *(this as *const bool).offset(0x268) { // Check if we are at the bottom of the stack
        return UNW_STEP_END;
    }

    // Extract args for step_with_dwarf
    let address_space = *this.offset(1) as *mut u64;
    let ip = get_ip_register(this);
    let unwind_info = *this.offset(0x4B) as *mut u64;
    let registers = this.offset(2);

    let result = step_with_dwarf(address_space, ip, unwind_info, registers);
    if result == UNW_STEP_SUCCESS {
        let ip = get_ip_register(this); // update the current ip, as it changed with step_with_dwarf
        // TODO: Check the vector of module infos
        
        if ip_is_in_module(ip) {
            let pc_end = byte_search(ip as *const u32, 0xB000B1E5, 0x2000);
            let unwind_info_ptr = this.offset(0x44);
            *unwind_info_ptr.offset(0) = ip; // doesn't get checked for some reason
            *unwind_info_ptr.offset(1) = pc_end as u64;
            *unwind_info_ptr.offset(3) = std::mem::transmute(custom_personality as *const extern "C" fn(i32, u64, u64, *mut u64, *mut u64) -> u64);
        }
        else {
            set_info_based_on_ip_register(this, true);
            if *(this as *const bool).offset(0x268) {
                return UNW_STEP_END;
            }
        }
    }

    result
}

// When setting the IP and installing the context back onto the CPU,
// a final call to setInfoBasedOnIPRegister is performed. Obviously,
// this can't happen in our module or else our invalid DWARF sections
// will terminate the program. So we inline hook to replace the function
// it is going to call with a stub
static mut BAD_INFO_CHECK_ADDRESS: usize = 0;
#[hook(replace = BAD_INFO_CHECK_ADDRESS, inline)]
unsafe fn prevent_bad_info_check(ctx: &mut InlineCtx) {
    fn stub() {}
    let ip = get_ip_register(*ctx.registers[0].x.as_ref() as *const u64);
    if ip_is_in_module(ip) {
        *ctx.registers[8].x.as_mut() = std::mem::transmute(stub as *const fn());
    }
}

pub fn install_hooks() {
    unsafe {
        if LOG_LEVEL > 0 {
            install_hooks!(
                abort_hook,
                fwrite_hook
            );
        }
        let mut nnsdk: usize = 0;
        nn::ro::LookupSymbol(&mut nnsdk, c_str!("_Unwind_Resume"));
        assert!(nnsdk != 0, "The symbol \"_Unwind_Resume\" was not located in memory. {}", nnsdk);
        nnsdk -= 0x521960; // offset of _Unwind_Resume into nnsdk. This is a lot less likely to change than smash's offsets
        
        // TODO: Implement byte searching based off of the above offset so we only have to do this once.
        STEP_WITH_DWARF = (nnsdk + 0x51ef30) as _;
        SET_INFO_BASED_ON_IP_REGISTER = (nnsdk + 0x51ed50) as _;

        UNWIND_CURSOR_STEP = nnsdk + 0x51ec68;
        BAD_INFO_CHECK_ADDRESS = nnsdk + 0x51e5dc;
        install_hooks!(
            step_replace,
            prevent_bad_info_check
        );
    }
}