use std::{time::Duration, sync::Arc, borrow::BorrowMut};

use rusb::{
    request_type, Context, Device, DeviceHandle, Direction, GlobalContext, Recipient, RequestType,
    UsbContext,
};
use rusb_async::TransferPool;

const EASYCAP_VID: u16 = 0x1b71;
const EASYCAP_PID: u16 = 0x3002;

const USBTV_BASE: u16 = 0xC000;
const CONTROL_REGISTER_REQUEST: u8 = 12;

struct Resolution(u16, u16);

pub enum TVStandard {
    NTSC,
    PAL,
    SECAM,
}

pub enum Input {
    Composite,
    SVideo,
}

pub struct EasyCap {
    device: Device<Context>,
    context: Context,
    pub(crate) handle: Arc<DeviceHandle<Context>>,

    resolution: Resolution,
    standard: TVStandard,
    input: Input,
}

fn get_device(context: &Context, vid: u16, pid: u16) -> Result<Device<Context>, rusb::Error> {
    for device in context.devices()?.iter() {
        let device_desc = device.device_descriptor()?;
        if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
            return Ok(device);
        }
    }
    Err(rusb::Error::NotFound)
}

impl EasyCap {
    pub fn new() -> Result<EasyCap, rusb::Error> {
        let context = rusb::Context::new()?;
        let device = get_device(&context, EASYCAP_VID, EASYCAP_PID)?;
        let mut handle = Arc::new(device.open()?);
        // This is very ugly, figure out a better way?
        Arc::get_mut(&mut handle).unwrap().claim_interface(0);

        //claim_interface(0);
        //Arc::make_mut(this)
        //let mut h = handle.clone();
        //let pool = TransferPool::new(handle.clone()).unwrap(); // TODO: Deal with errors properly
        //h.claim_interface(0)?;

        Ok(EasyCap {
            device,
            context,
            handle,

            resolution: Resolution(720, 480), // TODO: These are in an unknown state (don't know what program used the device before us). How can we force them to be set?
            standard: TVStandard::NTSC,
            input: Input::Composite,
        })
    }

    fn set_register(&self, register: u16, value: u8) -> Result<(), rusb::Error> {
        let request_type =
            rusb::request_type(Direction::Out, RequestType::Vendor, Recipient::Interface);

        self.handle.write_control(
            request_type,
            CONTROL_REGISTER_REQUEST,
            value.into(),
            register,
            &[],
            Duration::from_secs(1),
        )?;

        Ok(())
    }

    fn set_registers(&self, registers: &[(u16, u8)]) -> Result<(), rusb::Error> {
        for (register, value) in registers {
            self.set_register(*register, *value)?;
        }
        Ok(())
    }

