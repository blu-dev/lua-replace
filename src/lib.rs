#![feature(proc_macro_hygiene)]
use skyline::{hook, install_hook, install_hooks};
use smash::lua2cpp::*;
use smash::lib::*;
use smash::phx::Hash40;
use skyline::nro;
use skyline::nro::NroInfo;
use parking_lot::Mutex;
use unwind::CURRENT_NROS;
use std::collections::HashMap;
use std::vec::Vec;

mod unwind;
mod check;
mod rtld;
mod silent;

type ScriptBootstrapperFunc = extern "C" fn(&mut L2CAgentBase, &mut utility::Variadic);
type StatusFunc = extern "C" fn(&mut L2CFighterBase) -> L2CValue;
type SysLineControlFunc = extern "C" fn(&mut L2CFighterCommon) -> L2CValue;
type SysLineCallbackFunc = fn(&mut L2CFighterCommon);
type SysLineWeaponControlFunc = extern "C" fn(&mut L2CFighterBase) -> L2CValue;
type SysLineWeaponCallbackFunc = fn(&mut L2CFighterBase);

pub struct ScriptInfo {
    name: Hash40,
    replace: ScriptBootstrapperFunc
}

pub struct StatusInfo {
    status: LuaConst,
    condition: LuaConst,
    replace: StatusFunc
}

pub enum ScriptCategory {
    EFFECT,
    EXPRESSION,
    GAME,
    SOUND
}

impl PartialEq for ScriptInfo {
    fn eq(&self, other: &Self) -> bool {
        self.name.hash == other.name.hash
    }
}

lazy_static::lazy_static! {
    // Would use strings but really we need to allow for users to edit articles/agents they don't know the hash for
    static ref FUNC_MAP: Mutex<HashMap<String, Vec<ScriptInfo>>> = Mutex::new(HashMap::new());
    static ref NRO_MAP: Mutex<HashMap<String, unwind::Nro>> = Mutex::new(HashMap::new());
    static ref SYS_FTR_MAP: Mutex<HashMap<LuaConst, SysLineControlFunc>> = Mutex::new(HashMap::new());
    static ref SYS_WPN_MAP: Mutex<HashMap<LuaConst, SysLineWeaponControlFunc>> = Mutex::new(HashMap::new());
}

lazy_static::lazy_static! {
    pub static ref EFFECT_MAP: Mutex<HashMap<u64, Vec<ScriptInfo>>> = Mutex::new(HashMap::new());
    pub static ref EXPRESSION_MAP: Mutex<HashMap<u64, Vec<ScriptInfo>>> = Mutex::new(HashMap::new());
    pub static ref GAME_MAP: Mutex<HashMap<u64, Vec<ScriptInfo>>> = Mutex::new(HashMap::new());
    pub static ref SOUND_MAP: Mutex<HashMap<u64, Vec<ScriptInfo>>> = Mutex::new(HashMap::new());
    pub static ref STATUS_MAP: Mutex<HashMap<u64, Vec<StatusInfo>>> = Mutex::new(HashMap::new());
}

// no lazy_static because these are vecs :)
static mut FTR_CALLBACKS: Mutex<Vec<SysLineCallbackFunc>> = Mutex::new(Vec::new());
static mut WPN_CALLBACKS: Mutex<Vec<SysLineWeaponCallbackFunc>> = Mutex::new(Vec::new());

