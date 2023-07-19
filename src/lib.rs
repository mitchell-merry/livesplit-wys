mod gamemaker;
mod settings;

use std::collections::HashMap;

use asr::{future::next_tick, timer::{self, TimerState}, Process, Address, signature::Signature, watcher::Watcher, time::Duration};
use gamemaker::get_room_map;

asr::async_main!(stable);


async fn main() {
    asr::print_message("Initialising...");
    let settings = settings::initialise_settings(include_str!("../data/settings.ron"));

    if settings.is_none() {
        return;
    }

    let settings = settings.unwrap();

    let mut watchers = Watchers::new();

    loop {
        let process = Process::wait_attach(PROCESS_NAME).await;
        process.until_closes(async {
            let addresses = Addresses::init(&process).expect("Error initialising addresses!");

            let rooms = get_room_map(&process, &addresses.room_array).expect("Error reading rooms");

            loop {
                // update watchers
                watchers.update(&process, &addresses, &rooms);
                let Some(fulltime) = watchers.fulltime.pair else { return; };
                let Some(room_id) = watchers.room_id.pair else { return; };

                let old_name = rooms.get(&room_id.old);
                let current_name = rooms.get(&room_id.current);

                if timer::state() == TimerState::NotRunning {

                }

                if timer::state() == TimerState::Running {
                    timer::set_game_time(Duration::seconds_f64(fulltime.current));
                    
                    if room_id.changed() && old_name.is_some() && current_name.is_some() {
                        asr::print_message(&format!("room: {} -> {}", old_name.unwrap(), current_name.unwrap()));
                    }
                }

                next_tick().await;
            }
        }).await;
        
    }
}



const PROCESS_NAME: &str = "Will You Snail.exe";

const SIG_ROOM_ID: Signature<6> = Signature::new("4D 0F 45 F5 8B 0D");
const SIG_ROOM_ARRAY: Signature<13> = Signature::new("74 0C 48 8B 05 ?? ?? ?? ?? 48 8B 04 D0");


struct Addresses {
    main_module_base: Address,
    room_id: Address,
    room_array: Address,
}

impl Addresses {
    fn init(process: &Process) -> Option<Addresses> {
        let main_module_base = process.get_module_address(PROCESS_NAME).ok()?;
        let main_module_size = process.get_module_size(PROCESS_NAME).ok()?;
        asr::print_message(&format!("Found main module information: base = {:X}, size = {:X}", main_module_base.value(), main_module_size));

        Some(Addresses {
            main_module_base,
            room_id: scan_rel(process, &SIG_ROOM_ID, 0x6)?,
            room_array: scan_rel(process, &SIG_ROOM_ARRAY, 0x5)?,
        })
    }
}

fn scan_rel<const N: usize>(process: &Process, sig: &Signature<N>, offset: u32) -> Option<Address> {
    // TODO cache these?
    let main_module_base = process.get_module_address(PROCESS_NAME).ok()?;
    let main_module_size = process.get_module_size(PROCESS_NAME).ok()?;
    
    let ptr: Address = sig.scan_process_range(&process, (main_module_base, main_module_size))? + offset;
    let rip: u64 = process.read::<u32>(ptr).ok()?.into();
    
    Some(Address::new(ptr.value() + 0x4 + rip))
}

const CHAPTERTIME_PATH: [u64; 4] = [0x1739868, 0x0, 0x150, 0x520];
const FULLTIME_PATH: [u64; 4] = [0x1739868, 0x0, 0x150, 0x530];

struct Watchers {
    room_id: Watcher<u32>,
    room_name: Watcher<String>,
    chaptertime: Watcher<f64>,
    fulltime: Watcher<f64>,
}

impl Watchers {
    fn new() -> Watchers {
        Watchers {
            room_id: Watcher::new(),
            room_name: Watcher::new(),
            chaptertime: Watcher::new(),
            fulltime: Watcher::new(),
        }
    }

    fn update(&mut self, process: &Process, addresses: &Addresses, rooms: &HashMap<u32, String>) {
        let room_id = process.read::<u32>(addresses.room_id).unwrap_or_default();
        self.room_id.update(Some(room_id));

        self.chaptertime.update(Some(process.read_pointer_path64::<f64>(addresses.main_module_base, &CHAPTERTIME_PATH).unwrap_or_default()));
        self.fulltime.update(Some(process.read_pointer_path64::<f64>(addresses.main_module_base, &FULLTIME_PATH).unwrap_or_default()));
    }
}