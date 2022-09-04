use hemtt_paa::Paa;

#[test]
fn to_dxt5() {
    let image = image::open("tests/ace.jpg").unwrap();
    let mut out = std::fs::File::create("tests/ace.paa").unwrap();
    Paa::write(&mut image.into_rgba8(), &mut out).unwrap();
}