// I was never able to get this working, if somebody else wants to take a stab at it feel free.
// #[hook(replace = L2CFighterCommon_sys_line_system_init)]
// unsafe extern "C" fn sys_line_system_fighter_init_replace(fighter: &mut L2CFighterCommon) -> L2CValue {
//     use smash::hash40;
//     fighter.sv_set_function_hash(Some(std::mem::transmute(L2CFighterCommon_bind_hash_call_call_check_damage as *const extern "C" fn())), hash40("call_check_damage"));
//     fighter.sv_set_function_hash(Some(std::mem::transmute(L2CFighterCommon_bind_hash_call_call_check_attack as *const extern "C" fn())), hash40("call_check_attack"));
//     fighter.sv_set_function_hash(Some(std::mem::transmute(L2CFighterCommon_bind_hash_call_call_on_change_lr as *const extern "C" fn())), hash40("call_on_change_lr"));
//     fighter.sv_set_function_hash(Some(std::mem::transmute(L2CFighterCommon_bind_hash_call_call_leave_stop as *const extern "C" fn())), hash40("call_leave_stop"));
//     fighter.sv_set_function_hash(Some(std::mem::transmute(L2CFighterCommon_bind_hash_call_call_notify_event_gimmick as *const extern "C" fn())), hash40("call_notify_event_gimmick"));
//     fighter.sv_set_function_hash(Some(std::mem::transmute(L2CFighterCommon_bind_hash_call_call_calc_param as *const extern "C" fn())), hash40("call_calc_param"));
//     let ftr_map = SYS_FTR_MAP.lock();
//     let boma = smash::app::sv_system::battle_object_module_accessor(fighter.lua_state_agent);
//     let l2c_func: L2CValue;
//     if let Some(func) = ftr_map.get(&smash::app::utility::get_kind(boma)) {
//         l2c_func = L2CValue::new_ptr(std::mem::transmute(func));
//     }
//     else {
//         l2c_func = L2CValue::new_ptr(std::mem::transmute(L2CFighterCommon_sys_line_system_init as *const extern "C" fn()));
//     }
//     std::mem::drop(ftr_map); // i'm pretty sure that this needs to be dropped or else no one else can ever use it
//     let mut new_l2c: L2CValue = L2CValue::new();
//     copy_l2cvalue(&mut new_l2c, &l2c_func);
//     // fighter.shift(new_l2c);
//     let callable: extern "C" fn(&mut L2CFighterCommon) -> L2CValue = std::mem::transmute(l2c_func.get_ptr::<u8>());
//     callable(fighter)
// }

// this is not ideal, and I need to figure out why replacing the whole function causes a terrible amount of UB
static mut SYS_LINE_SYSTEM_INIT_SHIFT_CALL_FTR: usize = 0;
#[hook(replace = SYS_LINE_SYSTEM_INIT_SHIFT_CALL_FTR, inline)]
unsafe fn sys_line_fighter_hot_patch(ctx: &skyline::hooks::InlineCtx) {
    let fighter: &mut L2CFighterCommon = std::mem::transmute(*ctx.registers[0].x.as_ref());
    let l2c_val = *ctx.registers[1].x.as_ref() as *mut L2CValue;
    let boma = smash::app::sv_system::battle_object_module_accessor(fighter.lua_state_agent);
    let kind = smash::app::utility::get_kind(boma);
    let ftr_map = SYS_FTR_MAP.lock();
    *l2c_val = L2CValue::new_ptr(std::mem::transmute(L2CFighterCommon_sys_line_system_control_fighter as *const extern "C" fn()));
    for (agent, func) in ftr_map.iter() {
        if *agent == kind {
            *l2c_val = L2CValue::new_ptr(std::mem::transmute(*func));
        }
    }
}

// same as above
static mut SYS_LINE_SYSTEM_INIT_SHIFT_CALL_WPN: usize = 0;
#[hook(replace = SYS_LINE_SYSTEM_INIT_SHIFT_CALL_WPN, inline)]
unsafe fn sys_line_weapon_hot_patch(ctx: &skyline::hooks::InlineCtx) {
    let fighter: &mut L2CFighterBase = std::mem::transmute(*ctx.registers[0].x.as_ref());
    let l2c_val = *ctx.registers[1].x.as_ref() as *mut L2CValue;
    let boma = smash::app::sv_system::battle_object_module_accessor(fighter.lua_state_agent);
    let kind = smash::app::utility::get_kind(boma);
    let wpn_map = SYS_WPN_MAP.lock();
    *l2c_val = L2CValue::new_ptr(std::mem::transmute(L2CFighterBase_sys_line_system_control as *const extern "C" fn()));
    for (agent, func) in wpn_map.iter() {
        if *agent == kind {
            *l2c_val = L2CValue::new_ptr(std::mem::transmute(*func));
        }
    }
}

#[hook(replace = L2CFighterCommon_sys_line_system_control_fighter)]
unsafe extern "C" fn sys_line_system_control_fighter_hook(fighter: &mut L2CFighterCommon) -> L2CValue {
    let funcs = FTR_CALLBACKS.lock();
    for cb in funcs.iter() {
        cb(fighter);
    }
    original!()(fighter)
}

#[hook(replace = L2CFighterBase_sys_line_system_control)]
unsafe extern "C" fn sys_line_system_control_weapon_hook(fighter: &mut L2CFighterBase) -> L2CValue {
    let funcs = WPN_CALLBACKS.lock();
    for cb in funcs.iter() {
        cb(fighter);
    }
    original!()(fighter)
}

