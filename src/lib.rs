#![feature(ptr_sub_ptr, const_ptr_sub_ptr)]

use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicUsize, Ordering};
use skyline::libc::c_void;
use unity::prelude::*;

const BUTTON_A: i64 = 1;
const BUTTON_X: i64 = 4;
const BUTTON_Y: i64 = 8;
const BUTTON_ZL: i64 = 256;
const BUTTON_ZR: i64 = 512;
const BUTTON_MINUS: i64 = 2048;
const BUTTON_UP: i64 = 8192;
const BUTTON_DOWN: i64 = 32768;

const WIN_CAMERAS: [i32; 3] = [1000, 1001, 1010];
const DIE_CAMERAS: [i32; 3] = [1002, 1003, 1012];

const BEHIND_PLAYER: i32 = 101;
const BEHIND_ENEMY: i32 = 150;

const AUTO: i32 = i32::MIN;

const ALWAYS_ALLOW: [i32; 21] = [
    0, -1,
    850, 851,
    600, 601, 9999,
    901, 902, 903, 904,
    115,
    120, 751, 752,
    8000, 8001, 8002, 8050,
    6800, 6801,
];

const BATTLE_CYCLE: [i32; 8] = [
    AUTO,
    1,
    3,
    BEHIND_PLAYER,
    BEHIND_ENEMY,
    300,
    200,
    210,
];

const FINISH_CYCLE: [i32; 8] = [
    BEHIND_ENEMY,
    BEHIND_PLAYER,
    700,
    750,
    300,
    301,
    200,
    1,
];

const HOLD: f32 = 1337.0;

const M_SECONDS: usize = 40;
const CHARACTER_FSM: usize = 64;

static CAMERA_SWITCH: AtomicUsize = AtomicUsize::new(0);

static LOCKED: AtomicI32 = AtomicI32::new(AUTO);
static BATTLE_SLOT: AtomicUsize = AtomicUsize::new(0);

static CHOSEN_CAMERA: AtomicI32 = AtomicI32::new(AUTO);

static FINISH_SLOT: AtomicUsize = AtomicUsize::new(usize::MAX);

static OURS: AtomicBool = AtomicBool::new(false);

static HOLDING: AtomicBool = AtomicBool::new(false);

static CINEMATIC_ACTIVE: AtomicBool = AtomicBool::new(false);

static DIVERTED: AtomicBool = AtomicBool::new(false);

