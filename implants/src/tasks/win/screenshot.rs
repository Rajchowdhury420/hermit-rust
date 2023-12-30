use screenshots::Screen;
use std::io::Cursor;
use std::time::Instant;

pub async fn screenshot() -> Result<Vec<u8>, std::io::Error> {
    let start = Instant::now();
    // let screens = Screen::all().unwrap();

    // for screen in screens {
    //     println!("capturer {screen:?}");
    //     let mut image = screen.capture().unwrap();
    // }

    let screen = Screen::from_point(100, 100).unwrap();
    println!("capturer: {screen:?}");

    let img = screen.capture_area(300, 300, 300, 300).unwrap();

    let mut bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), screenshots::image::ImageOutputFormat::Png);

    Ok(bytes)
}