    pub fn begin_capture(&self) -> Result<(), rusb::Error> {
        const REGISTERS: [(u16, u8); 54] = [
            // These seem to enable the device.
            (USBTV_BASE + 0x0008, 0x0001),
            (USBTV_BASE + 0x01D0, 0x00FF),
            (USBTV_BASE + 0x01D9, 0x0002),
            // These seem to influence color parameters, such as
            //  brightness, etc.
            (USBTV_BASE + 0x0239, 0x0040),
            (USBTV_BASE + 0x0240, 0x0000),
            (USBTV_BASE + 0x0241, 0x0000),
            (USBTV_BASE + 0x0242, 0x0002),
            (USBTV_BASE + 0x0243, 0x0080),
            (USBTV_BASE + 0x0244, 0x0012),
            (USBTV_BASE + 0x0245, 0x0090),
            (USBTV_BASE + 0x0246, 0x0000),
            (USBTV_BASE + 0x0278, 0x002D),
            (USBTV_BASE + 0x0279, 0x000A),
            (USBTV_BASE + 0x027A, 0x0032),
            (0xF890, 0x000C),
            (0xF894, 0x0086),
            // Other setup values
            (USBTV_BASE + 0x00AC, 0x00C0),
            (USBTV_BASE + 0x00AD, 0x0000),
            (USBTV_BASE + 0x00A2, 0x0012),
            (USBTV_BASE + 0x00A3, 0x00E0),
            (USBTV_BASE + 0x00A4, 0x0028),
            (USBTV_BASE + 0x00A5, 0x0082),
            (USBTV_BASE + 0x00A7, 0x0080),
            (USBTV_BASE + 0x0000, 0x0014),
            (USBTV_BASE + 0x0006, 0x0003),
            (USBTV_BASE + 0x0090, 0x0099),
            (USBTV_BASE + 0x0091, 0x0090),
            (USBTV_BASE + 0x0094, 0x0068),
            (USBTV_BASE + 0x0095, 0x0070),
            (USBTV_BASE + 0x009C, 0x0030),
            (USBTV_BASE + 0x009D, 0x00C0),
            (USBTV_BASE + 0x009E, 0x00E0),
            (USBTV_BASE + 0x0019, 0x0006),
            (USBTV_BASE + 0x008C, 0x00BA),
            (USBTV_BASE + 0x0101, 0x00FF),
            (USBTV_BASE + 0x010C, 0x00B3),
            (USBTV_BASE + 0x01B2, 0x0080),
            (USBTV_BASE + 0x01B4, 0x00A0),
            (USBTV_BASE + 0x014C, 0x00FF),
            (USBTV_BASE + 0x014D, 0x00CA),
            (USBTV_BASE + 0x0113, 0x0053),
            (USBTV_BASE + 0x0119, 0x008A),
            (USBTV_BASE + 0x013C, 0x0003),
            (USBTV_BASE + 0x0150, 0x009C),
            (USBTV_BASE + 0x0151, 0x0071),
            (USBTV_BASE + 0x0152, 0x00C6),
            (USBTV_BASE + 0x0153, 0x0084),
            (USBTV_BASE + 0x0154, 0x00BC),
            (USBTV_BASE + 0x0155, 0x00A0),
            (USBTV_BASE + 0x0156, 0x00A0),
            (USBTV_BASE + 0x0157, 0x009C),
            (USBTV_BASE + 0x0158, 0x001F),
            (USBTV_BASE + 0x0159, 0x0006),
            (USBTV_BASE + 0x015D, 0x0000),
        ];
        self.set_registers(&REGISTERS)?;
        Ok(())
    }

