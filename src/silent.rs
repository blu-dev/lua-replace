// module to silently hook the NRO and replace acmd loading stuff with my own
use smash::{lib, lua2cpp, phx, app, lua_State};
use std::vec::Vec;
use std::collections::HashMap;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use std::mem::*;
use smash::hash40;

type StatusFunc = unsafe extern "C" fn(&mut lua2cpp::L2CAgentBase);
type CreateAgentFunc = unsafe extern "C" fn(phx::Hash40, *mut app::BattleObject, *mut app::BattleObjectModuleAccessor, *mut lua_State) -> *mut lua2cpp::L2CAgentBase;

pub struct CreateAgentInfo {
    pub original: CreateAgentFunc,
    pub hashes: Vec<u64>
}
#[derive(Clone)]
pub struct StatusAgentInfo {
    pub hash: u64,
    pub set_status_func: StatusFunc,
    pub status_dtor: StatusFunc,
    pub status_del_dtor: StatusFunc // del dtor deletes memory and also destroys
}

lazy_static! {
    pub static ref EFFECT_MAP: Mutex<HashMap<String, CreateAgentInfo>> = Mutex::new(HashMap::new());
    pub static ref EFFECT_SHARE_MAP: Mutex<HashMap<String, CreateAgentInfo>> = Mutex::new(HashMap::new());
    pub static ref EXPRESSION_MAP: Mutex<HashMap<String, CreateAgentInfo>> = Mutex::new(HashMap::new());
    pub static ref EXPRESSION_SHARE_MAP: Mutex<HashMap<String, CreateAgentInfo>> = Mutex::new(HashMap::new());
    pub static ref GAME_MAP: Mutex<HashMap<String, CreateAgentInfo>> = Mutex::new(HashMap::new());
    pub static ref GAME_SHARE_MAP: Mutex<HashMap<String, CreateAgentInfo>> = Mutex::new(HashMap::new());
    pub static ref SOUND_MAP: Mutex<HashMap<String, CreateAgentInfo>> = Mutex::new(HashMap::new());
    pub static ref SOUND_SHARE_MAP: Mutex<HashMap<String, CreateAgentInfo>> = Mutex::new(HashMap::new());
    pub static ref STATUS_MAP: Mutex<HashMap<String, CreateAgentInfo>> = Mutex::new(HashMap::new());
    pub static ref STATUS_SET_MAP: Mutex<HashMap<u64, StatusAgentInfo>> = Mutex::new(HashMap::new());
}

unsafe extern "C" fn create_agent_fighter_animcmd_effect(
    hash: phx::Hash40,
    bobj: *mut app::BattleObject,
    boma: *mut app::BattleObjectModuleAccessor,
    state: *mut lua_State
) -> *mut lua2cpp::L2CAgentBase {
    let map = EFFECT_MAP.lock();
    for (_, info) in map.iter() {
        if info.hashes.contains(&hash.hash) {
            let ret = (info.original)(hash, bobj, boma, state);
            let move_map = crate::EFFECT_MAP.lock();
            if let Some(moves) = move_map.get(&hash.hash) {
                for info in moves.iter() {
                    (*ret).sv_set_function_hash(Some(transmute(info.replace)), info.name.hash);
                }
            }
            else if let Some(moves) = move_map.get(&hash40("common")) {
                for info in moves.iter() {
                    (*ret).sv_set_function_hash(Some(transmute(info.replace)), info.name.hash);
                }
            }
            return ret;
        }
    }
    return 0 as *mut lua2cpp::L2CAgentBase;
}

unsafe extern "C" fn create_agent_fighter_animcmd_effect_share(
    hash: phx::Hash40,
    bobj: *mut app::BattleObject,
    boma: *mut app::BattleObjectModuleAccessor,
    state: *mut lua_State
) -> *mut lua2cpp::L2CAgentBase {
    let map = EFFECT_SHARE_MAP.lock();
    for (_, info) in map.iter() {
        if info.hashes.contains(&hash.hash) {
            let ret = (info.original)(hash, bobj, boma, state);
            let move_map = crate::EFFECT_MAP.lock();
            if let Some(moves) = move_map.get(&hash.hash) {
                for info in moves.iter() {
                    (*ret).sv_set_function_hash(Some(transmute(info.replace)), info.name.hash);
                }
            }
            else if let Some(moves) = move_map.get(&hash40("common")) {
                for info in moves.iter() {
                    (*ret).sv_set_function_hash(Some(transmute(info.replace)), info.name.hash);
                }
            }
            return ret;
        }
    }
    return 0 as *mut lua2cpp::L2CAgentBase;
}