#[no_mangle]
pub unsafe extern "Rust" fn replace_status_script(agent: Hash40, status: LuaConst, condition: LuaConst, func: StatusFunc) {
    let (plug_start, plug_end) = skyline::info::containing_plugin(std::mem::transmute(func as *const fn()));
    if plug_start == 0 || plug_end == 0 {
        println!("[lua-replace] Failed to get plugin information for replacement script -- skipping.");
    }
    let nro = unwind::Nro::new(plug_start, plug_end);
    let mut registered_plugs = CURRENT_NROS.lock();
    if !registered_plugs.contains(&nro) {
        println!("[lua-replace] Registered user: {:X} {:X}", nro.start, nro.end);
        registered_plugs.push(nro);
    }

    let info = StatusInfo { status: status, condition: condition, replace: func };
    let mut map = STATUS_MAP.lock();
    if let Some(x) = map.get_mut(&agent.hash) {
        x.push(info); // not able to perform checking of replacements, since we can't deref lua consts
    }
    else {
        map.insert(agent.hash, vec![info]);
    }
}

#[no_mangle]
pub unsafe extern "Rust" fn replace_lua_script(agent: Hash40, script: Hash40, func: ScriptBootstrapperFunc, category: ScriptCategory) {
    let (plug_start, plug_end) = skyline::info::containing_plugin(std::mem::transmute(func as *const fn()));
    if plug_start == 0 || plug_end == 0 {
        println!("[lua-replace] Failed to get plugin information for replacement script -- skipping.");
    }
    let nro = unwind::Nro::new(plug_start, plug_end);
    let mut registered_plugs = CURRENT_NROS.lock();
    if !registered_plugs.contains(&nro) {
        println!("[lua-replace] Registered user: {:X} {:X}", nro.start, nro.end);
        registered_plugs.push(nro);
    }

    let info = ScriptInfo { name: script, replace: func };
    match category {
        ScriptCategory::EFFECT => {
            let mut map = EFFECT_MAP.lock();
            if let Some(x) = map.get_mut(&agent.hash) {
                if !x.contains(&info) {
                    x.push(info);
                }
                else {
                    println!("[lua-replace] Script has already been replaced | Agent: {:#x}, Script: {:#x}", agent.hash, info.name.hash);
                }
            }
            else {
                map.insert(agent.hash, vec![info]);
            }
        },
        ScriptCategory::EXPRESSION => {
            let mut map = EXPRESSION_MAP.lock();
            if let Some(x) = map.get_mut(&agent.hash) {
                if !x.contains(&info) {
                    x.push(info);
                }
                else {
                    println!("[lua-replace] Script has already been replaced | Agent: {:#x}, Script: {:#x}", agent.hash, info.name.hash);
                }
            }
            else {
                map.insert(agent.hash, vec![info]);
            }
        },
        ScriptCategory::GAME => {
            let mut map = GAME_MAP.lock();
            if let Some(x) = map.get_mut(&agent.hash) {
                if !x.contains(&info) {
                    x.push(info);
                }
                else {
                    println!("[lua-replace] Script has already been replaced | Agent: {:#x}, Script: {:#x}", agent.hash, info.name.hash);
                }
            }
            else {
                map.insert(agent.hash, vec![info]);
            }
        },
        ScriptCategory::SOUND => {
            let mut map = SOUND_MAP.lock();
            if let Some(x) = map.get_mut(&agent.hash) {
                if !x.contains(&info) {
                    x.push(info);
                }
                else {
                    println!("[lua-replace] Script has already been replaced | Agent: {:#x}, Script: {:#x}", agent.hash, info.name.hash);
                }
            }
            else {
                map.insert(agent.hash, vec![info]);
            }
        },
        _ => {
            panic!("Invalid script category");
        }
    }
}

#[no_mangle]
pub unsafe extern "Rust" fn replace_sys_line_fighter_script(agent: smash::lib::LuaConst, func: SysLineControlFunc) {
    let (plug_start, plug_end) = skyline::info::containing_plugin(std::mem::transmute(func as *const fn()));
    if plug_start == 0 || plug_end == 0 {
        println!("[lua-replace] Failed to get plugin information for replacment fighter sys line script -- skipping.");
    }
    let nro = unwind::Nro::new(plug_start, plug_end);
    let mut registered_plugs = CURRENT_NROS.lock();
    if !registered_plugs.contains(&nro) {
        println!("[lua-replace] Registered user: {:X} {:X}", nro.start, nro.end);
        registered_plugs.push(nro);
    }

    let mut ftr_map = SYS_FTR_MAP.lock();
    if let Some(_) = ftr_map.get_mut(&agent) {
        println!("[lua-replace] Fighter sys line script has already been replaced -- skipping.");
    }
    else {
        ftr_map.insert(agent, func);
    }
}

