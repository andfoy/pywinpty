extern crate cxx;

/// Native bindings for ConPTY/WinPTY
#[cxx::bridge]
pub mod winpty {
    // {% include "definitions.rs.in" %}

    extern "Rust" {}

    unsafe extern "C++" {
        include!("wrapper.h");

        /// Reference to a torch tensor in memory
        // type CrossTensor;
        type WinptyPTY;

    }
}
