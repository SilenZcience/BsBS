use alloc::string::String;
use alloc::vec::Vec;
use crate::device::cpu;
use crate::shell::registry;

pub fn register() {
    registry::register("cpuinfo", "Show CPU information", run);
}

fn run(_args: &[String]) {
    println!("CPU Information");
    println!("---------------");

    println!("Vendor:            {}", cpu::vendor());

    let (eax, ebx, ecx, edx) = crate::device::cpu::cpuid(1);
    let family = ((eax >> 8) & 0xf) + ((eax >> 20) & 0xff);
    let model = ((eax >> 4) & 0xf) + (((eax >> 16) & 0xf) << 4);
    let stepping = eax & 0xf;
    let logical_cpus = ((ebx >> 16) & 0xff) as usize;
    println!("Family/Model/Step: {}/{}/{}", family, model, stepping);
    println!("Logical CPUs:      {}", logical_cpus);

    println!("Model name:        {}", cpu::brand());

    println!("Features:");
    let mut features: Vec<&str> = Vec::new();
    if ecx & (1 <<  0) != 0 { features.push("sse3"             ); }
    if ecx & (1 <<  1) != 0 { features.push("pclmulqdq"        ); }
    if ecx & (1 <<  2) != 0 { features.push("dtes64"           ); }
    if ecx & (1 <<  3) != 0 { features.push("monitor"          ); }
    if ecx & (1 <<  4) != 0 { features.push("ds-cpl"           ); }
    if ecx & (1 <<  5) != 0 { features.push("vmx"              ); }
    if ecx & (1 <<  6) != 0 { features.push("smx"              ); }
    if ecx & (1 <<  7) != 0 { features.push("eist"             ); }
    if ecx & (1 <<  8) != 0 { features.push("tm2"              ); }
    if ecx & (1 <<  9) != 0 { features.push("ssse3"            ); }
    if ecx & (1 << 10) != 0 { features.push("cnxt-id"          ); }
    if ecx & (1 << 11) != 0 { features.push("sdbg"             ); }
    if ecx & (1 << 12) != 0 { features.push("fma"              ); }
    if ecx & (1 << 13) != 0 { features.push("cmpxchg16b"       ); }
    if ecx & (1 << 14) != 0 { features.push("xtprupdatecontrol"); }
    if ecx & (1 << 15) != 0 { features.push("pdcm"             ); }
    if ecx & (1 << 17) != 0 { features.push("pcid"             ); }
    if ecx & (1 << 18) != 0 { features.push("dca"              ); }
    if ecx & (1 << 19) != 0 { features.push("sse4.1"           ); }
    if ecx & (1 << 20) != 0 { features.push("sse4.2"           ); }
    if ecx & (1 << 21) != 0 { features.push("x2apic"           ); }
    if ecx & (1 << 22) != 0 { features.push("movbe"            ); }
    if ecx & (1 << 23) != 0 { features.push("popcnt"           ); }
    if ecx & (1 << 24) != 0 { features.push("tsc-deadline"     ); }
    if ecx & (1 << 25) != 0 { features.push("aesni"            ); }
    if ecx & (1 << 26) != 0 { features.push("xsave"            ); }
    if ecx & (1 << 27) != 0 { features.push("osxsave"          ); }
    if ecx & (1 << 28) != 0 { features.push("avx"              ); }
    if ecx & (1 << 29) != 0 { features.push("f16c"             ); }
    if ecx & (1 << 30) != 0 { features.push("rdrand"           ); }

    if edx & (1 <<  0) != 0 { features.push("fpu"   ); }
    if edx & (1 <<  1) != 0 { features.push("vme"   ); }
    if edx & (1 <<  2) != 0 { features.push("de"    ); }
    if edx & (1 <<  3) != 0 { features.push("pse"   ); }
    if edx & (1 <<  4) != 0 { features.push("tsc"   ); }
    if edx & (1 <<  5) != 0 { features.push("msr"   ); }
    if edx & (1 <<  6) != 0 { features.push("pae"   ); }
    if edx & (1 <<  7) != 0 { features.push("mce"   ); }
    if edx & (1 <<  8) != 0 { features.push("cx8"   ); }
    if edx & (1 <<  9) != 0 { features.push("apic"  ); }
    if edx & (1 << 11) != 0 { features.push("sep"   ); }
    if edx & (1 << 12) != 0 { features.push("mtrr"  ); }
    if edx & (1 << 13) != 0 { features.push("pge"   ); }
    if edx & (1 << 14) != 0 { features.push("mca"   ); }
    if edx & (1 << 15) != 0 { features.push("cmov"  ); }
    if edx & (1 << 16) != 0 { features.push("pat"   ); }
    if edx & (1 << 17) != 0 { features.push("pse-36"); }
    if edx & (1 << 18) != 0 { features.push("psn"   ); }
    if edx & (1 << 19) != 0 { features.push("clfsh" ); }
    if edx & (1 << 21) != 0 { features.push("ds"    ); }
    if edx & (1 << 22) != 0 { features.push("acpi"  ); }
    if edx & (1 << 23) != 0 { features.push("mmx"   ); }
    if edx & (1 << 24) != 0 { features.push("fxsr"  ); }
    if edx & (1 << 25) != 0 { features.push("sse"   ); }
    if edx & (1 << 26) != 0 { features.push("sse2"  ); }
    if edx & (1 << 27) != 0 { features.push("ss"    ); }
    if edx & (1 << 28) != 0 { features.push("htt"   ); }
    if edx & (1 << 29) != 0 { features.push("tm"    ); }
    if edx & (1 << 31) != 0 { features.push("pbe"   ); }

    if features.is_empty() {
        println!("  (none reported)");
    } else {
        let mut first = true;
        for feature in features {
            if !first {
                print!(" ");
            }
            print!("{}", feature);
            first = false;
        }
        println!("");
    }
}
