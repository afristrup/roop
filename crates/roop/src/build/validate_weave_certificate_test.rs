use crate::build::validate_weave_certificate;
use roop_weave::{emit_model, parse_model};

#[test]
fn weave_certificate_validation_rejects_a_call_that_overwrites_its_source() {
    let model = parse_model(
        r#"{"name":"net","width":2,"outputs":1,"step":0.25,"layers":[{"kind":"mlp","activation":"relu","w1":[[0.1,0.2],[0.3,0.4]],"b1":[0.1,0.2],"w2":[[0.1,0.2],[0.3,0.4]],"b2":[0.1,0.2]}]}"#,
    )
    .unwrap();
    let source = emit_model(&model, false, None, false);
    let source = source.replacen("call mlp<2, 2>(q,", "call mlp<2, 2>(p,", 1);
    let error = validate_weave_certificate(&source, &model).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("does not preserve the source tensor")
    );
}
