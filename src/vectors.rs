pub fn scale(a: &Vec<f32>, by: f32) -> Vec<f32> {
    a.iter().map(|a| a * by).collect()
}

pub fn sum(a: &Vec<f32>, b: &Vec<f32>) -> Vec<f32> {
    assert_eq!(a.len(), b.len());

    a.iter().zip(b.iter()).map(|(a_i, b_i)| a_i + b_i).collect()
}

/// a - b
pub fn difference(a: &Vec<f32>, b: &Vec<f32>) -> Vec<f32> {
    assert_eq!(a.len(), b.len());

    a.iter().zip(b.iter()).map(|(a_i, b_i)| a_i - b_i).collect()
}

pub fn norm(a: &Vec<f32>) -> f32 {
    a.iter().map(|x| x.abs()).sum::<f32>().sqrt()
}

pub fn distance(a: &Vec<f32>, b: &Vec<f32>) -> f32 {
    norm(&difference(a, b))
}

pub fn sum_many(xs: &Vec<Vec<f32>>) -> Vec<f32> {
    let len = xs[0].len();
    let zero = vec![0.; len];
    xs.iter().fold(zero, |a, x| sum(&a, x))
}

pub fn average(xs: &Vec<Vec<f32>>) -> Vec<f32> {
    let len = xs.len() as f32;
    let sum = sum_many(xs);
    sum.iter().map(|a| a / len).collect()
}
