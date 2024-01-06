#![no_std]
#![no_main]

mod keymap;

use defmt_rtt as _;
use panic_probe as _;

use rp_pico::{hal, pac, hal::usb};
use rp_pico::hal::gpio::{DynPinId, FunctionSioOutput, FunctionSioInput, Pin, DynPullType};
use embedded_hal::digital::v2::OutputPin;
type DynPinInput = Pin<DynPinId, FunctionSioInput, DynPullType>;
type DynPinOutput = Pin<DynPinId, FunctionSioOutput, DynPullType>;

use rp2040_monotonic::Rp2040Monotonic;
type Instant = <Rp2040Monotonic as rtic::Monotonic>::Instant;
type Duration = <Rp2040Monotonic as rtic::Monotonic>::Duration;

use usb_device::bus::UsbBusAllocator;
use usb_device::class::UsbClass as _;
use usb_device::device::UsbDeviceState;
use usb_device::prelude::*;
type UsbClass = keyberon::Class<'static, usb::UsbBus, ()>;
type UsbDevice = usb_device::device::UsbDevice<'static, usb::UsbBus>;

const KBD_COLS: usize = 3;
const KBD_ROWS: usize = 3;
const KBD_LAYERS: usize = 2;

/// USB VID/PID for a generic keyboard from
/// https://github.com/obdev/v-usb/blob/master/usbdrv/USB-IDs-for-free.txt
const VID: u16 = 0x16c0;
const PID: u16 = 0x27db;
const PRODUCT_MANUFACTURER: &'static str = "kolontsov.com";
const PRODUCT_NAME: &'static str = "M8-kbd";
const PRODUCT_SERIAL: &'static str = env!("CARGO_PKG_VERSION");
const KBD_TICK_PERIOD: Duration = Duration::millis(1);

use keyberon::key_code::KbHidReport;
use keyberon::layout::{Event, CustomEvent};
use keyberon::chording::{ChordDef, Chording};
use keyberon::action::Action;
use keyberon::debounce::Debouncer;
use keyberon::matrix::Matrix;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CustomAction { Uf2, Reset }
pub const UF2: Action<CustomAction> = Action::Custom(CustomAction::Uf2);
pub const RESET: Action<CustomAction> = Action::Custom(CustomAction::Reset);
pub type Layout = keyberon::layout::Layout<KBD_COLS, KBD_ROWS, KBD_LAYERS, CustomAction>;
pub type Layers = keyberon::layout::Layers<KBD_COLS, KBD_ROWS, KBD_LAYERS, CustomAction>;

const LEFT_DOWN: ChordDef = ((2, 0), &[(1, 0), (1, 1)]);
const CHORDS: [ChordDef; 1] = [LEFT_DOWN];

#[cfg(not(debug_assertions))]
pub fn debug_print_event(_event: Event) {
}

#[cfg(debug_assertions)]
pub fn debug_print_event(event: Event) {
    match event {
        keyberon::layout::Event::Press (row, col) => {
            defmt::info!("Press: row: {:?}, col: {:?}", row, col);
        },
        keyberon::layout::Event::Release (row, col) => {
            defmt::info!("Release: row: {:?}, col: {:?}", row, col);
        },
    }
}

#[rtic::app(device = rp_pico::hal::pac, peripherals = true, dispatchers = [SW0_IRQ, SW1_IRQ])]
mod app {
    use super::*;

    #[monotonic(binds = TIMER_IRQ_0, default = true)]
    type Monotonic = Rp2040Monotonic;

    #[shared]
    struct Shared {
        usb_dev: UsbDevice,
        usb_class: UsbClass,
        #[lock_free]
        layout: Layout,
    }

    #[local]
    struct Local {
        matrix: Matrix<DynPinInput, DynPinOutput, KBD_COLS, KBD_ROWS>,
        debouncer: Debouncer<[[bool; KBD_COLS]; KBD_ROWS]>,
        chording: Chording<1>,
    }

    #[init(local = [usb_alloc: Option<UsbBusAllocator<hal::usb::UsbBus>> = None])]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        unsafe { hal::sio::spinlock_reset() }

        defmt::info!("Booting");

        // Clock configuration
        let mut resets = cx.device.RESETS;
        let mut watchdog = hal::watchdog::Watchdog::new(cx.device.WATCHDOG);
        let clocks = hal::clocks::init_clocks_and_plls(
            rp_pico::XOSC_CRYSTAL_FREQ,
            cx.device.XOSC,
            cx.device.CLOCKS,
            cx.device.PLL_SYS,
            cx.device.PLL_USB,
            &mut resets,
            &mut watchdog,
        ).ok().unwrap();

        // GPIO configuration
        let sio = hal::Sio::new(cx.device.SIO);
        let pins = hal::gpio::Pins::new(
            cx.device.IO_BANK0,
            cx.device.PADS_BANK0,
            sio.gpio_bank0,
            &mut resets);

