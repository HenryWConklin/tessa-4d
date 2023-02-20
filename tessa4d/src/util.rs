pub fn approx_equal(a: f32, b: f32, eps: f32) -> bool {
    (a - b).abs() < eps
}