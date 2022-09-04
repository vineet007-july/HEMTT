#[test]
fn from_dxt1() {
    let file = std::fs::File::open("tests/dxt1.paa").unwrap();
    let paa = hemtt_paa::Paa::read(file).unwrap();
    paa.maps[0].get_image();
}

#[test]
fn from_dxt5() {
    let file = std::fs::File::open("tests/dxt5.paa").unwrap();
    let paa = hemtt_paa::Paa::read(file).unwrap();
    paa.maps[0].get_image();
}