unsafe extern "C" fn create_agent_fighter_animcmd_expression(
    hash: phx::Hash40,
    bobj: *mut app::BattleObject,
    boma: *mut app::BattleObjectModuleAccessor,
    state: *mut lua_State
) -> *mut lua2cpp::L2CAgentBase {
    let map = EXPRESSION_MAP.lock();
    for (_, info) in map.iter() {
        if info.hashes.contains(&hash.hash) {
            let ret = (info.original)(hash, bobj, boma, state);
            let move_map = crate::EXPRESSION_MAP.lock();
            if let Some(moves) = move_map.get(&hash.hash) {
                for info in moves.iter() {
                    (*ret).sv_set_function_hash(Some(transmute(info.replace)), info.name.hash);
                }
            }
            else if let Some(moves) = move_map.get(&hash40("common")) {
                for info in moves.iter() {
                    (*ret).sv_set_function_hash(Some(transmute(info.replace)), info.name.hash);
                }
            }
            return ret;
        }
    }
    return 0 as *mut lua2cpp::L2CAgentBase;
}

unsafe extern "C" fn create_agent_fighter_animcmd_expression_share(
    hash: phx::Hash40,
    bobj: *mut app::BattleObject,
    boma: *mut app::BattleObjectModuleAccessor,
    state: *mut lua_State
) -> *mut lua2cpp::L2CAgentBase {
    let map = EXPRESSION_SHARE_MAP.lock();
    for (_, info) in map.iter() {
        if info.hashes.contains(&hash.hash) {
            let ret = (info.original)(hash, bobj, boma, state);
            let move_map = crate::EXPRESSION_MAP.lock();
            if let Some(moves) = move_map.get(&hash.hash) {
                for info in moves.iter() {
                    (*ret).sv_set_function_hash(Some(transmute(info.replace)), info.name.hash);
                }
            }
            else if let Some(moves) = move_map.get(&hash40("common")) {
                for info in moves.iter() {
                    (*ret).sv_set_function_hash(Some(transmute(info.replace)), info.name.hash);
                }
            }
            return ret;
        }
    }
    return 0 as *mut lua2cpp::L2CAgentBase;
}

unsafe extern "C" fn create_agent_fighter_animcmd_game(
    hash: phx::Hash40,
    bobj: *mut app::BattleObject,
    boma: *mut app::BattleObjectModuleAccessor,
    state: *mut lua_State
) -> *mut lua2cpp::L2CAgentBase {
    let map = GAME_MAP.lock();
    for (_, info) in map.iter() {
        if info.hashes.contains(&hash.hash) {
            let ret = (info.original)(hash, bobj, boma, state);
            let move_map = crate::GAME_MAP.lock();
            if let Some(moves) = move_map.get(&hash.hash) {
                for info in moves.iter() {
                    (*ret).sv_set_function_hash(Some(transmute(info.replace)), info.name.hash);
                    println!("setting hash for agent {:#x}: {:#x}", hash.hash, info.name.hash);
                }
            }
            else if let Some(moves) = move_map.get(&hash40("common")) {
                for info in moves.iter() {
                    (*ret).sv_set_function_hash(Some(transmute(info.replace)), info.name.hash);
                }
            }
            return ret;
        }
    }
    return 0 as *mut lua2cpp::L2CAgentBase;
}

unsafe extern "C" fn create_agent_fighter_animcmd_game_share(
    hash: phx::Hash40,
    bobj: *mut app::BattleObject,
    boma: *mut app::BattleObjectModuleAccessor,
    state: *mut lua_State
) -> *mut lua2cpp::L2CAgentBase {
    let map = GAME_SHARE_MAP.lock();
    for (_, info) in map.iter() {
        if info.hashes.contains(&hash.hash) {
            let ret = (info.original)(hash, bobj, boma, state);
            let move_map = crate::GAME_MAP.lock();
            if let Some(moves) = move_map.get(&hash.hash) {
                for info in moves.iter() {
                    (*ret).sv_set_function_hash(Some(transmute(info.replace)), info.name.hash);
                }
            }
            else if let Some(moves) = move_map.get(&hash40("common")) {
                for info in moves.iter() {
                    (*ret).sv_set_function_hash(Some(transmute(info.replace)), info.name.hash);
                }
            }
            return ret;
        }
    }
    return 0 as *mut lua2cpp::L2CAgentBase;
}

