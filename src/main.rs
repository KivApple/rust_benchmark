use std::time::Instant;
use std::collections::HashMap;
use std::cmp::{min, max};
use glam::Vec3;

fn mid_vertex_for_edge(cache: &mut HashMap<(u32, u32), u32>, vertexes: &mut Vec<Vec3>, first: u32, second: u32) -> u32 {
	let key = (min(first, second), max(first, second));
	*cache.entry(key).or_insert_with(|| {
		vertexes.push((vertexes[first as usize] + vertexes[second as usize]).normalize());
		(vertexes.len() - 1) as u32
	})
}

fn subdivide_mesh(vertexes: &mut Vec<Vec3>, triangles: &Vec<(u32, u32, u32)>, cache: &mut HashMap<(u32, u32), u32>, result: &mut Vec<(u32, u32, u32)>) {
	cache.clear();
	result.clear();
	for triangle in triangles {
		let mid = (
			mid_vertex_for_edge(cache, vertexes, triangle.0, triangle.1),
			mid_vertex_for_edge(cache, vertexes, triangle.1, triangle.2),
			mid_vertex_for_edge(cache, vertexes, triangle.2, triangle.0)
		);
		result.push((triangle.0, mid.0, mid.2));
		result.push((triangle.1, mid.1, mid.0));
		result.push((triangle.2, mid.2, mid.1));
		result.push(mid);
	}
	debug_assert!(result.len() == triangles.len() * 4);
	debug_assert!(cache.len() == triangles.len() + triangles.len() / 2);
}

fn generate_mesh(subdivision_count: u32) -> (Vec<Vec3>, Vec<(u32, u32, u32)>) {
	const X: f32 = 0.525731112119133606;
	const Z: f32 = 0.850650808352039932;
	const N: f32 = 0.0;
	
	let mut vertexes = vec![
		Vec3::new(-X, N, Z), 
		Vec3::new(X, N, Z), 
		Vec3::new(-X, N, -Z), 
		Vec3::new(X, N, -Z),
		Vec3::new(N, Z, X), 
		Vec3::new(N, Z, -X), 
		Vec3::new(N, -Z, X), 
		Vec3::new(N, -Z, -X),
		Vec3::new(Z, X, N), 
		Vec3::new(-Z, X, N), 
		Vec3::new(Z, -X, N),
		Vec3::new(-Z, -X, N)
	];

	let mut triangles = vec![
		(0, 4, 1), (0, 9, 4), (9, 5, 4), (4, 5, 8), (4, 8, 1),
		(8, 10, 1), (8, 3, 10), (5, 3, 8), (5, 2, 3), (2, 7, 3),
		(7, 10, 3), (7, 6, 10), (7, 11, 6), (11, 0, 6), (0, 1, 6),
		(6, 1, 10), (9, 0, 11), (9, 11, 2), (9, 2, 5), (7, 2, 11)
	];

	let mut predicted_vertex_count = vertexes.len();
	let mut predicted_triangle_count = triangles.len();
	let mut predicted_cache_size = 0;
	for _ in 0..subdivision_count {
		predicted_cache_size = predicted_triangle_count + predicted_triangle_count / 2;
		predicted_vertex_count = predicted_vertex_count * 4 - 6;
		predicted_triangle_count *= 4;
	}

	vertexes.reserve(predicted_vertex_count - vertexes.len());
	triangles.reserve(predicted_triangle_count - triangles.len());
	let mut tmp_triangles = Vec::with_capacity(predicted_triangle_count);
	let mut cache = HashMap::with_capacity(predicted_cache_size);

	for _ in 0..subdivision_count {
		subdivide_mesh(&mut vertexes, &triangles, &mut cache, &mut tmp_triangles);
		std::mem::swap(&mut triangles, &mut tmp_triangles);
	}

	debug_assert!(vertexes.len() == predicted_vertex_count);
	debug_assert!(triangles.len() == predicted_triangle_count);
	debug_assert!(cache.len() == predicted_cache_size);

	(vertexes, triangles)
}

#[inline(never)]
fn run_test() -> usize {
	let (vertexes, triangles) = generate_mesh(4);
	vertexes.len() * triangles.len()
}

fn main() {
	let start = Instant::now();
	let count = 10000;
	for _ in 0..count {
		run_test();
	}
	let elapsed = start.elapsed();
	println!("Mesh generation time: {} us", elapsed.as_micros() / count);
}
