use crate::{Activation, Layer, LossKind, Model, Optimizer, Tensor, WeaveError};
use serde_json::Value;

fn field<'a>(
    parent: &'a Value,
    path: &str,
    key: &str,
    expected: &'static str,
) -> Result<&'a Value, WeaveError> {
    parent.get(key).ok_or_else(|| WeaveError::Field {
        path: format!("{path}{key}"),
        expected,
    })
}

fn count(parent: &Value, key: &str) -> Result<usize, WeaveError> {
    match field(parent, "", key, "a positive whole number")?.as_u64() {
        Some(n) if n > 0 => Ok(n as usize),
        _ => Err(WeaveError::Field {
            path: key.into(),
            expected: "a positive whole number",
        }),
    }
}

fn numbers(value: &Value, path: &str) -> Result<Vec<f64>, WeaveError> {
    let bad = || WeaveError::Field {
        path: path.into(),
        expected: "a list of numbers",
    };
    let list = value.as_array().ok_or_else(bad)?;
    list.iter().map(|x| x.as_f64().ok_or_else(bad)).collect()
}

fn vector(layer: &Value, path: &str, key: &str, name: String) -> Result<Tensor, WeaveError> {
    let data = numbers(
        field(layer, path, key, "a list of numbers")?,
        &format!("{path}{key}"),
    )?;
    Ok(Tensor {
        name,
        dims: vec![data.len()],
        data,
    })
}

fn matrix(
    layer: &Value,
    path: &str,
    key: &str,
    name: String,
    rows: usize,
    cols: usize,
) -> Result<Tensor, WeaveError> {
    let at = format!("{path}{key}");
    let list = field(layer, path, key, "a list of rows")?;
    let rows_found = list.as_array().ok_or_else(|| WeaveError::Field {
        path: at.clone(),
        expected: "a list of rows",
    })?;
    let mut data = Vec::new();
    for row in rows_found {
        let row = numbers(row, &at)?;
        if row.len() != cols {
            let found = format!("a row of {}", row.len());
            return Err(WeaveError::Shape {
                name,
                expected: format!("{rows} rows of {cols}"),
                found,
            });
        }
        data.extend(row);
    }
    if rows_found.len() != rows {
        let found = format!("{} rows", rows_found.len());
        return Err(WeaveError::Shape {
            name,
            expected: format!("{rows} rows of {cols}"),
            found,
        });
    }
    Ok(Tensor {
        name,
        dims: vec![rows, cols],
        data,
    })
}

fn activation(layer: &Value, path: &str) -> Result<Activation, WeaveError> {
    let name = field(layer, path, "activation", "an activation name")?
        .as_str()
        .ok_or_else(|| WeaveError::Field {
            path: format!("{path}activation"),
            expected: "an activation name",
        })?;
    Activation::parse(name).ok_or_else(|| WeaveError::Activation {
        path: format!("{path}activation"),
        name: name.into(),
    })
}

fn parse_attention(index: usize, layer: &Value, width: usize) -> Result<Layer, WeaveError> {
    let path = format!("layers[{index}].");
    let seq = count(layer, "seq")?;
    if width % seq != 0 {
        let found = format!("a width of {width}");
        return Err(WeaveError::Shape {
            name: format!("layers[{index}]"),
            expected: format!("a width that {seq} rows divide"),
            found,
        });
    }
    let d = width / seq;
    let weight = |key: &str| matrix(layer, &path, key, format!("{key}_{index}"), d, d);
    Ok(Layer::Attention {
        seq,
        wq: weight("wq")?,
        wk: weight("wk")?,
        wv: weight("wv")?,
    })
}

fn parse_conv(index: usize, layer: &Value, width: usize) -> Result<Layer, WeaveError> {
    let path = format!("layers[{index}].");
    let channels = count(layer, "channels")?;
    let kernel = count(layer, "kernel")?;
    let shape = |expected: String, found: String| WeaveError::Shape {
        name: format!("layers[{index}]"),
        expected,
        found,
    };
    if width % channels != 0 {
        return Err(shape(
            format!("a width that {channels} channels divide"),
            format!("a width of {width}"),
        ));
    }
    if kernel % 2 == 0 {
        return Err(shape(
            "an odd kernel".into(),
            format!("a kernel of {kernel}"),
        ));
    }
    let w = matrix(
        layer,
        &path,
        "weight",
        format!("w{index}"),
        channels,
        channels * kernel,
    )?;
    let b = vector(layer, &path, "bias", format!("b{index}"))?;
    if b.dims[0] != channels {
        let found = format!("{} numbers", b.dims[0]);
        return Err(WeaveError::Shape {
            name: b.name,
            expected: format!("{channels} numbers"),
            found,
        });
    }
    Ok(Layer::Conv {
        channels,
        kernel,
        w,
        b,
    })
}

