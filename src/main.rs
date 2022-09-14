#![allow(dead_code, unused)]

use crate::easycap::EasyCap;
use crate::streaming::*;

pub mod easycap;
pub mod streaming;

fn main() {
    //println!("Searching for EasyCap device...");

    let mut easycap = EasyCap::new().unwrap();
    easycap.begin_capture();
    easycap.set_standard(easycap::TVStandard::NTSC);
    easycap.set_input(easycap::Input::Composite);
    easycap.alt_setting();
    easycap.test();
    easycap.begin_streaming();
    //utv.open().unwrap();
    //println!("{}", utv.resolutionrus);

    // Note for the future: Create an EasyCap struct, and use impl methods on it.
}