    pub fn set_standard(&mut self, standard: TVStandard) -> Result<(), rusb::Error> {
        self.standard = standard;

        const AVPAL: [(u16, u8); 24] = [
            // "AVPAL" tuning sequence from .INF file
            (USBTV_BASE + 0x0003, 0x0004),
            (USBTV_BASE + 0x001A, 0x0068),
            (USBTV_BASE + 0x0100, 0x00D3),
            (USBTV_BASE + 0x010E, 0x0072),
            (USBTV_BASE + 0x010F, 0x00A2),
            (USBTV_BASE + 0x0112, 0x00B0),
            (USBTV_BASE + 0x0115, 0x0015),
            (USBTV_BASE + 0x0117, 0x0001),
            (USBTV_BASE + 0x0118, 0x002C),
            (USBTV_BASE + 0x012D, 0x0010),
            (USBTV_BASE + 0x012F, 0x0020),
            (USBTV_BASE + 0x0220, 0x002E),
            (USBTV_BASE + 0x0225, 0x0008),
            (USBTV_BASE + 0x024E, 0x0002),
            (USBTV_BASE + 0x024F, 0x0002),
            (USBTV_BASE + 0x0254, 0x0059),
            (USBTV_BASE + 0x025A, 0x0016),
            (USBTV_BASE + 0x025B, 0x0035),
            (USBTV_BASE + 0x0263, 0x0017),
            (USBTV_BASE + 0x0266, 0x0016),
            (USBTV_BASE + 0x0267, 0x0036),
            // End image tuning
            (USBTV_BASE + 0x024E, 0x0002),
            (USBTV_BASE + 0x024F, 0x0002),
            // This was referred to as "norm" in the Linux driver, but I've added it here. It doesn't seem to have any effect?
            (USBTV_BASE + 0x016F, 0x00EE),
        ];

        const AVNTSC: [(u16, u8); 24] = [
            // "AVNTSC" tuning sequence from .INF file
            (USBTV_BASE + 0x0003, 0x0004),
            (USBTV_BASE + 0x001A, 0x0079),
            (USBTV_BASE + 0x0100, 0x00D3),
            (USBTV_BASE + 0x010E, 0x0068),
            (USBTV_BASE + 0x010F, 0x009C),
            (USBTV_BASE + 0x0112, 0x00F0),
            (USBTV_BASE + 0x0115, 0x0015),
            (USBTV_BASE + 0x0117, 0x0000),
            (USBTV_BASE + 0x0118, 0x00FC),
            (USBTV_BASE + 0x012D, 0x0004),
            (USBTV_BASE + 0x012F, 0x0008),
            (USBTV_BASE + 0x0220, 0x002E),
            (USBTV_BASE + 0x0225, 0x0008),
            (USBTV_BASE + 0x024E, 0x0002),
            (USBTV_BASE + 0x024F, 0x0001),
            (USBTV_BASE + 0x0254, 0x005F),
            (USBTV_BASE + 0x025A, 0x0012),
            (USBTV_BASE + 0x025B, 0x0001),
            (USBTV_BASE + 0x0263, 0x001C),
            (USBTV_BASE + 0x0266, 0x0011),
            (USBTV_BASE + 0x0267, 0x0005),
            // End image tuning
            (USBTV_BASE + 0x024E, 0x0002),
            (USBTV_BASE + 0x024F, 0x0002),
            // This was referred to as "norm" in the Linux driver, but I've added it here. It doesn't seem to have any effect?
            (USBTV_BASE + 0x016F, 0x00B8),
        ];

        const AVSECAM: [(u16, u8); 24] = [
            // "AVSECAM" tuning sequence from .INF file
            (USBTV_BASE + 0x0003, 0x0004),
            (USBTV_BASE + 0x001A, 0x0073),
            (USBTV_BASE + 0x0100, 0x00DC),
            (USBTV_BASE + 0x010E, 0x0072),
            (USBTV_BASE + 0x010F, 0x00A2),
            (USBTV_BASE + 0x0112, 0x0090),
            (USBTV_BASE + 0x0115, 0x0035),
            (USBTV_BASE + 0x0117, 0x0001),
            (USBTV_BASE + 0x0118, 0x0030),
            (USBTV_BASE + 0x012D, 0x0004),
            (USBTV_BASE + 0x012F, 0x0008),
            (USBTV_BASE + 0x0220, 0x002D),
            (USBTV_BASE + 0x0225, 0x0028),
            (USBTV_BASE + 0x024E, 0x0008),
            (USBTV_BASE + 0x024F, 0x0002),
            (USBTV_BASE + 0x0254, 0x0069),
            (USBTV_BASE + 0x025A, 0x0016),
            (USBTV_BASE + 0x025B, 0x0035),
            (USBTV_BASE + 0x0263, 0x0021),
            (USBTV_BASE + 0x0266, 0x0016),
            (USBTV_BASE + 0x0267, 0x0036),
            // End image tuning
            (USBTV_BASE + 0x024E, 0x0002),
            (USBTV_BASE + 0x024F, 0x0002),
            // This was referred to as "norm" in the Linux driver, but I've added it here. It doesn't seem to have any effect?
            (USBTV_BASE + 0x016F, 0x00FF),
        ];

        let registers = match self.standard {
            TVStandard::PAL => &AVPAL,
            TVStandard::NTSC => &AVNTSC,
            TVStandard::SECAM => &AVSECAM,
        };

        self.set_registers(registers)?;
        Ok(())
    }

    pub fn set_input(&mut self, input: Input) -> Result<(), rusb::Error> {
        self.input = input;

        const COMPOSITE: [(u16, u8); 5] = [
            (USBTV_BASE + 0x0105, 0x0060),
            (USBTV_BASE + 0x011F, 0x00F2),
            (USBTV_BASE + 0x0127, 0x0060),
            (USBTV_BASE + 0x00AE, 0x0010),
            (USBTV_BASE + 0x0239, 0x0060),
        ];

        const SVIDEO: [(u16, u8); 5] = [
            (USBTV_BASE + 0x0105, 0x0010),
            (USBTV_BASE + 0x011F, 0x00FF),
            (USBTV_BASE + 0x0127, 0x0060),
            (USBTV_BASE + 0x00AE, 0x0030),
            (USBTV_BASE + 0x0239, 0x0060),
        ];

        let registers = match self.input {
            Input::Composite => &COMPOSITE,
            Input::SVideo => &SVIDEO,
        };

        self.set_registers(registers)?;
        Ok(())
    }

    // Activating the Alternative mode is necessary to begin streaming
    // TODO: refactor this
    pub fn alt_setting(&mut self) {
        Arc::get_mut(&mut self.handle).unwrap().set_alternate_setting(0, 1);
        //self.handle.set_alternate_setting(0, 1);
    }
}
