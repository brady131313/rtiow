use std::io::Write;

use rtiow::color::Color;

fn main() {
    let image_width = 256;
    let image_height = 256;

    println!("P3\n{image_width} {image_height}\n255");
    for j in 0..image_height {
        eprint!("\rScanlines remaining: {} ", image_height - j);
        std::io::stderr().flush().unwrap();

        for i in 0..image_width {
            let pixel_color = Color::new(
                i as f64 / (image_width - 1) as f64,
                j as f64 / (image_height - 1) as f64,
                0.0,
            );

            println!("{pixel_color}");
        }
    }

    eprintln!("\rDone.                         ")
}
