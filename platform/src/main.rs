
#[cfg(target_os = "windows")] 
mod win32;
#[cfg(target_os = "linux")] 
mod linux;

mod prelude {
    pub use gol_engine::*;    
    #[cfg(target_os = "windows")] 
    pub use crate::win32::*;     
    #[cfg(target_os = "linux")] 
    pub use crate::linux::*;     
}

fn main() {
    crate::prelude::App::start();
}