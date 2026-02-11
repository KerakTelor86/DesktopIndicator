use winvd::get_desktop_count;

fn main() {
    println!("Desktops: {:?}", get_desktop_count().unwrap());
}
