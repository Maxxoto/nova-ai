//! Prints xcap's display list next to the OS-level NSScreen names.
//! cargo run --release --example display_names

fn main() {
    let mut displays = nova_shell::displays::xcap_list();
    let names = nova_shell::displays::screen_names_probe();
    for d in displays.iter_mut() {
        if let Some(name) = names.get(&d.id) {
            d.name = name.clone();
        }
        println!(
            "id={} primary={} {}x{} -> {:?}",
            d.id, d.primary, d.width, d.height, d.name
        );
    }
}
