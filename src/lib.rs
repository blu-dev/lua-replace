#![feature(proc_macro_hygiene)]
use skyline::{hook, install_hook};
use smash::lua2cpp::L2CFighterCommon;
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
}

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
unsafe fn sv_set_function_hash_replace(agent: &mut L2CAgent, mut function: ScriptBootstrapperFunc, hash: Hash40) {
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

#[no_mangle]
pub unsafe extern "Rust" fn lua_replace_script(fighter: &'static str, script: Hash40, func: ScriptBootstrapperFunc) {
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

fn nro_load_hook(info: &NroInfo) {
    match info.name {
        "common" => {
            install_hook!(sv_set_function_hash_replace);
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