fn parse_layer(index: usize, layer: &Value, width: usize) -> Result<Layer, WeaveError> {
    let path = format!("layers[{index}].");
    let kind = field(layer, &path, "kind", "a layer kind")?
        .as_str()
        .unwrap_or("");
    if kind == "conv" {
        return parse_conv(index, layer, width);
    }
    if kind == "attention" {
        return parse_attention(index, layer, width);
    }
    let act = activation(layer, &path)?;
    match kind {
        "leapfrog" => {
            let b = vector(layer, &path, "bias", format!("b{index}"))?;
            let w = matrix(
                layer,
                &path,
                "weight",
                format!("w{index}"),
                b.dims[0],
                width,
            )?;
            Ok(Layer::Leapfrog { act, w, b })
        }
        "mlp" => {
            let b1 = vector(layer, &path, "b1", format!("b1_{index}"))?;
            let m = b1.dims[0];
            let w1 = matrix(layer, &path, "w1", format!("w1_{index}"), m, width)?;
            let w2 = matrix(layer, &path, "w2", format!("w2_{index}"), width, m)?;
            let b2 = vector(layer, &path, "b2", format!("b2_{index}"))?;
            if b2.dims[0] != width {
                let found = format!("{} numbers", b2.dims[0]);
                return Err(WeaveError::Shape {
                    name: b2.name,
                    expected: format!("{width} numbers"),
                    found,
                });
            }
            Ok(Layer::Mlp {
                act,
                w1,
                b1,
                w2,
                b2,
            })
        }
        other => Err(WeaveError::Layer {
            index,
            kind: other.into(),
        }),
    }
}

fn parse_loss(root: &Value) -> Result<LossKind, WeaveError> {
    let Some(value) = root.get("loss") else {
        return Ok(LossKind::Mse);
    };
    let name = value.as_str().unwrap_or("");
    LossKind::parse(name).ok_or_else(|| WeaveError::Loss(name.into()))
}

fn rate(optimizer: &Value, key: &str, default: f64) -> Result<f64, WeaveError> {
    let bad = || WeaveError::Field {
        path: format!("optimizer.{key}"),
        expected: "a number between 0 and 1",
    };
    match optimizer.get(key) {
        None => Ok(default),
        Some(v) => v
            .as_f64()
            .filter(|x| (0.0..1.0).contains(x))
            .ok_or_else(bad),
    }
}

fn parse_optimizer(root: &Value) -> Result<Optimizer, WeaveError> {
    let Some(value) = root.get("optimizer") else {
        return Ok(Optimizer::Sgd);
    };
    match value.get("kind").and_then(Value::as_str).unwrap_or("") {
        "sgd" => Ok(Optimizer::Sgd),
        "momentum" => Ok(Optimizer::Momentum {
            beta: rate(value, "beta", 0.9)?,
        }),
        "adam" => Ok(Optimizer::Adam {
            beta1: rate(value, "beta1", 0.9)?,
            beta2: rate(value, "beta2", 0.999)?,
        }),
        other => Err(WeaveError::Optimizer(other.into())),
    }
}

/// Reads a model from its JSON, and checks every shape so that nothing wrong
/// reaches the code that is written from it.
pub fn parse_model(text: &str) -> Result<Model, WeaveError> {
    let root: Value = serde_json::from_str(text).map_err(|e| WeaveError::Json(e.to_string()))?;
    let name = root.get("name").and_then(Value::as_str).unwrap_or("model");
    let is_name = |c: char| c.is_ascii_alphanumeric() || c == '_';
    if name.is_empty()
        || !name.chars().all(is_name)
        || name.starts_with(|c: char| c.is_ascii_digit())
    {
        return Err(WeaveError::Name(name.into()));
    }
    let width = count(&root, "width")?;
    let outputs = count(&root, "outputs")?;
    if outputs > width {
        return Err(WeaveError::Outputs { outputs, width });
    }
    let step = field(&root, "", "step", "a number")?
        .as_f64()
        .ok_or_else(|| WeaveError::Field {
            path: "step".into(),
            expected: "a number",
        })?;
    let listed = field(&root, "", "layers", "a list of layers")?
        .as_array()
        .ok_or_else(|| WeaveError::Field {
            path: "layers".into(),
            expected: "a list of layers",
        })?;
    if listed.is_empty() {
        return Err(WeaveError::NoLayers);
    }
    let layers = listed
        .iter()
        .enumerate()
        .map(|(i, layer)| parse_layer(i, layer, width))
        .collect::<Result<_, _>>()?;
    Ok(Model {
        name: name.into(),
        width,
        outputs,
        step,
        layers,
        loss: parse_loss(&root)?,
        optimizer: parse_optimizer(&root)?,
    })
}
