use qpmu_api::{export, Plugin};

struct Echo;

impl Plugin for Echo {
    fn test(name: String) -> Vec<String> {
        vec![name]
    }
}

export!(Echo with_types_in qpmu_api::bindings);
