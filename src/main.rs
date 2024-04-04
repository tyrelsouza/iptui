use cursive::align::HAlign;
use cursive::event::EventResult;
use cursive::traits::*;
use cursive::views::{Dialog, OnEventView, SelectView, TextView};
use cursive::Cursive;
use ifcfg::AddressFamily;

fn main() {
    // Get a list of all ifaces and their associated data.
    let ifaces = ifcfg::IfCfg::get().expect("could not get interfaces");
    let mut content: Vec<String> = ifaces.iter().map(|iface| iface.name.clone()).collect();

    let mut select = SelectView::new()
        .h_align(HAlign::Center);

    // Grab just the names from the Interfaces.
    content.sort();
    select.add_all_str(content);

    // Sets the callback for when "Enter" is pressed.
    select.set_on_submit(|s, iface: &String| show_next_window(s, iface.clone()));

    // Let's override the `j` and `k` keys for navigation
    let select = OnEventView::new(select)
        .on_pre_event_inner('k', |s, _| {
            let cb = s.select_up(1);
            Some(EventResult::Consumed(Some(cb)))
        })
        .on_pre_event_inner('j', |s, _| {
            let cb = s.select_down(1);
            Some(EventResult::Consumed(Some(cb)))
        });

    let mut siv = cursive::default();

    // Let's add a ResizedView to keep the list at a reasonable size
    // (it can scroll anyway).
    siv.add_layer(
        Dialog::around(select.scrollable().fixed_size((20, 10))).title("Select an Interface"),
    );

    siv.run();
}

// Let's put the callback in a separate function to keep it clean,
// but it's not required.
fn show_next_window(siv: &mut Cursive, iface: String) {
    let text: String;

    let ifaces = match ifcfg::IfCfg::get() {
        Ok(ifaces) => ifaces,
        Err(e) => {
            eprintln!("Error getting interfaces: {:?}", e);
            return;
        }
    };

    if let Some(interface) = ifaces.iter().find(|&p| p.name == iface) {
        let mac = interface.mac.clone();
        let name = interface.name.clone();
        let mut ipv4_addresses: Vec<String> = Vec::new();
        let mut ipv6_addresses: Vec<String> = Vec::new();
        for address in &interface.addresses {
            if let Some(addr) = address.address {
                match address.address_family {
                    AddressFamily::IPv4 => {
                        // Get rid of the Port
                        let addy = addr.to_string();
                        if let Some(colon_idx) = addy.find(':') {
                            let trimmed_socket_addr = &addy[..colon_idx];
                            ipv4_addresses.push(trimmed_socket_addr.parse().unwrap());
                        }
                    }
                    AddressFamily::IPv6 => {
                        let addy = addr.to_string();
                        if let Some(colon_idx) = addy.find(']') {
                            let trimmed_socket_addr = &addy[..colon_idx+1];
                            ipv6_addresses.push(trimmed_socket_addr.parse().unwrap());
                        }
                    }
                    AddressFamily::Link => {}
                    AddressFamily::Packet => {}
                    AddressFamily::Unknown(_) => {}
                }
            }
        }
        let mut ipv4_string = String::new();
        for (index, item) in ipv4_addresses.iter().enumerate() {
            if index > 0 {
                ipv4_string.push_str("\n ");
            }
            ipv4_string.push_str(&format!("{}", item));
        }

        let mut ipv6_string = String::new();
        for (index, item) in ipv6_addresses.iter().enumerate() {
            if index > 0 {
                ipv6_string.push_str("\n  ");
            }
            ipv6_string.push_str(&format!("{}", item));
        }

        text = format!("Name:\n  {}\nMac Address:\n  {}\nIPV4 Addresses:\n  {}\nIPV6 Addresses:\n  {}", name, mac, ipv4_string, ipv6_string);
    } else {
        text = format!("Interface not found with the name: {}", iface);
    }

    siv.pop_layer();
    siv.add_layer(Dialog::around(TextView::new(text)).button("Quit", |s| s.quit()));
}
