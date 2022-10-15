use std::time::Instant;

mod icosphere;
mod font_loader;

#[inline(never)]
fn run_test_icosphere() -> usize {
	let (vertexes, triangles) = icosphere::generate_mesh(4);
	vertexes.len() * triangles.len()
}

#[inline(never)]
fn run_test_font_loader(data: &[u8]) -> usize {
	let (texture_data, glyphs) = font_loader::PF2Loader::new(data).load().unwrap();
	texture_data.len() * glyphs.len()
}

fn main() {
	let start = Instant::now();
	let count = 10000;
	for _ in 0..count {
		run_test_icosphere();
	}
	let elapsed = start.elapsed();
	println!("Mesh generation time: {} us", elapsed.as_micros() / count);

	let start = Instant::now();
	let count = 1000;
	for _ in 0..count {
		run_test_font_loader(include_bytes!("../assets/DroidSans-32.pf2"));
	}
	let elapsed = start.elapsed();
	println!("Font loading time: {} us", elapsed.as_micros() / count);
}
