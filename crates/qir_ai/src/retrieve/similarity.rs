pub fn l2_norm(v: &[f32]) -> f32 {
    let mut sum = 0.0f32;
    for x in v {
        sum += x * x;
    }
    sum.sqrt()
}

pub fn cosine_similarity(a: &[f32], b: &[f32], a_norm: f32, b_norm: f32) -> f32 {
    let mut dot = 0.0f32;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
    }
    dot / (a_norm * b_norm)
}