        // USB
        let usb_bus = hal::usb::UsbBus::new(
            cx.device.USBCTRL_REGS, 
            cx.device.USBCTRL_DPRAM,
            clocks.usb_clock, true,
            &mut resets);
        let usb_alloc = cx.local.usb_alloc.insert(UsbBusAllocator::new(usb_bus));

        let usb_class = keyberon::new_class(usb_alloc, ());
        let usb_dev = UsbDeviceBuilder::new(usb_alloc, UsbVidPid(VID, PID))
            .manufacturer(PRODUCT_MANUFACTURER)
            .product(PRODUCT_NAME)
            .serial_number(PRODUCT_SERIAL)
            .supports_remote_wakeup(true)
            .build();

        // Enable the USB interrupt
        unsafe { pac::NVIC::unmask(pac::Interrupt::USBCTRL_IRQ) };

        let now = monotonics::now();
        tick::spawn(now).unwrap();

        // Keyboard matrix configuration
        let mut rows: [DynPinOutput; KBD_ROWS] = [
            pins.gpio7.into_push_pull_output().into_pull_type().into_dyn_pin(),
            pins.gpio8.into_push_pull_output().into_pull_type().into_dyn_pin(),
            pins.gpio9.into_push_pull_output().into_pull_type().into_dyn_pin(),
        ];
        let cols: [DynPinInput; KBD_COLS] = [
            pins.gpio2.into_pull_up_input().into_pull_type().into_dyn_pin(),
            pins.gpio3.into_pull_up_input().into_pull_type().into_dyn_pin(),
            pins.gpio4.into_pull_up_input().into_pull_type().into_dyn_pin(),
        ];
        for r in rows.iter_mut() {
            r.set_high().unwrap();
        }

        // Keyboard state
        let matrix = Matrix::new(cols, rows).unwrap();
        let debouncer = Debouncer::new(
            [[false; KBD_COLS]; KBD_ROWS],
            [[false; KBD_COLS]; KBD_ROWS],
            5,
        );
        let layout = Layout::new(&keymap::LAYERS);
        let chording = Chording::new(&CHORDS);

        (
            Shared { usb_dev, usb_class, layout },
            Local { matrix, debouncer, chording },
            init::Monotonics(Rp2040Monotonic::new(cx.device.TIMER)),
        )
    }

    #[task(priority = 1, local = [matrix, debouncer, chording])]
    fn tick(cx: tick::Context, now: Instant) {
        // scan matrix, debounce, detect chording, generate events
        let matrix_state = cx.local.matrix.get().unwrap();
        let debounced = cx.local.debouncer.events(matrix_state);
        let events = cx.local.chording.tick(debounced.collect()).into_iter();

        // pass matrix events to layout handler to convert to keycodes
        for event in events {
            debug_print_event(event);
            handle_event::spawn(event).unwrap();
        }

        // send keycodes to the host
        tick_keyberon::spawn().unwrap();

        let next = now + KBD_TICK_PERIOD;
        tick::spawn_at(next, next).unwrap();
    }

    #[task(priority = 2, capacity = 8, shared = [layout])]
    fn handle_event(cx: handle_event::Context, event: Event) {
        cx.shared.layout.event(event)
    }

    #[task(priority = 2, shared = [usb_dev, usb_class, layout])]
    fn tick_keyberon(mut cx: tick_keyberon::Context) {
        // actually process keypresses
        let tick = cx.shared.layout.tick();
        let report: KbHidReport = cx.shared.layout.keycodes().collect();
        let key_pressed = report.as_bytes().iter().any(|&b| b != 0);

        // wake up the host if it's sleeping
        cx.shared.usb_dev.lock(|d| {
            if key_pressed && d.state() == UsbDeviceState::Suspend && d.remote_wakeup_enabled() {
                defmt::info!("Wakeup!");
                d.bus().remote_wakeup();
            }
        });

        // special keys
        match tick {
            CustomEvent::Press(event) => match event {
                CustomAction::Uf2 => {
                    defmt::info!("Switching to UF2 bootloader");
                    hal::rom_data::reset_to_usb_boot(0, 0);
                }
                CustomAction::Reset => {
                    defmt::info!("Reset!");
                    cortex_m::peripheral::SCB::sys_reset();
                }
            },
            _ => (),
        }
        
        if cx.shared.usb_dev.lock(|d| d.state()) != UsbDeviceState::Configured {
            return;
        }
        
        // if no new keys are pressed, don't send a report
        if !cx.shared.usb_class.lock(|c| c.device_mut().set_keyboard_report(report.clone())) {
            return;
        };

        // actually send the report
        while let Ok(0) = cx.shared.usb_class.lock(|c| c.write(report.as_bytes())) { };
    }

    // --- USB interrupt request ---
    #[task(binds = USBCTRL_IRQ, priority = 3, shared = [usb_dev, usb_class], local = [])]
    fn usb_irq(cx: usb_irq::Context) {
        (cx.shared.usb_dev, cx.shared.usb_class).lock(|usb_dev, usb_class| {
            if usb_dev.poll(&mut [usb_class]) {
                usb_class.poll();
            }
        });
    }
}
