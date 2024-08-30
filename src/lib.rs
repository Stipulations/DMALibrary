use memprocfs::{Vmm, VmmProcess, CONFIG_OPT_PROCESS_DTB};
use std::{thread, time};
use std::error::Error;

/// Initializes a `Vmm` instance with the provided path and arguments.
///
/// # Arguments
///
/// * `vmm_path` - Path to the VMM (Virtual Machine Monitor).
/// * `args` - Arguments to pass to the VMM.
///
/// # Returns
///
/// A `Result` containing the `Vmm` instance on success, or a boxed error on failure.
///
/// # Examples
///
/// ```
/// let vmm_path = "path/to/vmm.dll";
/// let args = vec!["", "-device", "fpga"];
/// let vmm = init(vmm_path, &args).expect("Failed to initialize Vmm");
/// ```
pub fn init<'a>(vmm_path: &'a str, args: &'a Vec<&'a str>) -> Result<Vmm<'a>, Box<dyn std::error::Error + 'a>> {
    let vmm = Vmm::new(vmm_path, args)?;
    Ok(vmm)
}

/// Retrieves the Windows version from the VMM instance.
///
/// # Arguments
///
/// * `vmm` - Reference to a `Vmm` instance.
///
/// # Returns
///
/// A `Result` containing the Windows version as a `String`.
///
/// # Examples
///
/// ```
/// let winver = get_winver(&vmm).expect("Failed to get Windows version");
/// println!("Windows version: {}", winver);
/// ```
pub fn get_winver(vmm: &Vmm) -> Result<String, Box<dyn Error>> {
    Ok(vmm.kernel().build().to_string())
}

/// Finds the process ID (PID) of a process by its name.
///
/// # Arguments
///
/// * `vmm` - Reference to a `Vmm` instance.
/// * `process_name` - Name of the process to find.
///
/// # Returns
///
/// An `Option<u32>` containing the PID if found, or `None` if not found.
///
/// # Examples
///
/// ```
/// let pid = find_process(&vmm, "smss.exe").expect("Process not found");
/// println!("PID: {}", pid);
/// ```
pub fn find_process(vmm: &Vmm, process_name: &str) -> Option<u32> {
    match vmm.process_from_name(process_name) {
        Ok(process) => Some(process.pid),
        Err(e) => {
            println!("Failed to find {}: {}", process_name, e);
            None
        }
    }
}

/// Finds the base address of a module within a process.
///
/// # Arguments
///
/// * `vmm` - Reference to a `Vmm` instance.
/// * `process_pid` - PID of the process.
/// * `module_name` - Name of the module to find.
///
/// # Returns
///
/// An `Option<u64>` containing the base address if found, or `None` if not found.
///
/// # Examples
///
/// ```
/// let base_address = find_base_address(&vmm, pid, "smss.exe").expect("Module not found");
/// println!("Base address: 0x{:X}", base_address);
/// ```
pub fn find_base_address(vmm: &Vmm, process_pid: u32, module_name: &str) -> Option<u64> {
    if let Ok(process) = vmm.process_from_pid(process_pid) {
        match process.get_module_base(module_name) {
            Ok(base) => Some(base),
            Err(e) => {
                println!("Failed to find {} base: {}", module_name, e);
                None
            }
        }
    } else {
        None
    }
}

/// Attempts to fix the CR3 register for a given process and module.
///
/// # Arguments
///
/// * `vmm` - Reference to a `Vmm` instance.
/// * `process` - Reference to a `VmmProcess` instance representing the target process.
/// * `target_module` - Name of the target module.
/// * `pid` - PID of the process.
///
/// # Returns
///
/// A `Result<bool, Box<dyn Error>>` indicating success (`true`) or failure (`false`).
///
/// # Examples
///
/// ```
/// let success = fix_cr3(&vmm, &process, "smss.exe", pid).expect("Failed to fix CR3");
/// if success {
///     println!("Successfully fixed CR3 register.");
/// } else {
///     println!("Failed to fix CR3 register.");
/// }
/// ```
pub fn fix_cr3(vmm: &Vmm, process: &VmmProcess, target_module: &str, pid: u32) -> Result<bool, Box<dyn Error>> {
    let mut possible_dtbs = Vec::new();

    loop {
        if let Ok(progress_percent) = vmm.vfs_read("\\misc\\procinfo\\progress_percent.txt", 3, 0) {
            if progress_percent.len() == 3 {
                break;
            }
        }
        thread::sleep(time::Duration::from_millis(500));
    }

    let dtbs = vmm.vfs_read("\\misc\\procinfo\\dtb.txt", 0x80000, 0)?;
    let result = String::from_utf8_lossy(&dtbs);

    for line in result.lines() {
        let mut split = line.split_whitespace().filter(|s| !s.is_empty());
        if let (Some(_), Some("0"), Some(dtb)) = (split.next(), split.next(), split.next()) {
            if let Ok(dtb_value) = u64::from_str_radix(dtb, 16) {
                possible_dtbs.push(dtb_value);
            }
        }
    }

    for dtb in &possible_dtbs {
        if vmm.set_config(CONFIG_OPT_PROCESS_DTB | pid as u64, *dtb).is_ok() {
            if process.get_module_base(target_module).is_ok() {
                return Ok(true);
            }
        }
    }

    Ok(false)
}
