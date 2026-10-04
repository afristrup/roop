use roop_weave::{Activation, Tensor, spectral_norm};

fn matrix(rows: usize, cols: usize, data: Vec<f64>) -> Tensor {
    Tensor {
        name: "w".into(),
        dims: vec![rows, cols],
        data,
    }
}

#[test]
fn the_spectral_norm_of_known_matrices() {
    let diagonal = matrix(2, 2, vec![3.0, 0.0, 0.0, 1.0]);
    assert!((spectral_norm(&diagonal) / 1.01 - 3.0).abs() < 1e-9);
    let rotation = matrix(2, 2, vec![0.0, -2.0, 2.0, 0.0]);
    assert!((spectral_norm(&rotation) / 1.01 - 2.0).abs() < 1e-9);
    let row = matrix(1, 3, vec![1.0, 2.0, 2.0]);
    assert!((spectral_norm(&row) / 1.01 - 3.0).abs() < 1e-9);
    assert_eq!(spectral_norm(&matrix(2, 2, vec![0.0; 4])), 0.0);
}

#[test]
fn no_activation_is_steeper_than_its_stated_bound() {
    let all = [
        Activation::Identity,
        Activation::Cauchy,
        Activation::Softsign,
        Activation::Relu,
        Activation::Tanh,
        Activation::Sigmoid,
        Activation::Silu,
        Activation::Gelu,
    ];
    for act in all {
        let h = 1e-6;
        let steepest = (-80_000..80_000)
            .map(|i| {
                let z = i as f64 / 10_000.0 + 1e-7;
                ((act.eval(z + h) - act.eval(z - h)) / (2.0 * h)).abs()
            })
            .fold(0.0, f64::max);
        assert!(steepest <= act.lipschitz() + 1e-6, "{act:?}: {steepest}");
        assert!(steepest > 0.9 * act.lipschitz(), "{act:?}: {steepest}");
    }
}
