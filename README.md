
# DMALibrary

A crate for memory forensics and video game hacking


## Features

- Get Windows Version
- Getting PID & Base Address
- Patch CR3 [Untested]

## ToDo

- Sig Scanning
- Read Memory
- Write Memory
- Scatter Read Memory
- Scatter Write Memory
- Dumping Physical Memory
- Dumping Memory
- Target Computer Keyboard
- Code Cave Finder
- Function Caller
- Syscalling kernel functions
- Utilities (Get Import, Get Export, Get Base Size ect)
## Acknowledgements

 - [Metick's C++ DMA Library](https://github.com/Metick/DMALibrary)
 - [Memprocfs](https://crates.io/crates/memprocfs)

## Contributing

Contributions are always welcome!

Make a PR and ill add if it is worth adding.


## Usage/Examples

```rust
use colored::*;
use dmalibrary::{find_base_address, find_process, fix_cr3, get_winver, init};
use std::{env, error::Error};

const TARGETPE: &str = "smss.exe";

fn main() -> Result<(), Box<dyn Error>> {
    let current_dir = env::current_dir()?;
    let current_dir_str = current_dir
        .to_str()
        .ok_or("Failed to convert current directory to string")?;
    let vmm_path = format!("{}\\vmm.dll", current_dir_str);
    let args = ["", "-device", "fpga"].to_vec();

    match init(vmm_path.as_str(), &args) {
        Ok(vmm) => {
            println!("{}", "Successfully initialized Vmm.".green());

            let winver = get_winver(&vmm)?;
            println!("Windows version: {:?}", winver);

            let process_pid = find_process(&vmm, TARGETPE).ok_or_else(|| {
                Box::<dyn Error>::from(format!(
                    "{}",
                    format!("Failed to find process {}", TARGETPE).red()
                ))
            })?;

            println!("PID: {}", process_pid);

            let process = vmm.process_from_pid(process_pid).map_err(|e| {
                Box::<dyn Error>::from(format!(
                    "{}",
                    format!("Failed to get process from PID: {}", e).red()
                ))
            })?;

            let base_address = find_base_address(&vmm, process_pid, TARGETPE).ok_or_else(|| {
                Box::<dyn Error>::from(format!(
                    "{}",
                    format!("Failed to find base address for {}", TARGETPE).red()
                ))
            })?;

            println!("Base address: 0x{:X}", base_address);

            let patch_cr3 = fix_cr3(&vmm, &process, TARGETPE, process_pid)?;
            if patch_cr3 {
                println!("{}", "Successfully fixed CR3 register.".green());
            } else {
                println!("{}", "Failed to fix CR3 register.".red());
                println!("{}", "Probably should reboot PC".red());
            }
        }
        Err(e) => {
            println!("{} {}", "Failed to initialize Vmm:".red(), e);
        }
    }
    Ok(())
}
```


## Documentation

[Documentation](https://docs.rs/dmalibrary/latest/dmalibrary/)

[Crates.io](https://crates.io/crates/dmalibrary)


## Appendix

Make sure you have all the nessessary dlls like vmm.dll, leechcore.dll, FTD3XX.dll and so on 

