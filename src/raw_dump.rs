fn print_dump_header(ptr: u64, length: usize) {
    print!("{:width$} ", " ", width = format!("{:X}", ptr + (length as u64)).len());
    for x in 0..8 {
        print!("{:02X} ", x);
    }
    print!(" ");
    for x in 8..0x10 {
        print!("{:02X} ", x);
    }
    print!("│ ");
    for x in 0..0x10 {
        print!("{:X}", x);
    }
    print!("\n");
    print!("{:width$} ", " ", width = format!("{:X}", ptr + (length as u64)).len());
    for x in 0..8 {
        print!("───");
    }
    print!("─");
    for x in 8..0x10 {
        print!("───");
    }
    print!("┼─");
    for x in 0..0x10 {
        print!("─");
    }
    print!("\n");
}

fn print_address(address: u64, length: usize) {
    print!("{:0width$X} ", address, width = format!("{:X}", address + length as u64).len());
}

fn print_raw(address: u64, count: usize) {
    unsafe {
        for by in 0..0x10 {
            if by >= count {
                print!("   ");
            }
            else {
                print!("{:02X} ", *((address + (by as u64)) as *mut u8));
            }
            if by == 0x7 {
                print!(" ")
            }
            else if by == 0xF {
                print!("│ ");
            }
        }
    }
}

// Copied from skyline-rs, thanks jam1garner
fn to_ascii_dots(x: u8) -> char {
    match x {
        0..=0x1F | 0x7F..=0xA0 | 0xAD => '.',
        x => x as char,
    }
}


fn print_pretty(address: u64, count: usize) {
    unsafe {
        for ch in 0..count {
            print!("{}", to_ascii_dots(*((address + (ch as u64)) as *mut u8)));
        }
        print!("\n");
    }
}

fn print_row(ptr: u64, length: usize, row: usize) {
    let row_address: u64 = ptr + ((row * 0x10) as u64);
    let mut bytes_to_print: usize = length - row * 0x10;
    if bytes_to_print > 0x10 {
        bytes_to_print = 0x10;
    }
    print_address(row_address, length);
    print_raw(row_address, bytes_to_print);
    print_pretty(row_address, bytes_to_print);
}

pub fn perform(ptr: u64, length: usize) {
    let mut rows = length / 0x10;
    if length % 0x10 != 0 {
        rows += 1;
    }
    print_dump_header(ptr, length);
    for row in 0..rows {
        print_row(ptr, length, row);
    }
}