unsafe extern "C" fn create_agent_fighter_animcmd_sound(
    hash: phx::Hash40,
    bobj: *mut app::BattleObject,
    boma: *mut app::BattleObjectModuleAccessor,
    state: *mut lua_State
) -> *mut lua2cpp::L2CAgentBase {
    let map = SOUND_MAP.lock();
    for (_, info) in map.iter() {
        if info.hashes.contains(&hash.hash) {
            let ret = (info.original)(hash, bobj, boma, state);
            let move_map = crate::SOUND_MAP.lock();
            if let Some(moves) = move_map.get(&hash.hash) {
                for info in moves.iter() {
                    (*ret).sv_set_function_hash(Some(transmute(info.replace)), info.name.hash);
                }
            }
            else if let Some(moves) = move_map.get(&hash40("common")) {
                for info in moves.iter() {
                    (*ret).sv_set_function_hash(Some(transmute(info.replace)), info.name.hash);
                }
            }
            return ret;
        }
    }
    return 0 as *mut lua2cpp::L2CAgentBase;
}

unsafe extern "C" fn create_agent_fighter_animcmd_sound_share(
    hash: phx::Hash40,
    bobj: *mut app::BattleObject,
    boma: *mut app::BattleObjectModuleAccessor,
    state: *mut lua_State
) -> *mut lua2cpp::L2CAgentBase {
    let map = SOUND_SHARE_MAP.lock();
    for (_, info) in map.iter() {
        if info.hashes.contains(&hash.hash) {
            let ret = (info.original)(hash, bobj, boma, state);
            let move_map = crate::SOUND_MAP.lock();
            if let Some(moves) = move_map.get(&hash.hash) {
                for info in moves.iter() {
                    (*ret).sv_set_function_hash(Some(transmute(info.replace)), info.name.hash);
                }
            }
            else if let Some(moves) = move_map.get(&hash40("common")) {
                for info in moves.iter() {
                    (*ret).sv_set_function_hash(Some(transmute(info.replace)), info.name.hash);
                }
            }
            return ret;
        }
    }
    return 0 as *mut lua2cpp::L2CAgentBase;
}

unsafe extern "C" fn create_agent_fighter_status_script(
    hash: phx::Hash40,
    bobj: *mut app::BattleObject,
    boma: *mut app::BattleObjectModuleAccessor,
    state: *mut lua_State
) -> *mut lua2cpp::L2CAgentBase {
    let map = STATUS_MAP.lock();
    for (_, info) in map.iter() {
        if info.hashes.contains(&hash.hash) {
            let ret = (info.original)(hash, bobj, boma, state);
            let mut set_map = STATUS_SET_MAP.lock();
            // we don't want to double hook the status funcs
            let mut matching_info = 0 as *const StatusAgentInfo;
            for (_, info) in set_map.iter() {
                if info.hash == hash.hash {
                    matching_info = info as *const StatusAgentInfo;
                    break;
                }
            }
            if !matching_info.is_null() {
                set_map.insert(ret as u64, (*matching_info).clone());
                return ret;
            }
            let vtable = (*ret).vtable as *const u64;
            let dtor = *vtable;
            let del_dtor = *vtable.offset(1);
            let set_status = *vtable.offset(9);
            let dtor_replace = status_agent_dtor as *const fn();
            let del_dtor_replace = status_agent_del_dtor as *const fn();
            let set_status_replace = set_status_scripts as *const fn();
            sky_memcpy(vtable as *const c_void, transmute(&dtor_replace), 8);
            sky_memcpy(vtable.offset(1) as *const c_void, transmute(&del_dtor_replace), 8);
            sky_memcpy(vtable.offset(9) as *const c_void, transmute(&set_status_replace), 8);
            set_map.insert(ret as u64, StatusAgentInfo{hash: hash.hash, set_status_func: transmute(set_status), status_dtor: transmute(dtor), status_del_dtor: transmute(del_dtor)});
            return ret;
        }
    }
    0 as *mut lua2cpp::L2CAgentBase
}

unsafe extern "C" fn status_agent_dtor(agent: &mut lua2cpp::L2CAgentBase) {
    // remove from our map
    let mut map = STATUS_SET_MAP.lock();
    let as_u64 = agent as *mut lua2cpp::L2CAgentBase as u64;
    let info = map.remove(&as_u64).unwrap();
    (info.status_dtor)(agent)
}

unsafe extern "C" fn status_agent_del_dtor(agent: &mut lua2cpp::L2CAgentBase) {
    // remove from our map
    let mut map = STATUS_SET_MAP.lock();
    let as_u64 = agent as *mut lua2cpp::L2CAgentBase as u64;
    let info = map.remove(&as_u64).unwrap();
    (info.status_del_dtor)(agent)
}