fn is_cinematic(id: i32) -> bool {
    id > 0 && ALWAYS_ALLOW.contains(&id)
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct V3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct V2 {
    pub x: f32,
    pub y: f32,
}

const V2_ZERO: V2 = V2 { x: 0.0, y: 0.0 };

#[unity::from_offset("App", "Pad", "IsTrigger")]
pub fn pad_is_trigger(buttons: i64, method_info: OptionalMethod) -> bool;

#[unity::from_offset("App", "Pad", "IsButton")]
pub fn pad_is_button(buttons: i64, method_info: OptionalMethod) -> bool;

#[unity::from_offset("App", "Pad", "GetStickLX")]
pub fn pad_stick_lx(method_info: OptionalMethod) -> f32;
#[unity::from_offset("App", "Pad", "GetStickLY")]
pub fn pad_stick_ly(method_info: OptionalMethod) -> f32;
#[unity::from_offset("App", "Pad", "GetStickRX")]
pub fn pad_stick_rx(method_info: OptionalMethod) -> f32;
#[unity::from_offset("App", "Pad", "GetStickRY")]
pub fn pad_stick_ry(method_info: OptionalMethod) -> f32;

#[unity::from_offset("Combat", "FSMBuilder", "get_anyone")]
pub fn fsm_builder_get_anyone(method_info: OptionalMethod) -> *mut u8;

#[unity::from_offset("Combat", "FSMBuilder", "get_record")]
pub fn fsm_builder_get_record(method_info: OptionalMethod) -> *mut u8;

#[unity::from_offset("Combat", "CombatRecord", "get_LastPhase")]
pub fn record_get_last_phase(this: *mut u8, method_info: OptionalMethod) -> *mut u8;

#[unity::from_offset("Combat", "CombatRecord", "get_CombatStyle")]
pub fn record_get_combat_style(this: *mut u8, method_info: OptionalMethod) -> i32;

#[unity::from_offset("Combat", "Phase", "FindDieSide")]
pub fn phase_find_die_side(this: *mut u8, method_info: OptionalMethod) -> i32;

#[unity::from_offset("Combat", "Side", "IsPlayerSide")]
pub fn side_is_player_side(side: i32, method_info: OptionalMethod) -> bool;

#[unity::from_offset("Combat", "CameraSwitch", "get_CurrentCamera")]
pub fn camera_switch_get_current(this: *mut u8, method_info: OptionalMethod) -> i32;

#[unity::from_offset("Combat", "CameraSwitch", "get_Item")]
pub fn camera_switch_get_item(this: *mut u8, position: i32, method_info: OptionalMethod) -> *mut u8;

#[unity::from_offset("Combat", "CameraSwitch", "SwitchCamera")]
pub fn camera_switch_apply(this: *mut u8, position: i32, force: bool, method_info: OptionalMethod);

#[unity::from_offset("Combat", "ActionWaitTime", ".ctor")]
pub fn action_wait_time_ctor(this: *mut u8, chr: *mut u8, seconds: f32, method_info: OptionalMethod);

#[unity::from_offset("Combat", "FSM", "Add")]
pub fn fsm_add(this: *mut u8, state: *mut u8, method_info: OptionalMethod);

#[unity::from_offset("UnityEngine", "Time", "get_timeScale")]
pub fn time_get_scale(method_info: OptionalMethod) -> f32;

#[unity::from_offset("UnityEngine", "Time", "set_timeScale")]
pub fn time_set_scale(value: f32, method_info: OptionalMethod);

#[unity::from_offset("UnityEngine", "Time", "get_unscaledDeltaTime")]
pub fn time_get_unscaled_delta(method_info: OptionalMethod) -> f32;

#[unity::from_offset("UnityEngine", "Camera", "get_main")]
pub fn camera_get_main(method_info: OptionalMethod) -> *mut u8;

#[unity::from_offset("UnityEngine", "Camera", "get_fieldOfView")]
pub fn camera_get_fov(this: *mut u8, method_info: OptionalMethod) -> f32;

#[unity::from_offset("UnityEngine", "Camera", "set_fieldOfView")]
pub fn camera_set_fov(this: *mut u8, value: f32, method_info: OptionalMethod);

#[unity::from_offset("App", "GameConfig", "get_BattleCameraReverseHorizontal")]
pub fn config_battle_reverse_horizontal(this: *const u8, method_info: OptionalMethod) -> bool;

#[unity::from_offset("App", "GameConfig", "get_BattleCameraReverseVertical")]
pub fn config_battle_reverse_vertical(this: *const u8, method_info: OptionalMethod) -> bool;

#[unity::class("App", "GameConfig")]
pub struct GameConfig {}

/// GameConfig is a SingletonClass<GameConfig>: its get_Instance lives on the parent class.
fn game_config() -> *const u8 {
    let parent = &*GameConfig::class()._1.parent;
    let method = match parent.get_methods().iter().find(|m| m.get_name().as_deref() == Some("get_Instance")) {
        Some(m) => m,
        None => return std::ptr::null(),
    };
    let get_instance = unsafe {
        std::mem::transmute::<_, extern "C" fn(&unity::il2cpp::method::MethodInfo) -> *const u8>(method.method_ptr)
    };
    get_instance(method)
}

#[unity::from_offset("UnityEngine", "Component", "get_transform")]
pub fn component_get_transform(this: *mut u8, method_info: OptionalMethod) -> *mut u8;

#[unity::from_offset("UnityEngine", "Transform", "get_position")]
pub fn transform_get_position(this: *mut u8, method_info: OptionalMethod) -> V3;

#[unity::from_offset("UnityEngine", "Transform", "get_eulerAngles")]
pub fn transform_get_euler(this: *mut u8, method_info: OptionalMethod) -> V3;

#[unity::from_offset("UnityEngine", "Transform", "set_position")]
pub fn transform_set_position(this: *mut u8, value: V3, method_info: OptionalMethod);

#[unity::from_offset("UnityEngine", "Transform", "set_eulerAngles")]
pub fn transform_set_euler(this: *mut u8, value: V3, method_info: OptionalMethod);

#[unity::from_offset("UnityEngine", "Component", "get_gameObject")]
pub fn component_get_game_object(this: *mut u8, method_info: OptionalMethod) -> *mut u8;

#[skyline::from_offset(0x2c4dd40)]
pub fn game_object_get_component_by_name(this: *mut u8, type_name: &Il2CppString, method_info: OptionalMethod) -> *mut u8;

#[unity::from_offset("UnityEngine", "Behaviour", "get_enabled")]
pub fn behaviour_get_enabled(this: *mut u8, method_info: OptionalMethod) -> bool;

#[unity::from_offset("UnityEngine", "Behaviour", "set_enabled")]
pub fn behaviour_set_enabled(this: *mut u8, value: bool, method_info: OptionalMethod);

unsafe fn apply_camera(switch: *mut u8, id: i32) {
    OURS.store(true, Ordering::Relaxed);
    camera_switch_apply(switch, id, true, None);
    OURS.store(false, Ordering::Relaxed);
}

unsafe fn camera_behind_loser() -> Option<i32> {
    let record = fsm_builder_get_record(None);
    if record.is_null() {
        return None;
    }
    let phase = record_get_last_phase(record, None);
    if phase.is_null() {
        return None;
    }

    let dead_side = phase_find_die_side(phase, None);
    let player_died = side_is_player_side(dead_side, None);

    Some(if player_died { BEHIND_PLAYER } else { BEHIND_ENEMY })
}

#[unity::hook("Combat", "CameraSwitch", "SwitchCamera", 2)]
fn camera_switch_switch_camera(
    this: &mut c_void,
    next_camera: i32,
    force: bool,
    method_info: OptionalMethod,
) {
    let switch = this as *mut c_void as *mut u8;

    if CAMERA_SWITCH.swap(switch as usize, Ordering::Relaxed) != switch as usize {
        LOCKED.store(AUTO, Ordering::Relaxed);
        BATTLE_SLOT.store(0, Ordering::Relaxed);
        HOLDING.store(false, Ordering::Relaxed);
        CINEMATIC_ACTIVE.store(false, Ordering::Relaxed);
        DIVERTED.store(false, Ordering::Relaxed);
    }

    if OURS.load(Ordering::Relaxed) {
        return call_original!(this, next_camera, force, method_info);
    }

    if WIN_CAMERAS.contains(&next_camera) || DIE_CAMERAS.contains(&next_camera) {
        LOCKED.store(AUTO, Ordering::Relaxed);
        BATTLE_SLOT.store(0, Ordering::Relaxed);
        FINISH_SLOT.store(usize::MAX, Ordering::Relaxed);

        let mut chosen = next_camera;
        unsafe {
            if let Some(wanted) = camera_behind_loser() {
                if !camera_switch_get_item(switch, wanted, None).is_null() {
                    chosen = wanted;
                }
            }
        }
        CHOSEN_CAMERA.store(chosen, Ordering::Relaxed);
        return call_original!(this, chosen, force, method_info);
    }

    let locked = LOCKED.load(Ordering::Relaxed);

    if is_cinematic(next_camera) {
        CINEMATIC_ACTIVE.store(true, Ordering::Relaxed);
    } else if CINEMATIC_ACTIVE.swap(false, Ordering::Relaxed) && locked != AUTO {
        DIVERTED.store(false, Ordering::Relaxed);
        unsafe { apply_camera(switch, locked) }
        return;
    }

    if locked != AUTO && !ALWAYS_ALLOW.contains(&next_camera) {
        if DIVERTED.swap(false, Ordering::Relaxed) {
            unsafe { apply_camera(switch, locked) }
        }
        return;
    }

    if locked != AUTO {
        DIVERTED.store(true, Ordering::Relaxed);
    }

    call_original!(this, next_camera, force, method_info)
}

unsafe fn step_battle_camera(forward: bool) {
    let switch = CAMERA_SWITCH.load(Ordering::Relaxed) as *mut u8;
    if switch.is_null() {
        return;
    }

    let len = BATTLE_CYCLE.len();
    let mut slot = BATTLE_SLOT.load(Ordering::Relaxed);

    for _ in 0..len {
        slot = if forward { (slot + 1) % len } else { (slot + len - 1) % len };
        let id = BATTLE_CYCLE[slot];

        if id == AUTO {
            BATTLE_SLOT.store(slot, Ordering::Relaxed);
            LOCKED.store(AUTO, Ordering::Relaxed);
            return;
        }

        if !camera_switch_get_item(switch, id, None).is_null() {
            BATTLE_SLOT.store(slot, Ordering::Relaxed);
            LOCKED.store(id, Ordering::Relaxed);
            apply_camera(switch, id);
            return;
        }
    }
}

unsafe fn step_finish_camera(forward: bool) {
    let switch = CAMERA_SWITCH.load(Ordering::Relaxed) as *mut u8;
    if switch.is_null() {
        return;
    }

    let slot = FINISH_SLOT.load(Ordering::Relaxed);
    let len = FINISH_CYCLE.len();

    for step in 1..=len {
        let next = if slot == usize::MAX {
            if forward { step - 1 } else { len - step }
        } else if forward {
            (slot + step) % len
        } else {
            (slot + len - step) % len
        };

        let id = FINISH_CYCLE[next];
        if !camera_switch_get_item(switch, id, None).is_null() {
            FINISH_SLOT.store(next, Ordering::Relaxed);
            apply_camera(switch, id);
            return;
        }
    }
}

static FREE_CAM: AtomicBool = AtomicBool::new(false);

static CAM_X: AtomicU32 = AtomicU32::new(0);
static CAM_Y: AtomicU32 = AtomicU32::new(0);
static CAM_Z: AtomicU32 = AtomicU32::new(0);
static CAM_YAW: AtomicU32 = AtomicU32::new(0);
static CAM_PITCH: AtomicU32 = AtomicU32::new(0);
static CAM_FOV: AtomicU32 = AtomicU32::new(0);
static SAVED_FOV: AtomicU32 = AtomicU32::new(0);
static SAVED_SCALE: AtomicU32 = AtomicU32::new(0);
static FAST: AtomicBool = AtomicBool::new(false);
/// Cinemachine brain switched off while free cam writes the camera itself
static BRAIN: AtomicUsize = AtomicUsize::new(0);

fn get_f32(cell: &AtomicU32) -> f32 {
    f32::from_bits(cell.load(Ordering::Relaxed))
}

fn set_f32(cell: &AtomicU32, value: f32) {
    cell.store(value.to_bits(), Ordering::Relaxed);
}

fn wrap360(angle: f32) -> f32 {
    let wrapped = angle % 360.0;
    if wrapped < 0.0 {
        wrapped + 360.0
    } else {
        wrapped
    }
}

/// Angle in -180..180, so pitch can be clamped
fn wrap180(angle: f32) -> f32 {
    let wrapped = wrap360(angle);
    if wrapped > 180.0 { wrapped - 360.0 } else { wrapped }
}

/// Stick value with the deadzone cut out and the rest rescaled, so movement starts from 0
fn stick(value: f32) -> f32 {
    if value.abs() < DEADZONE {
        0.0
    } else {
        value.signum() * (value.abs() - DEADZONE) / (1.0 - DEADZONE)
    }
}

const DEADZONE: f32 = 0.15;
const MOVE_SPEED: f32 = 3.6; // units per second
const TURN_SPEED: f32 = 54.0; // degrees per second at the entry field of view
const ZOOM_SPEED: f32 = 30.0; // field of view degrees per second
const FAST_FACTOR: f32 = 3.0;
const PITCH_LIMIT: f32 = 89.0;

unsafe fn active_rig_transform() -> *mut u8 {
    let switch = CAMERA_SWITCH.load(Ordering::Relaxed) as *mut u8;
    if switch.is_null() {
        return std::ptr::null_mut();
    }
    let rig = camera_switch_get_item(switch, camera_switch_get_current(switch, None), None);
    if rig.is_null() {
        return std::ptr::null_mut();
    }
    component_get_transform(rig, None)
}

unsafe fn free_cam_enter() {
    let camera = camera_get_main(None);
    let transform = if camera.is_null() { active_rig_transform() } else { component_get_transform(camera, None) };
    if transform.is_null() {
        println!("[death_hold] free cam: no camera");
        return;
    }

    let position = transform_get_position(transform, None);
    let euler = transform_get_euler(transform, None);
    set_f32(&CAM_X, position.x);
    set_f32(&CAM_Y, position.y);
    set_f32(&CAM_Z, position.z);
    set_f32(&CAM_YAW, euler.y);
    set_f32(&CAM_PITCH, wrap180(euler.x).clamp(-PITCH_LIMIT, PITCH_LIMIT));

    let fov = if camera.is_null() { 0.0 } else { camera_get_fov(camera, None) };
    set_f32(&SAVED_FOV, fov);
    set_f32(&CAM_FOV, fov);

    // Cinemachine keeps steering the camera (damping, framing, shake): switch it off
    // and write the camera directly in free_cam_drive
    if !camera.is_null() {
        let brain = game_object_get_component_by_name(component_get_game_object(camera, None), "CinemachineBrain".into(), None);
        if !brain.is_null() && behaviour_get_enabled(brain, None) {
            behaviour_set_enabled(brain, false, None);
            BRAIN.store(brain as usize, Ordering::Relaxed);
        } else {
            println!("[death_hold] free cam: no Cinemachine brain on the camera");
        }
    }

    set_f32(&SAVED_SCALE, time_get_scale(None));
    time_set_scale(0.0, None);

    FREE_CAM.store(true, Ordering::Relaxed);
}

unsafe fn free_cam_exit() {
    if !FREE_CAM.swap(false, Ordering::Relaxed) {
        return;
    }
    let scale = get_f32(&SAVED_SCALE);
    time_set_scale(if scale > 0.0 { scale } else { 1.0 }, None);

    let brain = BRAIN.swap(0, Ordering::Relaxed) as *mut u8;
    if !brain.is_null() {
        behaviour_set_enabled(brain, true, None);
    }

    let camera = camera_get_main(None);
    let fov = get_f32(&SAVED_FOV);
    if !camera.is_null() && fov > 0.0 {
        camera_set_fov(camera, fov, None);
    }

    let switch = CAMERA_SWITCH.load(Ordering::Relaxed) as *mut u8;
    let locked = LOCKED.load(Ordering::Relaxed);
    if !switch.is_null() && locked != AUTO {
        apply_camera(switch, locked);
    }
}

fn forward_vector(yaw: f32, pitch: f32) -> V3 {
    let (sin_yaw, cos_yaw) = yaw.to_radians().sin_cos();
    let (sin_pitch, cos_pitch) = pitch.to_radians().sin_cos();
    V3 { x: sin_yaw * cos_pitch, y: -sin_pitch, z: cos_yaw * cos_pitch }
}

fn free_cam_pose() -> (V3, V3) {
    let forward = forward_vector(get_f32(&CAM_YAW), get_f32(&CAM_PITCH));
    let position = V3 { x: get_f32(&CAM_X), y: get_f32(&CAM_Y), z: get_f32(&CAM_Z) };
    let target = V3 {
        x: position.x + forward.x,
        y: position.y + forward.y,
        z: position.z + forward.z,
    };
    (position, target)
}

unsafe fn free_cam_drive() {
    // Battle time is frozen in free cam, so use unscaled time
    let dt = time_get_unscaled_delta(None);
    if pad_is_trigger(BUTTON_Y, None) {
        FAST.store(!FAST.load(Ordering::Relaxed), Ordering::Relaxed);
    }
    let boost = if FAST.load(Ordering::Relaxed) { FAST_FACTOR } else { 1.0 };
    let (lx, ly) = (stick(pad_stick_lx(None)), stick(pad_stick_ly(None)));
    let (rx, ry) = (stick(pad_stick_rx(None)), stick(pad_stick_ry(None)));

    // Zoom (ZL out, ZR in); looking slows down when zoomed in
    let camera = camera_get_main(None);
    let mut fov = get_f32(&CAM_FOV);
    if !camera.is_null() && fov > 0.0 {
        if pad_is_button(BUTTON_ZL, None) { fov += ZOOM_SPEED * dt; }
        if pad_is_button(BUTTON_ZR, None) { fov -= ZOOM_SPEED * dt; }
        fov = fov.clamp(5.0, 150.0);
        set_f32(&CAM_FOV, fov);
        camera_set_fov(camera, fov, None);
    }
    let saved_fov = get_f32(&SAVED_FOV);
    let zoom = if fov > 0.0 && saved_fov > 0.0 { fov / saved_fov } else { 1.0 };

    // The player's battle camera invert settings
    let config = game_config();
    let invert_x = !config.is_null() && config_battle_reverse_horizontal(config, None);
    let invert_y = !config.is_null() && config_battle_reverse_vertical(config, None);
    let turn = TURN_SPEED * zoom * dt;
    let yaw = wrap360(get_f32(&CAM_YAW) + rx * turn * if invert_x { -1.0 } else { 1.0 });
    let pitch = (get_f32(&CAM_PITCH) - ry * turn * if invert_y { -1.0 } else { 1.0 }).clamp(-PITCH_LIMIT, PITCH_LIMIT);

    let forward = forward_vector(yaw, pitch);
    let (sin_yaw, cos_yaw) = yaw.to_radians().sin_cos();
    let right = V3 { x: cos_yaw, y: 0.0, z: -sin_yaw };
    let step = MOVE_SPEED * boost * dt;
    let lift = if pad_is_button(BUTTON_UP, None) { 1.0 } else if pad_is_button(BUTTON_DOWN, None) { -1.0 } else { 0.0 };

    set_f32(&CAM_X, get_f32(&CAM_X) + (forward.x * ly + right.x * lx) * step);
    set_f32(&CAM_Y, get_f32(&CAM_Y) + (forward.y * ly + lift) * step);
    set_f32(&CAM_Z, get_f32(&CAM_Z) + (forward.z * ly + right.z * lx) * step);
    set_f32(&CAM_YAW, yaw);
    set_f32(&CAM_PITCH, pitch);

    if !camera.is_null() {
        let transform = component_get_transform(camera, None);
        if !transform.is_null() {
            let position = V3 { x: get_f32(&CAM_X), y: get_f32(&CAM_Y), z: get_f32(&CAM_Z) };
            transform_set_position(transform, position, None);
            transform_set_euler(transform, V3 { x: pitch, y: yaw, z: 0.0 }, None);
        }
    }
}

#[unity::hook("Combat", "BaseCameraController", "SetFollowLookupPos", 2)]
fn set_follow_lookup_pos(
    this: &mut c_void,
    follow: V3,
    look_at: V3,
    method_info: OptionalMethod,
) {
    if FREE_CAM.load(Ordering::Relaxed) {
        let (position, target) = free_cam_pose();
        return call_original!(this, position, target, method_info);
    }
    call_original!(this, follow, look_at, method_info)
}

#[unity::hook("Combat", "CombatInput", "get_CameraPan", 0)]
fn combat_input_camera_pan(this: &mut c_void, method_info: OptionalMethod) -> V2 {
    let pan = call_original!(this, method_info);
    if FREE_CAM.load(Ordering::Relaxed) { V2_ZERO } else { pan }
}

#[unity::hook("Combat", "CombatInput", "get_CameraTruck", 0)]
fn combat_input_camera_truck(this: &mut c_void, method_info: OptionalMethod) -> V2 {
    let truck = call_original!(this, method_info);
    if FREE_CAM.load(Ordering::Relaxed) { V2_ZERO } else { truck }
}

#[unity::hook("UnityEngine", "Time", "set_timeScale", 1)]
fn time_set_scale_hook(value: f32, method_info: OptionalMethod) {
    let value = if FREE_CAM.load(Ordering::Relaxed) { 0.0 } else { value };
    call_original!(value, method_info)
}

#[unity::hook("Combat", "FSM", "Update", 0)]
fn fsm_update(this: &mut c_void, method_info: OptionalMethod) {
    if FREE_CAM.load(Ordering::Relaxed) {
        return;
    }
    call_original!(this, method_info)
}

unsafe fn append_hold(reason: &str) {
    HOLDING.store(false, Ordering::Relaxed);

    let chr = fsm_builder_get_anyone(None);
    if chr.is_null() {
        println!("[death_hold] {}: no character", reason);
        return;
    }
    let fsm = *(chr.add(CHARACTER_FSM) as *mut *mut u8);
    if fsm.is_null() {
        println!("[death_hold] {}: no fsm", reason);
        return;
    }
    match unity::il2cpp::instantiate_class_by_name::<u8>("Combat", "ActionWaitTime") {
        Ok(state) => {
            let state = state as *mut u8;
            action_wait_time_ctor(state, chr, HOLD, None);
            fsm_add(fsm, state, None);
        }
        Err(_) => println!("[death_hold] {}: could not create wait state", reason),
    }
}

#[unity::hook("Combat", "FSMBuilderStandard", "BuildEnd_EnemyKilled", 0)]
fn build_end_enemy_killed(method_info: OptionalMethod) {
    call_original!(method_info);
    unsafe { append_hold("EnemyKilled") }
}

#[unity::hook("Combat", "FSMBuilderStandard", "BuildEnd_PlayerKilled", 0)]
fn build_end_player_killed(method_info: OptionalMethod) {
    call_original!(method_info);
    unsafe { append_hold("PlayerKilled") }
}

#[unity::hook("Combat", "ActionWaitTime", "OnUpdate", 0)]
fn action_wait_time_on_update(this: &mut c_void, method_info: OptionalMethod) {
    unsafe {
        let seconds = (this as *mut c_void as *mut u8).add(M_SECONDS) as *mut f32;
        if *seconds > 300.0 {
            HOLDING.store(true, Ordering::Relaxed);

            if FREE_CAM.load(Ordering::Relaxed) {
                return call_original!(this, method_info);
            }

            if pad_is_trigger(BUTTON_X, None) {
                let switch = CAMERA_SWITCH.load(Ordering::Relaxed) as *mut u8;
                let chosen = CHOSEN_CAMERA.load(Ordering::Relaxed);
                if !switch.is_null() && chosen != AUTO {
                    FINISH_SLOT.store(usize::MAX, Ordering::Relaxed);
                    apply_camera(switch, chosen);
                }
            } else if pad_is_trigger(BUTTON_ZR, None) {
                step_finish_camera(true);
            } else if pad_is_trigger(BUTTON_ZL, None) {
                step_finish_camera(false);
            }

            if pad_is_trigger(BUTTON_A, None) {
                *seconds = 0.0;
                HOLDING.store(false, Ordering::Relaxed);
            }
        }
    }
    call_original!(this, method_info)
}

const DRAGON_CHANGE: i32 = 4194304;

unsafe fn dragon_change_in_progress() -> bool {
    let record = fsm_builder_get_record(None);
    if record.is_null() {
        return false;
    }
    record_get_combat_style(record, None) & DRAGON_CHANGE != 0
}

#[unity::hook("Combat", "CharacterBuilder", "SetVisibleForced", 1)]
fn character_builder_set_visible_forced(
    this: &mut c_void,
    value: bool,
    method_info: OptionalMethod,
) {
    if !value {
        unsafe {
            if dragon_change_in_progress() {
                return call_original!(this, value, method_info);
            }
        }
    }
    call_original!(this, true, method_info)
}

#[unity::hook("Combat", "CombatWorld", "Update", 0)]
fn combat_world_update(this: &mut c_void, method_info: OptionalMethod) {
    call_original!(this, method_info);

    if HOLDING.load(Ordering::Relaxed) {
        return;
    }

    unsafe {
        if pad_is_trigger(BUTTON_MINUS, None) {
            if FREE_CAM.load(Ordering::Relaxed) {
                free_cam_exit();
            } else {
                free_cam_enter();
            }
            return;
        }
        if FREE_CAM.load(Ordering::Relaxed) {
            return;
        }

        if CINEMATIC_ACTIVE.load(Ordering::Relaxed) {
            return;
        }

        if pad_is_trigger(BUTTON_X, None) {
            BATTLE_SLOT.store(0, Ordering::Relaxed);
            LOCKED.store(AUTO, Ordering::Relaxed);
            return;
        }

        if pad_is_trigger(BUTTON_ZR, None) {
            step_battle_camera(true);
        } else if pad_is_trigger(BUTTON_ZL, None) {
            step_battle_camera(false);
        }
    }
}

#[unity::hook("Combat", "CameraSwitch", "LateUpdate", 0)]
fn camera_switch_late_update(this: &mut c_void, method_info: OptionalMethod) {
    call_original!(this, method_info);

    if FREE_CAM.load(Ordering::Relaxed) {
        unsafe { free_cam_drive() }
    }
}

#[unity::hook("Combat", "CameraManager", "EndCamera", 2)]
fn camera_manager_end_camera(
    this: &mut c_void,
    transition: bool,
    mode: i32,
    method_info: OptionalMethod,
) {
    unsafe { free_cam_exit() }
    call_original!(this, transition, mode, method_info)
}

#[skyline::main(name = "dethhold")]
pub fn main() {
    println!("[death_hold] loaded");
    skyline::install_hooks!(
        build_end_enemy_killed,
        build_end_player_killed,
        action_wait_time_on_update,
        camera_switch_switch_camera,
        camera_switch_late_update,
        camera_manager_end_camera,
        combat_world_update,
        fsm_update,
        time_set_scale_hook,
        set_follow_lookup_pos,
        combat_input_camera_pan,
        combat_input_camera_truck,
        character_builder_set_visible_forced
    );
}
