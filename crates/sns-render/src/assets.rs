use sns_model::DeviceSymbol;
/// The exact same offline assets are used by the GUI and document renderers.
pub fn symbol_svg(symbol: DeviceSymbol, diagram: bool) -> &'static [u8] {
    macro_rules! asset {
        ($name:literal) => {
            if diagram {
                include_bytes!(concat!("../../../assets/network/diagram/", $name, ".svg"))
                    .as_slice()
            } else {
                include_bytes!(concat!("../../../assets/network/icon/", $name, ".svg")).as_slice()
            }
        };
    }
    match symbol {
        DeviceSymbol::AccessPoint => asset!("access_point"),
        DeviceSymbol::Modem => asset!("modem"),
        DeviceSymbol::Desktop => asset!("desktop"),
        DeviceSymbol::Laptop => asset!("laptop"),
        DeviceSymbol::Server => asset!("server"),
        DeviceSymbol::Printer => asset!("printer"),
        DeviceSymbol::Tablet => asset!("tablet"),
        DeviceSymbol::Router => asset!("router"),
        DeviceSymbol::Switch => asset!("switch"),
        DeviceSymbol::Cloud => asset!("cloud"),
    }
}