unsafe extern "C" fn set_status_scripts(agent: &mut lua2cpp::L2CAgentBase) {
    use smash::lib::lua_const::*;
    let as_u64 = agent as *mut lua2cpp::L2CAgentBase as u64;
    let map = STATUS_SET_MAP.lock();
    if let Some(info) = map.get(&as_u64) {
        (info.set_status_func)(agent);
        let move_map = crate::STATUS_MAP.lock();
        let category = smash::app::sv_system::battle_object_category(agent.lua_state_agent);
        if category == *BATTLE_OBJECT_CATEGORY_FIGHTER as u8 {
            if let Some(statuses) = move_map.get(&hash40("common")) {
                for status in statuses {
                    agent.sv_set_status_func(
                        lib::L2CValue::new_int(*status.status as u64),
                        lib::L2CValue::new_int(*status.condition as u64),
                        transmute(status.replace)
                    );
                }
            }
        }
        if let Some(statuses) = move_map.get(&info.hash) {
            for status in statuses {
                agent.sv_set_status_func(
                    lib::L2CValue::new_int(*status.status as u64),
                    lib::L2CValue::new_int(*status.condition as u64),
                    transmute(status.replace)
                );
            }
        }
    }
}

struct Cpu {
    pub registers: [u64; 31]
}

impl Cpu {
    const fn new() -> Self {
        Self { registers: [0; 31] }
    }
}

use aarch64_decode::*;

fn get_movk_offset(instr: u32) -> u32 {
    (instr & 0x00600000) >> 21
}

pub unsafe fn get_hashes(mut start: *const u32) -> Vec<u64> {
    let mut hashes = std::vec::Vec::<u64>::new();
    let mut cpu: [u64; 31] = [0; 31];
    loop {
        let instr = aarch64_decode::decode_a64(*start);
        if instr.is_none() { break; }
        let instr = instr.unwrap();
        match instr {
            Instr::Ret64RBranchReg => {
                break;
            },
            Instr::Movz64Movewide{ imm16: imm, Rd: dst, .. } => {
                cpu[dst as usize] = imm as u64;
            },
            Instr::Movk64Movewide{ imm16: part, Rd: dst, ..} => {
                let offset = get_movk_offset(*start) * 16;
                cpu[dst as usize] |= (part as u64) << offset;
            },
            Instr::Subs64AddsubShift{ Rm: src, Rn: _, Rd: _, .. } => {
                hashes.push(cpu[src as usize]);
            },
            _ => {}
        }
        start = start.offset(1);
    }
    hashes
}

fn make_sym(fighter: String, func: String) -> String {
    let func_name = format!("create_agent_fighter_{}_{}", func, fighter);
    format!("_ZN7lua2cpp{}{}EN3phx6Hash40EPN3app12BattleObjectEPNS2_26BattleObjectModuleAccessorEP9lua_State", func_name.len(), func_name)
}

use skyline::{libc::*, patching::sky_memcpy};

unsafe fn set_fighter_func(info: &skyline::nro::NroInfo, func: CreateAgentFunc, kind: String) -> u64 {
    let base = (*info.module.ModuleObject).module_base;
    let sym_name = make_sym(info.name.to_owned(), kind);
    let symbol = crate::rtld::get_symbol_by_name(info.module.ModuleObject as *const _, sym_name);
    if symbol == 0 as *const nnsdk::root::Elf64_Sym {
        return 0;
    }
    let orig = base + (*symbol).st_value;
    let difference = (func as *const fn() as u64) - base;
    skyline::patching::sky_memcpy(&(*symbol).st_value as *const u64 as *const c_void, &difference as *const u64 as *const c_void, 8);
    orig
}