#[no_mangle]
pub unsafe extern "Rust" fn replace_sys_line_weapon_script(agent: smash::lib::LuaConst, func: SysLineWeaponControlFunc) {
    let (plug_start, plug_end) = skyline::info::containing_plugin(std::mem::transmute(func as *const fn()));
    if plug_start == 0 || plug_end == 0 {
        println!("[lua-replace] Failed to get plugin information for replacement weapon sys line script -- skipping.");
    }
    let nro = unwind::Nro::new(plug_start, plug_end);
    let mut registered_plugs = CURRENT_NROS.lock();
    if !registered_plugs.contains(&nro) {
        println!("[lua-replace] Registered user: {:X} {:X}", nro.start, nro.end);
        registered_plugs.push(nro);
    }

    let mut wpn_map = SYS_WPN_MAP.lock();
    if let Some(_) = wpn_map.get_mut(&agent) {
        println!("[lua-replace] Weapon sys line script has already been replaced -- skipping.");
    }
    else {
        wpn_map.insert(agent, func);
    }
}

#[no_mangle]
pub unsafe extern "Rust" fn add_sys_line_fighter_callback(func: SysLineCallbackFunc) {
    let mut funcs = FTR_CALLBACKS.lock();
    funcs.push(func);
}

#[no_mangle]
pub unsafe extern "Rust" fn add_sys_line_weapon_callback(func: SysLineWeaponCallbackFunc) {
    let mut funcs = WPN_CALLBACKS.lock();
    funcs.push(func);
}

fn nro_load_hook(info: &NroInfo) {
    match info.name {
        "common" => {
            unsafe {
                SYS_LINE_SYSTEM_INIT_SHIFT_CALL_FTR = (*info.module.ModuleObject).module_base as usize + 0x174d0;
                SYS_LINE_SYSTEM_INIT_SHIFT_CALL_WPN = (*info.module.ModuleObject).module_base as usize + 0x5ecc;
            }
            install_hooks!(
                // sv_set_function_hash_replace,
                sys_line_fighter_hot_patch,
                sys_line_weapon_hot_patch,
                sys_line_system_control_fighter_hook,
                sys_line_system_control_weapon_hook,
                // call_coroutine
            );
            // unsafe {
            //     let sym = rtld::get_symbol_by_name(info.module.ModuleObject as *const nnsdk::root::rtld::ModuleObject, "_ZN7lua2cpp16L2CFighterCommon32bind_hash_call_call_check_damageEPN3lib8L2CAgentERNS1_7utility8VariadicEPKcSt9__va_list".to_owned());
            //     println!("{:X}", sym as u64);
            // }
        },
        "item" => { }, // We don't want to register the item module since it isn't compatible yet
        name => {
            unsafe {
                silent::set_fighter_funcs(info);
            }
            // let mut nro_map = NRO_MAP.lock();
            // if nro_map.contains_key(name) {
            //     println!("[lua-replace] Loaded NRO already has entry in nro map. Maybe a bug? Updating entry.");
            // }
            // unsafe {
            //     let mod_start = (*info.module.ModuleObject).module_base;
            //     let mod_end = mod_start + 0xFFFFFFFF; // safe, should probably figure this out.

            //     nro_map.insert(String::from(name), unwind::Nro::new(mod_start, mod_end));
            // }
            // println!("[lua-replace] Registered {}", name);
        }
    }
}

fn nro_unload_hook(info: &NroInfo) {
    match info.name {
        "common" => {
            println!("[lua-replace] The \"common\" nro has been removed, may cause unintended side effects.");
        },
        "item" => {}, // We don't want to unregister item module since we didn't register it
        name => {
            unsafe {
                silent::remove_fighter_funcs(info);
            }
            // NRO_MAP.lock().remove(&String::from(name)).unwrap();
            // println!("[lua-replace] Unregistered {}", name);
        }
    }
}

#[skyline::main(name = "lua-replace")]
pub fn main() {
    unwind::install_hooks();
    // check::install_hooks();
    nro::add_hook(nro_load_hook).unwrap();
    nro::add_unload_hook(nro_unload_hook).unwrap();
}
