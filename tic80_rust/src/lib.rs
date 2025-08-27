#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    rust_2018_idioms
)]
#![allow(
    clippy::many_single_char_names,
    clippy::too_many_arguments,
    clippy::similar_names,
    clippy::multiple_crate_versions,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]

pub mod gfx {
    pub mod framebuffer;
}

pub mod core {
    pub mod memory;
}

pub mod script {
    pub mod lua_runner;
}

pub mod audio {
    pub mod capture;
    pub mod fft;
    pub mod vqt;
}