// this code is really ugly please don't look at it k thx :)
pub unsafe fn set_fighter_funcs(info: &skyline::nro::NroInfo) {
    let effect = set_fighter_func(info, create_agent_fighter_animcmd_effect, "animcmd_effect".to_owned());
    let effect_share = set_fighter_func(info, create_agent_fighter_animcmd_effect_share, "animcmd_effect_share".to_owned());
    let expression = set_fighter_func(info, create_agent_fighter_animcmd_expression, "animcmd_expression".to_owned());
    let expression_share = set_fighter_func(info, create_agent_fighter_animcmd_expression_share, "animcmd_expression_share".to_owned());
    let game = set_fighter_func(info, create_agent_fighter_animcmd_game, "animcmd_game".to_owned());
    let game_share = set_fighter_func(info, create_agent_fighter_animcmd_game_share, "animcmd_game_share".to_owned());
    let sound = set_fighter_func(info, create_agent_fighter_animcmd_sound, "animcmd_sound".to_owned());
    let sound_share = set_fighter_func(info, create_agent_fighter_animcmd_sound_share, "animcmd_sound_share".to_owned());
    let status = set_fighter_func(info, create_agent_fighter_status_script, "status_script".to_owned());
    if effect != 0 {
        let mut map = EFFECT_MAP.lock();
        map.insert(info.name.to_owned(), CreateAgentInfo{original: transmute(effect), hashes: get_hashes(effect as *const u32)});
    }
    if effect_share != 0 {
        let mut map = EFFECT_SHARE_MAP.lock();
        map.insert(info.name.to_owned(), CreateAgentInfo{original: transmute(effect_share), hashes: get_hashes(effect_share as *const u32)});
    }
    if expression != 0 {
        let mut map = EXPRESSION_MAP.lock();
        map.insert(info.name.to_owned(), CreateAgentInfo{original: transmute(expression), hashes: get_hashes(expression as *const u32)});
    }
    if expression_share != 0 {
        let mut map = EXPRESSION_SHARE_MAP.lock();
        map.insert(info.name.to_owned(), CreateAgentInfo{original: transmute(expression_share), hashes: get_hashes(expression_share as *const u32)});
    }
    if game != 0 {
        let mut map = GAME_MAP.lock();
        map.insert(info.name.to_owned(), CreateAgentInfo{original: transmute(game), hashes: get_hashes(game as *const u32)});
    }
    if game_share != 0 {
        let mut map = GAME_SHARE_MAP.lock();
        map.insert(info.name.to_owned(), CreateAgentInfo{original: transmute(game_share), hashes: get_hashes(game_share as *const u32)});
    }
    if sound != 0 {
        let mut map = SOUND_MAP.lock();
        map.insert(info.name.to_owned(), CreateAgentInfo{original: transmute(sound), hashes: get_hashes(sound as *const u32)});
    }
    if sound_share != 0 {
        let mut map = SOUND_SHARE_MAP.lock();
        map.insert(info.name.to_owned(), CreateAgentInfo{original: transmute(sound_share), hashes: get_hashes(sound_share as *const u32)});
    }
    if status != 0 {
        let mut map = STATUS_MAP.lock();
        map.insert(info.name.to_owned(), CreateAgentInfo{original: transmute(status), hashes: get_hashes(status as *const u32)});
    }
}

pub unsafe fn remove_fighter_funcs(info: &skyline::nro::NroInfo) {
    let mut map = EFFECT_MAP.lock();
    if map.contains_key(info.name) {
        map.remove(info.name);
    }
    drop(map);
    let mut map = EFFECT_SHARE_MAP.lock();
    if map.contains_key(info.name) {
        map.remove(info.name);
    }
    drop(map);
    let mut map = EXPRESSION_MAP.lock();
    if map.contains_key(info.name) {
        map.remove(info.name);
    }
    drop(map);
    let mut map = EXPRESSION_SHARE_MAP.lock();
    if map.contains_key(info.name) {
        map.remove(info.name);
    }
    drop(map);
    let mut map = GAME_MAP.lock();
    if map.contains_key(info.name) {
        map.remove(info.name);
    }
    drop(map);
    let mut map = GAME_SHARE_MAP.lock();
    if map.contains_key(info.name) {
        map.remove(info.name);
    }
    drop(map);
    let mut map = SOUND_MAP.lock();
    if map.contains_key(info.name) {
        map.remove(info.name);
    }
    drop(map);
    let mut map = SOUND_SHARE_MAP.lock();
    if map.contains_key(info.name) {
        map.remove(info.name);
    }
    drop(map);
    let mut map = STATUS_MAP.lock();
    if map.contains_key(info.name) {
        map.remove(info.name);
    }
}

pub unsafe fn byte_search(needle: &[u32]) -> Option<usize> {
    let mut matching = 0usize;
    let text_start = skyline::hooks::getRegionAddress(skyline::hooks::Region::Text) as *const u32;
    let text_end = skyline::hooks::getRegionAddress(skyline::hooks::Region::Rodata) as *const u32;
    let mut pos = 0isize;
    let mut match_begin = 0usize;
    loop {
        if text_start.offset(pos) == text_end { break; }
        if matching == needle.len() { break; }
        if *text_start.offset(pos) == needle[matching] {
            if matching == 0 { match_begin = text_start.offset(pos) as usize; }
            matching += 1;
        }
        else {
            matching = 0;
            match_begin = 0;
        }
        pos += 1;
    }
    if match_begin == 0 { None }
    else { Some(match_begin) }
}