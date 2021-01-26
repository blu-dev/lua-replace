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

type ScriptBootstrapperFunc = extern "C" fn(&mut L2CFighterCommon, &mut utility::Variadic);
type SysLineControlFunc = extern "C" fn(&mut L2CFighterCommon) -> L2CValue;
type SysLineCallbackFunc = fn(&mut L2CFighterCommon);
type SysLineWeaponControlFunc = extern "C" fn(&mut L2CFighterBase) -> L2CValue;
type SysLineWeaponCallbackFunc = fn(&mut L2CFighterBase);

struct ScriptInfo {
    name: Hash40,
    replace: ScriptBootstrapperFunc
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

// no lazy_static because these are vecs :)
static mut FTR_CALLBACKS: Mutex<Vec<SysLineCallbackFunc>> = Mutex::new(Vec::new());
static mut WPN_CALLBACKS: Mutex<Vec<SysLineWeaponCallbackFunc>> = Mutex::new(Vec::new());

unsafe fn get_module_name(function: ScriptBootstrapperFunc) -> Option<String> {
    let map = NRO_MAP.lock();
    let mut ret: Option<String> = None;

    let func: u64 = std::mem::transmute(function as *const fn());

    for (fighter, nro) in map.iter() {
        if nro.start < func && func < nro.end {
            ret = Some(fighter.clone());
            break;
        }
    }
    ret
}

#[hook(replace = L2CAgent_sv_set_function_hash)]
unsafe extern "C" fn sv_set_function_hash_replace(agent: &mut L2CAgent, mut function: ScriptBootstrapperFunc, hash: Hash40) {
    let fighter = get_module_name(function).unwrap_or("".to_owned());
    let mut test_hash: Hash40 = Hash40::new_raw(0);
    if check::HASH.is_none() && fighter != "" {
        test_hash = Hash40::new(fighter.as_str());
    }
    else if check::HASH.is_some() {
        test_hash = check::HASH.unwrap();
    }
    let funcs = FUNC_MAP.lock();

    for (agent, scripts) in funcs.iter() {
        let mut breakable = false;
        if Hash40::new(agent.as_str()).hash == test_hash.hash {
            for script in scripts.iter() {
                if script.name.hash == hash.hash {
                    function = script.replace;
                    breakable = true;
                    break;
                }
            }
        }
        if breakable {
            break;
        }
    }
    original!()(agent, function, hash)
}

extern "C" {
    #[link_name = "\u{1}_ZN3lib8L2CValueC1ERKS0_"]
    fn copy_l2cvalue(dst: &mut L2CValue, src: &L2CValue);

    #[link_name = "\u{1}_ZN7lua2cpp16L2CFighterCommon32bind_hash_call_call_check_damageEPN3lib8L2CAgentERNS1_7utility8VariadicEPKcSt9__va_list"]
    fn L2CFighterCommon_bind_hash_call_call_check_damage();

    #[link_name = "\u{1}_ZN7lua2cpp16L2CFighterCommon32bind_hash_call_call_check_attackEPN3lib8L2CAgentERNS1_7utility8VariadicEPKcSt9__va_list"]
    fn L2CFighterCommon_bind_hash_call_call_check_attack();

    #[link_name = "\u{1}_ZN7lua2cpp16L2CFighterCommon32bind_hash_call_call_on_change_lrEPN3lib8L2CAgentERNS1_7utility8VariadicEPKcSt9__va_list"]
    fn L2CFighterCommon_bind_hash_call_call_on_change_lr();

    #[link_name = "\u{1}_ZN7lua2cpp16L2CFighterCommon30bind_hash_call_call_leave_stopEPN3lib8L2CAgentERNS1_7utility8VariadicEPKcSt9__va_list"]
    fn L2CFighterCommon_bind_hash_call_call_leave_stop();

    #[link_name = "\u{1}_ZN7lua2cpp16L2CFighterCommon40bind_hash_call_call_notify_event_gimmickEPN3lib8L2CAgentERNS1_7utility8VariadicEPKcSt9__va_list"]
    fn L2CFighterCommon_bind_hash_call_call_notify_event_gimmick();

    #[link_name = "\u{1}_ZN7lua2cpp16L2CFighterCommon30bind_hash_call_call_calc_paramEPN3lib8L2CAgentERNS1_7utility8VariadicEPKcSt9__va_list"]
    fn L2CFighterCommon_bind_hash_call_call_calc_param();
}

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
    let ftr_map = SYS_FTR_MAP.lock();
    for (agent, func) in ftr_map.iter() {
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
pub unsafe extern "Rust" fn replace_lua_script(fighter: &'static str, script: Hash40, func: ScriptBootstrapperFunc) {
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
    let mut func_map = FUNC_MAP.lock();
    if let Some(x) = func_map.get_mut(fighter) {
        if !x.contains(&info) {
            x.push(info);
        }
        else {
            println!("[lua-replace] Script has already been replaced -- skipping.");
        }
    } else {
        func_map.insert(String::from(fighter), vec![info]);
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
                sv_set_function_hash_replace,
                sys_line_fighter_hot_patch,
                sys_line_weapon_hot_patch,
                sys_line_system_control_fighter_hook,
                sys_line_system_control_weapon_hook
            );
        },
        "item" => {}, // We don't want to register the item module since it isn't compatible yet
        name => {
            let mut nro_map = NRO_MAP.lock();
            if nro_map.contains_key(name) {
                println!("[lua-replace] Loaded NRO already has entry in nro map. Maybe a bug? Updating entry.");
            }
            unsafe {
                let mod_start = (*info.module.ModuleObject).module_base;
                let mod_end = mod_start + 0xFFFFFFFF; // safe, should probably figure this out.

                nro_map.insert(String::from(name), unwind::Nro::new(mod_start, mod_end));
            }
            println!("[lua-replace] Registered {}", name);
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
            NRO_MAP.lock().remove(&String::from(name)).unwrap();
            println!("[lua-replace] Unregistered {}", name);
        }
    }
}

#[skyline::main(name = "lua-replace")]
pub fn main() {
    unwind::install_hooks();
    check::install_hooks();
    nro::add_hook(nro_load_hook).unwrap();
    nro::add_unload_hook(nro_unload_hook).unwrap();
}
