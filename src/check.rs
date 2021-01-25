use smash::phx::Hash40;

// A hacky solution, desperately needs byte searching to work
// Old acmd_hook implementation intercepted at the source at the cost of runtime efficiency
// Since every `create_agent_...` function can create either articles or fighters, we have to make sure that we are only intercepting the right hooks

// Resets set the current hash to None
// Hashes set the current hash to whatever is getting set
const EFFECT_RESET: usize = 0x6447d0;
const EFFECT_HASH:  usize = 0x323ebf4;
const GAME_RESET:   usize = 0x6441f0;
const GAME_HASH:    usize = 0x323dd54;
const SOUND_RESET:  usize = 0x645390;
const SOUND_HASH:   usize = 0x323fa94;

pub static mut HASH: Option<Hash40> = None;

static RESETS: [usize; 3] = [EFFECT_RESET, GAME_RESET, SOUND_RESET];
static HASHES: [usize; 3] = [EFFECT_HASH, GAME_HASH, SOUND_HASH];

static mut CURRENT_OFF: usize = EFFECT_RESET;
#[skyline::hook(offset = CURRENT_OFF, inline)]
unsafe fn reset(_: &skyline::hooks::InlineCtx) {
    HASH = None;
}

#[skyline::hook(offset = CURRENT_OFF, inline)]
unsafe fn set_hash(ctx: &skyline::hooks::InlineCtx) {
    HASH = Some(smash::phx::Hash40::new_raw(*ctx.registers[0].x.as_ref()));
}

pub fn install_hooks() {
    unsafe {
        for x in 0..RESETS.len() {
            CURRENT_OFF = RESETS[x];
            skyline::install_hook!(reset);
        }
        for x in 0..HASHES.len() {
            CURRENT_OFF = HASHES[x];
            skyline::install_hook!(set_hash);
        }
    }   
}